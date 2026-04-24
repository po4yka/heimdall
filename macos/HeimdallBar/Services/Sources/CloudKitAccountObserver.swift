import CloudKit
import Foundation
import os

private let cloudKitAccountObserverLogger = Logger(subsystem: "dev.heimdall.heimdallbar", category: "CloudKit")

public final class CloudKitAccountObserver: @unchecked Sendable {
    private let notificationCenter: NotificationCenter
    private let handler: @Sendable () -> Void
    private var observerToken: NSObjectProtocol?

    public init(
        notificationCenter: NotificationCenter = .default,
        handler: @escaping @Sendable () -> Void
    ) {
        self.notificationCenter = notificationCenter
        self.handler = handler
    }

    public func start() {
        guard self.observerToken == nil else { return }
        let handler = self.handler
        self.observerToken = self.notificationCenter.addObserver(
            forName: .CKAccountChanged,
            object: nil,
            queue: nil
        ) { _ in
            cloudKitAccountObserverLogger.info("CKAccountChanged fired")
            handler()
        }
    }

    public func stop() {
        if let observerToken {
            self.notificationCenter.removeObserver(observerToken)
            self.observerToken = nil
        }
    }

    deinit {
        self.stop()
    }
}
