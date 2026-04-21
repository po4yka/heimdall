import HeimdallDomain

public final class WidgetSnapshotCoordinator: Sendable {
    private let writer: any WidgetSnapshotWriter
    private let reloader: any WidgetReloading

    public init(
        writer: any WidgetSnapshotWriter,
        reloader: any WidgetReloading
    ) {
        self.writer = writer
        self.reloader = reloader
    }

    @discardableResult
    public func persist(_ snapshot: WidgetSnapshot) throws -> WidgetSnapshotSaveResult {
        let result = try self.writer.save(snapshot)
        if result == .saved {
            self.reloader.reloadAllTimelines()
        }
        return result
    }

    public func load() -> WidgetSnapshotLoadResult {
        self.writer.load()
    }
}
