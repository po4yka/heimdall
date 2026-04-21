import AppIntents
import HeimdallDomain

public enum WidgetProviderIntent: String, CaseIterable, AppEnum {
    case claude
    case codex

    public static let typeDisplayRepresentation = TypeDisplayRepresentation(name: "Provider")
    public static let caseDisplayRepresentations: [WidgetProviderIntent: DisplayRepresentation] = [
        .claude: DisplayRepresentation(title: "Claude"),
        .codex: DisplayRepresentation(title: "Codex"),
    ]

    public var providerID: ProviderID {
        switch self {
        case .claude: return .claude
        case .codex: return .codex
        }
    }
}

public struct ProviderSelectionIntent: AppIntent, WidgetConfigurationIntent {
    public static let title: LocalizedStringResource = "Provider"
    public static let description = IntentDescription("Choose the provider shown in the widget.")

    @Parameter(title: "Provider", default: .claude)
    public var provider: WidgetProviderIntent

    public init() {}
}
