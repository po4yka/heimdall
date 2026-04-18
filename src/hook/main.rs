/// Binary entry point for `heimdall-hook`.
///
/// Thin wrapper around `hook::main_impl()` from the shared library crate.
///
/// Exit contract:
/// - ALWAYS exits 0 — a non-zero exit would surface an error dialog to the user.
/// - ALWAYS prints `{}` on stdout.
/// - NEVER blocks for more than 1 second (stdin read is guarded by a timeout).
fn main() {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "warn".into()),
        )
        .init();

    claude_usage_tracker::hook::main_impl();
    std::process::exit(0);
}
