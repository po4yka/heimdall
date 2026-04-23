import Foundation
import HeimdallDomain

public actor LocalNotificationCoordinator: LocalNotificationCoordinating {
    private let authorizationManager: any NotificationAuthorizationManaging
    private let scheduler: any LocalNotificationScheduling
    private let stateStore: any LocalNotificationStatePersisting

    public init(
        authorizationManager: any NotificationAuthorizationManaging,
        scheduler: any LocalNotificationScheduling,
        stateStore: any LocalNotificationStatePersisting
    ) {
        self.authorizationManager = authorizationManager
        self.scheduler = scheduler
        self.stateStore = stateStore
    }

    public func handleConfigChange(previous: HeimdallBarConfig, current: HeimdallBarConfig) async -> AppIssue? {
        guard current.localNotificationsEnabled else {
            if previous.localNotificationsEnabled {
                self.stateStore.clearState()
            }
            return nil
        }
        guard !previous.localNotificationsEnabled else {
            return nil
        }

        self.stateStore.clearState()
        do {
            let granted = try await self.authorizationManager.requestAuthorization()
            if granted {
                return nil
            }
            return Self.permissionDeniedIssue()
        } catch {
            return Self.deliveryIssue("Failed to request notification permission. \(error.localizedDescription)")
        }
    }

    public func process(envelope: ProviderSnapshotEnvelope, config: HeimdallBarConfig) async -> AppIssue? {
        guard config.localNotificationsEnabled else {
            return nil
        }
        guard let notificationState = envelope.localNotificationState else {
            return nil
        }

        let authorizationStatus = await self.authorizationManager.authorizationStatus()
        switch authorizationStatus {
        case .authorized:
            break
        case .denied:
            return Self.permissionDeniedIssue()
        case .notDetermined:
            return nil
        }

        var persisted = self.stateStore.loadState()
        var didChangeState = false

        for condition in notificationState.conditions {
            let wasActive = persisted.lastKnownActive[condition.id] ?? false

            if condition.kind == "daily_cost_threshold" {
                if condition.isActive,
                   let dayKey = condition.dayKey,
                   persisted.lastFiredDayKeys[condition.id] != dayKey
                {
                    do {
                        try await self.scheduler.schedule(
                            Self.request(
                                id: condition.id,
                                generatedAt: notificationState.generatedAt,
                                title: condition.activationTitle,
                                body: condition.activationBody,
                                suffix: "activation"
                            )
                        )
                        persisted.lastFiredDayKeys[condition.id] = dayKey
                        didChangeState = true
                    } catch {
                        self.stateStore.saveState(persisted)
                        return Self.deliveryIssue("Failed to schedule a notification. \(error.localizedDescription)")
                    }
                }

                if persisted.lastKnownActive[condition.id] != condition.isActive {
                    persisted.lastKnownActive[condition.id] = condition.isActive
                    didChangeState = true
                }
                continue
            }

            do {
                if !wasActive && condition.isActive {
                    try await self.scheduler.schedule(
                        Self.request(
                            id: condition.id,
                            generatedAt: notificationState.generatedAt,
                            title: condition.activationTitle,
                            body: condition.activationBody,
                            suffix: "activation"
                        )
                    )
                } else if wasActive && !condition.isActive,
                          let recoveryTitle = condition.recoveryTitle,
                          let recoveryBody = condition.recoveryBody
                {
                    try await self.scheduler.schedule(
                        Self.request(
                            id: condition.id,
                            generatedAt: notificationState.generatedAt,
                            title: recoveryTitle,
                            body: recoveryBody,
                            suffix: "recovery"
                        )
                    )
                }
            } catch {
                self.stateStore.saveState(persisted)
                return Self.deliveryIssue("Failed to schedule a notification. \(error.localizedDescription)")
            }

            if persisted.lastKnownActive[condition.id] != condition.isActive {
                persisted.lastKnownActive[condition.id] = condition.isActive
                didChangeState = true
            }
        }

        if didChangeState {
            self.stateStore.saveState(persisted)
        }
        return nil
    }

    private static func request(
        id: String,
        generatedAt: String,
        title: String,
        body: String,
        suffix: String
    ) -> LocalNotificationRequest {
        LocalNotificationRequest(
            identifier: "heimdall.\(id).\(suffix).\(generatedAt)",
            title: title,
            body: body
        )
    }

    private static func permissionDeniedIssue() -> AppIssue {
        AppIssue(
            kind: .localNotifications,
            message: "Local notifications are enabled, but macOS notification permission is denied. Enable HeimdallBar in System Settings > Notifications."
        )
    }

    private static func deliveryIssue(_ message: String) -> AppIssue {
        AppIssue(kind: .localNotifications, message: message)
    }
}

public struct NoopLocalNotificationCoordinator: LocalNotificationCoordinating {
    public init() {}

    public func handleConfigChange(previous _: HeimdallBarConfig, current _: HeimdallBarConfig) async -> AppIssue? {
        nil
    }

    public func process(envelope _: ProviderSnapshotEnvelope, config _: HeimdallBarConfig) async -> AppIssue? {
        nil
    }
}
