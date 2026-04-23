import Foundation
import Testing
@testable import HeimdallPlatformMac
import HeimdallDomain

struct TerminalAuthCommandRunnerTests {
    @Test
    func scriptContentsSelfDeletesOnExit() {
        let script = TerminalAuthCommandRunner.scriptContents(
            provider: .codex,
            title: "Codex Login",
            command: "codex login",
            scriptPath: "/tmp/heimdallbar-codex-auth.command"
        )

        #expect(script.contains("trap cleanup EXIT HUP INT TERM"))
        #expect(script.contains("rm -f -- \"$SCRIPT_PATH\""))
        #expect(script.contains("candidate=\"$(/bin/zsh -ilc 'command -v -- codex'"))
        #expect(script.contains("CLI_PATH=\"$(resolve_cli_path)\""))
        #expect(script.contains("\"$CLI_PATH\" 'login'"))
    }

    @Test
    func runnerWritesScriptAndLaunchesTerminal() throws {
        let tempDirectory = try Self.makeTempDirectory()
        let recorder = PathRecorder()
        let runner = TerminalAuthCommandRunner(
            temporaryDirectoryProvider: { tempDirectory },
            terminalLauncher: { scriptURL in
                recorder.record(scriptURL.path())
            }
        )

        try runner.runAuthCommand(
            provider: .claude,
            title: "Claude Login",
            command: "claude /login"
        )

        let launchedPath = try #require(recorder.value)
        let contents = try String(contentsOfFile: launchedPath, encoding: .utf8)
        #expect(FileManager.default.fileExists(atPath: launchedPath))
        #expect(contents.contains("Claude Auth Recovery"))
        #expect(contents.contains("trap cleanup EXIT HUP INT TERM"))
        #expect(contents.contains("\"$CLI_PATH\" '/login'"))
    }

    @Test
    func deviceAuthUsesResolvedExecutablePath() {
        let script = TerminalAuthCommandRunner.scriptContents(
            provider: .codex,
            title: "Codex Device Login",
            command: "codex login --device-auth",
            scriptPath: "/tmp/heimdallbar-codex-auth.command"
        )

        #expect(script.contains("\"$CLI_PATH\" 'login' '--device-auth'"))
    }

    @Test
    func launcherFailureRemovesTemporaryScript() throws {
        let tempDirectory = try Self.makeTempDirectory()
        let expectedPath = tempDirectory
            .appendingPathComponent("heimdallbar-codex-auth.command", isDirectory: false)
            .path()
        let runner = TerminalAuthCommandRunner(
            temporaryDirectoryProvider: { tempDirectory },
            terminalLauncher: { _ in
                throw NSError(domain: "TerminalAuthCommandRunnerTests", code: 1)
            }
        )

        #expect(throws: NSError.self) {
            try runner.runAuthCommand(
                provider: .codex,
                title: "Codex Login",
                command: "codex login"
            )
        }
        #expect(!FileManager.default.fileExists(atPath: expectedPath))
    }

    private static func makeTempDirectory() throws -> URL {
        let url = FileManager.default.temporaryDirectory
            .appendingPathComponent(UUID().uuidString, isDirectory: true)
        try FileManager.default.createDirectory(at: url, withIntermediateDirectories: true)
        return url
    }
}

private final class PathRecorder: @unchecked Sendable {
    private let lock = NSLock()
    private var storage: String?

    var value: String? {
        self.lock.lock()
        defer { self.lock.unlock() }
        return self.storage
    }

    func record(_ path: String) {
        self.lock.lock()
        self.storage = path
        self.lock.unlock()
    }
}
