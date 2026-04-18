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

/// Returns `true` if any ancestor process in the current process tree contains
/// `--dangerously-skip-permissions` in its command-line arguments.
///
/// Safe to call from any context: never panics, returns `false` on any error.
pub fn is_bypass_active() -> bool {
    is_bypass_active_from(std::process::id())
}

/// Inner implementation — accepts a starting pid so tests can exercise the
/// walk logic without spoofing process ancestry.
pub fn is_bypass_active_from(start_pid: u32) -> bool {
    let mut current_pid = start_pid;

    for _depth in 0..MAX_DEPTH {
        // ps -o pid=,ppid=,command= -p <pid>
        // Fields: pid, parent pid, full command line.
        // On macOS and Linux `command=` includes arguments.
        let output = match std::process::Command::new("ps")
            .args(["-o", "pid=,ppid=,command=", "-p", &current_pid.to_string()])
            .output()
        {
            Ok(o) => o,
            Err(_) => return false, // ps not available
        };

        if !output.status.success() || output.stdout.is_empty() {
            // Process not found — we've reached the top or it exited.
            return false;
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
            return true;
        }

        // Parse parent pid.
        let ppid: u32 = ppid_str.parse().unwrap_or(0);

        // Terminate at pid 1 (init/launchd) or if we'd loop.
        if ppid <= 1 || ppid == current_pid {
            return false;
        }

        current_pid = ppid;
    }

    // Exceeded max depth without finding the flag.
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bypass_walk_does_not_panic_on_current_process() {
        // Calling is_bypass_active() from the current process must not panic
        // regardless of what the actual ancestor chain looks like. In CI or when
        // running under `claude --dangerously-skip-permissions` this will return
        // true; in normal environments it returns false. We only verify liveness.
        let _ = is_bypass_active();
    }

    #[test]
    fn walk_does_not_panic_on_depth_limit() {
        // Starting from pid 1 (or our own pid) should terminate within MAX_DEPTH
        // without panicking regardless of process tree shape.
        let result = is_bypass_active_from(1);
        // We don't assert the value — just that it didn't panic.
        let _ = result;
    }

    #[test]
    fn walk_does_not_panic_on_nonexistent_pid() {
        // A non-existent pid (very large number) must return false without panicking.
        let result = is_bypass_active_from(u32::MAX);
        assert!(!result);
    }

    #[test]
    fn walk_terminates_from_own_pid() {
        // Walking from our own pid must terminate cleanly.
        let own_pid = std::process::id();
        let _ = is_bypass_active_from(own_pid);
    }

    #[test]
    fn bypass_detection_completes_quickly() {
        // The walk should complete in well under 500ms even on first call.
        let start = std::time::Instant::now();
        let _ = is_bypass_active();
        let elapsed = start.elapsed();
        assert!(
            elapsed.as_millis() < 500,
            "bypass check took {}ms — too slow",
            elapsed.as_millis()
        );
    }
}
