/// Bypass-mode detection for the heimdall-hook binary.
///
/// Walks the ancestor process chain looking for `--dangerously-skip-permissions`
/// in any ancestor's command line. When found, the hook should exit 0 without
/// touching the database so it does not interfere with the bypass session.
///
/// Implementation uses a targeted `ps` shell-out to retrieve only the fields we
/// need (`pid`, `ppid`, `command`) for a single process at a time. This is
/// dramatically faster than loading the entire process table via `sysinfo` on
/// macOS (where `ProcessesToUpdate::All` can take 25+ seconds on cold start).
///
/// The walk is capped at depth 16 to prevent infinite loops. It terminates
/// naturally when the pid reaches 1 (init/launchd) or when `ps` cannot find
/// a process.
const MAX_DEPTH: usize = 16;
const BYPASS_FLAG: &str = "--dangerously-skip-permissions";

/// Details about the ancestor process that triggered bypass mode.
pub struct BypassMatch {
    /// How many hops up the process tree the match was found (1 = direct parent).
    pub depth: u32,
    /// PID of the ancestor process that contained the bypass flag.
    pub ancestor_pid: u32,
    /// First 256 characters of the ancestor process command line.
    pub ancestor_command: String,
}

/// Check whether any ancestor process uses `--dangerously-skip-permissions`.
///
/// Returns `Some(BypassMatch)` if a matching ancestor is found, `None` otherwise.
/// Safe to call from any context: never panics.
pub fn detect_bypass() -> Option<BypassMatch> {
    detect_bypass_from(std::process::id())
}

/// Inner implementation — accepts a starting pid so tests can exercise the
/// walk logic without spoofing process ancestry.
pub fn detect_bypass_from(start_pid: u32) -> Option<BypassMatch> {
    let mut current_pid = start_pid;

    for depth in 1..=(MAX_DEPTH as u32) {
        // ps -o pid=,ppid=,command= -p <pid>
        // Fields: pid, parent pid, full command line.
        // On macOS and Linux `command=` includes arguments.
        let output = match std::process::Command::new("ps")
            .args(["-o", "pid=,ppid=,command=", "-p", &current_pid.to_string()])
            .output()
        {
            Ok(o) => o,
            Err(_) => return None, // ps not available
        };

        if !output.status.success() || output.stdout.is_empty() {
            // Process not found — we've reached the top or it exited.
            return None;
        }

        let line = String::from_utf8_lossy(&output.stdout);
        let line = line.trim();

        // Parse: first token = pid, second = ppid, rest = command
        let mut parts = line.splitn(3, char::is_whitespace);
        let _pid_str = parts.next().unwrap_or("");
        let ppid_str = parts.next().unwrap_or("0").trim();
        let command = parts.next().unwrap_or("");

        // Check this process's command for the bypass flag.
        if command.contains(BYPASS_FLAG) {
            return Some(BypassMatch {
                depth,
                ancestor_pid: current_pid,
                ancestor_command: command.chars().take(256).collect(),
            });
        }

        // Parse parent pid.
        let ppid: u32 = ppid_str.parse().unwrap_or(0);

        // Terminate at pid 1 (init/launchd) or if we'd loop.
        if ppid <= 1 || ppid == current_pid {
            return None;
        }

        current_pid = ppid;
    }

    // Exceeded max depth without finding the flag.
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bypass_walk_does_not_panic_on_current_process() {
        // Calling detect_bypass() from the current process must not panic
        // regardless of what the actual ancestor chain looks like. In CI or when
        // running under `claude --dangerously-skip-permissions` this will return
        // Some; in normal environments it returns None. We only verify liveness.
        let _ = detect_bypass();
    }

    #[test]
    fn walk_does_not_panic_on_depth_limit() {
        // Starting from pid 1 should terminate within MAX_DEPTH
        // without panicking regardless of process tree shape.
        let _ = detect_bypass_from(1);
    }

    #[test]
    fn walk_does_not_panic_on_nonexistent_pid() {
        // A non-existent pid (very large number) must return None without panicking.
        let result = detect_bypass_from(u32::MAX);
        assert!(result.is_none());
    }

    #[test]
    fn walk_terminates_from_own_pid() {
        // Walking from our own pid must terminate cleanly.
        let own_pid = std::process::id();
        let _ = detect_bypass_from(own_pid);
    }

    #[test]
    fn bypass_detection_completes_quickly() {
        // The walk should complete in well under 500ms even on first call.
        let start = std::time::Instant::now();
        let _ = detect_bypass();
        let elapsed = start.elapsed();
        assert!(
            elapsed.as_millis() < 500,
            "bypass check took {}ms — too slow",
            elapsed.as_millis()
        );
    }

    #[test]
    fn nonexistent_pid_returns_none() {
        let result = detect_bypass_from(u32::MAX - 1);
        assert!(result.is_none());
    }
}
