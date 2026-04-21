import Foundation
import HeimdallDomain

public final class AuthCoordinator: Sendable {
    private let runner: any AuthCommandRunning

    public init(runner: any AuthCommandRunning) {
        self.runner = runner
    }

    public func recoveryActions(
        for provider: ProviderID,
        projection: ProviderMenuProjection
    ) -> [AuthRecoveryAction] {
        if !projection.authRecoveryActions.isEmpty {
            return projection.authRecoveryActions
        }
        return self.defaultAuthRecoveryActions(for: provider)
    }

    public func primaryAction(
        for provider: ProviderID,
        projection: ProviderMenuProjection
    ) -> AuthRecoveryAction? {
        self.recoveryActions(for: provider, projection: projection).first
    }

    public func run(
        _ action: AuthRecoveryAction,
        provider: ProviderID
    ) throws {
        guard let launch = self.recoveryLaunch(for: action.actionID, provider: provider) else {
            throw AuthCoordinatorError.unsupportedRecoveryAction(provider, action.actionID)
        }
        try self.runner.runAuthCommand(provider: provider, title: launch.title, command: launch.command)
    }

    public func defaultCommand(
        for action: AuthRecoveryAction,
        provider: ProviderID
    ) -> String {
        if let launch = self.recoveryLaunch(for: action.actionID, provider: provider) {
            return launch.command
        }
        switch (provider, action.actionID) {
        case (.claude, "claude-doctor"):
            return "claude doctor"
        case (.claude, _):
            return "claude login"
        case (.codex, "codex-login-device"):
            return "codex login --device-auth"
        case (.codex, _):
            return "codex login"
        }
    }

    private func defaultAuthRecoveryActions(for provider: ProviderID) -> [AuthRecoveryAction] {
        switch provider {
        case .claude:
            return [
                AuthRecoveryAction(
                    label: "Run Claude Login",
                    actionID: "claude-login",
                    command: "claude login",
                    detail: "Run Claude login to restore desktop subscription OAuth."
                ),
                AuthRecoveryAction(
                    label: "Run Claude Doctor",
                    actionID: "claude-doctor",
                    command: "claude doctor",
                    detail: "Use Claude doctor to diagnose credential, keychain, and environment problems."
                ),
            ]
        case .codex:
            return [
                AuthRecoveryAction(
                    label: "Run Codex Login",
                    actionID: "codex-login",
                    command: "codex login",
                    detail: "Run Codex login to restore ChatGPT-backed auth."
                ),
                AuthRecoveryAction(
                    label: "Run Device Login",
                    actionID: "codex-login-device",
                    command: "codex login --device-auth",
                    detail: "Use device auth when localhost callback login is blocked or headless."
                ),
            ]
        }
    }

    private struct RecoveryLaunch {
        var title: String
        var command: String
    }

    private func recoveryLaunch(
        for actionID: String,
        provider: ProviderID
    ) -> RecoveryLaunch? {
        switch (provider, actionID) {
        case (.claude, "claude-run"):
            return RecoveryLaunch(title: "Run Claude", command: "claude")
        case (.claude, "claude-login"):
            return RecoveryLaunch(title: "Run Claude Login", command: "claude login")
        case (.claude, "claude-doctor"):
            return RecoveryLaunch(title: "Run Claude Doctor", command: "claude doctor")
        case (.codex, "codex-login"):
            return RecoveryLaunch(title: "Run Codex Login", command: "codex login")
        case (.codex, "codex-login-device"):
            return RecoveryLaunch(title: "Run Device Login", command: "codex login --device-auth")
        default:
            return nil
        }
    }
}

public enum AuthCoordinatorError: Error, LocalizedError {
    case unsupportedRecoveryAction(ProviderID, String)

    public var errorDescription: String? {
        switch self {
        case .unsupportedRecoveryAction(let provider, _):
            return "Unsupported \(provider.title) auth recovery action."
        }
    }
}
