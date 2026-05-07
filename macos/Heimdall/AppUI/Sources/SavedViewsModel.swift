import Foundation
import Observation

public struct SavedFilterState: Codable, Equatable, Sendable {
    public var range: DashboardRange?
    public var bucket: DashboardBucket?
    public var provider: ProviderScope?
    public var selectedModels: [String]
    public var projectSearch: String

    public init(
        range: DashboardRange? = nil,
        bucket: DashboardBucket? = nil,
        provider: ProviderScope? = nil,
        selectedModels: [String] = [],
        projectSearch: String = ""
    ) {
        self.range = range
        self.bucket = bucket
        self.provider = provider
        self.selectedModels = selectedModels
        self.projectSearch = projectSearch
    }
}

public struct SavedView: Codable, Identifiable, Equatable, Sendable {
    public let id: UUID
    public var name: String
    public var filters: SavedFilterState
    public let isBuiltIn: Bool

    public init(id: UUID = UUID(), name: String, filters: SavedFilterState, isBuiltIn: Bool = false) {
        self.id = id
        self.name = name
        self.filters = filters
        self.isBuiltIn = isBuiltIn
    }
}

@MainActor
@Observable
public final class SavedViewsModel {
    public private(set) var views: [SavedView]
    public var activeViewID: UUID?

    private static let customKey = "heimdall.savedViews.custom"

    public static let builtInViews: [SavedView] = [
        SavedView(
            id: UUID(uuidString: "00000000-0000-0000-0000-000000000001")!,
            name: "Default",
            filters: SavedFilterState(range: .last30d, bucket: .day, provider: .both),
            isBuiltIn: true
        ),
        SavedView(
            id: UUID(uuidString: "00000000-0000-0000-0000-000000000002")!,
            name: "Compact",
            filters: SavedFilterState(range: .last7d, bucket: .day, provider: .both),
            isBuiltIn: true
        ),
        SavedView(
            id: UUID(uuidString: "00000000-0000-0000-0000-000000000003")!,
            name: "Triage",
            filters: SavedFilterState(range: .last90d, bucket: .week, provider: .both),
            isBuiltIn: true
        ),
    ]

    public init() {
        var all = Self.builtInViews
        if let data = UserDefaults.standard.data(forKey: Self.customKey),
           let custom = try? JSONDecoder().decode([SavedView].self, from: data)
        {
            all += custom
        }
        self.views = all
        self.activeViewID = Self.builtInViews.first?.id
    }

    public func save(name: String, snapshot: SavedFilterState) {
        let view = SavedView(name: name, filters: snapshot)
        views.append(view)
        activeViewID = view.id
        persistCustom()
    }

    public func delete(_ view: SavedView) {
        guard !view.isBuiltIn else { return }
        views.removeAll { $0.id == view.id }
        if activeViewID == view.id {
            activeViewID = Self.builtInViews.first?.id
        }
        persistCustom()
    }

    public func activate(_ view: SavedView, applying filters: DashboardFiltersModel) {
        activeViewID = view.id
        filters.apply(view.filters)
    }

    private func persistCustom() {
        let custom = views.filter { !$0.isBuiltIn }
        if let data = try? JSONEncoder().encode(custom) {
            UserDefaults.standard.set(data, forKey: Self.customKey)
        }
    }
}
