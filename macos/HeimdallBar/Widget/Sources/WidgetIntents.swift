import AppIntents
import HeimdallBarShared

enum WidgetProviderIntent: String, CaseIterable, AppEnum {
    case claude
    case codex

    static let typeDisplayRepresentation = TypeDisplayRepresentation(name: "Provider")
    static let caseDisplayRepresentations: [WidgetProviderIntent: DisplayRepresentation] = [
        .claude: DisplayRepresentation(title: "Claude"),
        .codex: DisplayRepresentation(title: "Codex"),
    ]

    var providerID: ProviderID {
        switch self {
        case .claude: return .claude
        case .codex: return .codex
        }
    }
}

struct ProviderSelectionIntent: AppIntent, WidgetConfigurationIntent {
    static let title: LocalizedStringResource = "Provider"
    static let description = IntentDescription("Choose the provider shown in the widget.")

    @Parameter(title: "Provider", default: .claude)
    var provider: WidgetProviderIntent
}
