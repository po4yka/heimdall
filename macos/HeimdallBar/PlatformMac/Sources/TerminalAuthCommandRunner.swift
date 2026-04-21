import Foundation
import HeimdallDomain
import HeimdallServices

public struct TerminalAuthCommandRunner: AuthCommandRunning {
    public init() {}

    public func runAuthCommand(
        provider: ProviderID,
        title: String,
        command: String
    ) throws {
        let scriptURL = FileManager.default.temporaryDirectory
            .appendingPathComponent("heimdallbar-\(provider.rawValue)-auth.command", isDirectory: false)
        let script = """
        #!/bin/zsh
        export PATH="$HOME/.local/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:$PATH"
        clear
        echo "HeimdallBar \(provider.title) Auth Recovery"
        echo
        if ! command -v \(provider == .claude ? "claude" : "codex") >/dev/null 2>&1; then
          echo "\(provider.title) CLI was not found in PATH."
          echo "Run '\(command)' manually in a shell where the \(provider == .claude ? "claude" : "codex") command exists."
          echo
          read -k '?Press any key to close...'
          exit 1
        fi
        echo "\(title)"
        echo
        echo "Running: \(command)"
        echo
        \(command)
        echo
        if [ "\(provider.rawValue)" = "claude" ]; then
          if [ -f "$HOME/.claude/.credentials.json" ]; then
            echo "Claude OAuth credentials were saved to ~/.claude/.credentials.json."
          else
            echo "Claude OAuth credentials file is still missing."
          fi
        else
          if [ -f "${CODEX_HOME:-$HOME/.codex}/auth.json" ]; then
            echo "Codex auth file is present at ${CODEX_HOME:-$HOME/.codex}/auth.json."
          else
            echo "Codex auth file is still missing."
          fi
        fi
        echo "Return to HeimdallBar and refresh \(provider.title)."
        echo
        read -k '?Press any key to close...'
        """
        try script.write(to: scriptURL, atomically: true, encoding: .utf8)
        try FileManager.default.setAttributes([.posixPermissions: 0o755], ofItemAtPath: scriptURL.path())
        let process = Process()
        process.executableURL = URL(fileURLWithPath: "/usr/bin/open")
        process.arguments = ["-a", "Terminal", scriptURL.path()]
        try process.run()
    }
}
