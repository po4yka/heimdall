import HeimdallDomain
import HeimdallServices
import SwiftUI
import WidgetKit

public enum WidgetContentState {
    case snapshot(WidgetSnapshot)
    case empty
    case failure(String)
}

public struct SingleProviderWidgetEntry: TimelineEntry {
    public let date: Date
    public let provider: ProviderID
    public let state: WidgetContentState

    public init(date: Date, provider: ProviderID, state: WidgetContentState) {
        self.date = date
        self.provider = provider
        self.state = state
    }
}

public struct SwitcherWidgetEntry: TimelineEntry {
    public let date: Date
    public let state: WidgetContentState

    public init(date: Date, state: WidgetContentState) {
        self.date = date
        self.state = state
    }
}

public struct SingleProviderTimelineProvider: AppIntentTimelineProvider {
    public init() {}

    public func placeholder(in context: Context) -> SingleProviderWidgetEntry {
        SingleProviderWidgetEntry(date: Date(), provider: .claude, state: .empty)
    }

    public func snapshot(for configuration: ProviderSelectionIntent, in context: Context) async -> SingleProviderWidgetEntry {
        SingleProviderWidgetEntry(
            date: Date(),
            provider: configuration.provider.providerID,
            state: self.loadState()
        )
    }

    public func timeline(for configuration: ProviderSelectionIntent, in context: Context) async -> Timeline<SingleProviderWidgetEntry> {
        let state = self.loadState()
        let entry = SingleProviderWidgetEntry(
            date: Date(),
            provider: configuration.provider.providerID,
            state: state
        )
        return Timeline(
            entries: [entry],
            policy: .after(.now.addingTimeInterval(self.cadence(for: state, provider: configuration.provider.providerID)))
        )
    }

    private func loadState() -> WidgetContentState {
        switch WidgetSnapshotStore.load() {
        case .success(let snapshot):
            return .snapshot(snapshot)
        case .empty:
            return .empty
        case .failure(let error):
            return .failure(error.localizedDescription)
        }
    }

    private func cadence(for state: WidgetContentState, provider: ProviderID) -> TimeInterval {
        switch state {
        case .snapshot(let snapshot):
            return WidgetSelection.cadenceSeconds(snapshot: snapshot, provider: provider)
        case .empty, .failure:
            return 300
        }
    }
}

public struct SwitcherTimelineProvider: TimelineProvider {
    public init() {}

    public func placeholder(in context: Context) -> SwitcherWidgetEntry {
        SwitcherWidgetEntry(date: Date(), state: .empty)
    }

    public func getSnapshot(in context: Context, completion: @escaping (SwitcherWidgetEntry) -> Void) {
        completion(SwitcherWidgetEntry(date: Date(), state: self.loadState()))
    }

    public func getTimeline(in context: Context, completion: @escaping (Timeline<SwitcherWidgetEntry>) -> Void) {
        let state = self.loadState()
        let entry = SwitcherWidgetEntry(date: Date(), state: state)
        let cadence: TimeInterval
        switch state {
        case .snapshot(let snapshot):
            cadence = WidgetSelection.cadenceSeconds(snapshot: snapshot, provider: nil)
        case .empty, .failure:
            cadence = 300
        }
        completion(Timeline(entries: [entry], policy: .after(.now.addingTimeInterval(cadence))))
    }

    private func loadState() -> WidgetContentState {
        switch WidgetSnapshotStore.load() {
        case .success(let snapshot):
            return .snapshot(snapshot)
        case .empty:
            return .empty
        case .failure(let error):
            return .failure(error.localizedDescription)
        }
    }
}
