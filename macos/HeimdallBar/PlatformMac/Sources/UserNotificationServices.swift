import Foundation
import HeimdallServices
import UserNotifications

public final class UserNotificationAuthorizationManager: @unchecked Sendable, NotificationAuthorizationManaging {
    private let center: UNUserNotificationCenter

    public init(center: UNUserNotificationCenter = .current()) {
        self.center = center
    }

    public func authorizationStatus() async -> NotificationAuthorizationStatus {
        await withCheckedContinuation { continuation in
            self.center.getNotificationSettings { settings in
                let status: NotificationAuthorizationStatus
                switch settings.authorizationStatus {
                case .authorized, .provisional:
                    status = .authorized
                case .denied:
                    status = .denied
                case .notDetermined:
                    status = .notDetermined
                @unknown default:
                    status = .notDetermined
                }
                continuation.resume(returning: status)
            }
        }
    }

    public func requestAuthorization() async throws -> Bool {
        try await withCheckedThrowingContinuation { continuation in
            self.center.requestAuthorization(options: [.alert, .badge, .sound]) { granted, error in
                if let error {
                    continuation.resume(throwing: error)
                    return
                }
                continuation.resume(returning: granted)
            }
        }
    }
}

public final class UserNotificationScheduler: @unchecked Sendable, LocalNotificationScheduling {
    private let center: UNUserNotificationCenter

    public init(center: UNUserNotificationCenter = .current()) {
        self.center = center
    }

    public func schedule(_ request: LocalNotificationRequest) async throws {
        let content = UNMutableNotificationContent()
        content.title = request.title
        content.body = request.body
        content.sound = .default

        let notificationRequest = UNNotificationRequest(
            identifier: request.identifier,
            content: content,
            trigger: UNTimeIntervalNotificationTrigger(timeInterval: 1, repeats: false)
        )

        try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<Void, Error>) in
            self.center.add(notificationRequest) { error in
                if let error {
                    continuation.resume(throwing: error)
                    return
                }
                continuation.resume(returning: ())
            }
        }
    }
}

public final class UserDefaultsLocalNotificationStateStore: @unchecked Sendable, LocalNotificationStatePersisting {
    private let defaults: UserDefaults
    private let key: String
    private let encoder = JSONEncoder()
    private let decoder = JSONDecoder()

    public init(
        defaults: UserDefaults = .standard,
        key: String = "heimdall.local-notification-state"
    ) {
        self.defaults = defaults
        self.key = key
    }

    public func loadState() -> PersistedLocalNotificationState {
        guard let data = self.defaults.data(forKey: self.key),
              let state = try? self.decoder.decode(PersistedLocalNotificationState.self, from: data) else {
            return PersistedLocalNotificationState()
        }
        return state
    }

    public func saveState(_ state: PersistedLocalNotificationState) {
        guard let data = try? self.encoder.encode(state) else {
            return
        }
        self.defaults.set(data, forKey: self.key)
    }

    public func clearState() {
        self.defaults.removeObject(forKey: self.key)
    }
}
