import AppIntents
import HeimdallBarShared
import SwiftUI
import WidgetKit

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

struct HeimdallBarWidgetEntry: TimelineEntry {
    let date: Date
    let provider: ProviderID
    let snapshot: WidgetSnapshot
}

struct HeimdallBarTimelineProvider: AppIntentTimelineProvider {
    func placeholder(in context: Context) -> HeimdallBarWidgetEntry {
        HeimdallBarWidgetEntry(date: Date(), provider: .claude, snapshot: Self.emptySnapshot())
    }

    func snapshot(for configuration: ProviderSelectionIntent, in context: Context) async -> HeimdallBarWidgetEntry {
        HeimdallBarWidgetEntry(
            date: Date(),
            provider: configuration.provider.providerID,
            snapshot: WidgetSnapshotStore.load() ?? Self.emptySnapshot()
        )
    }

    func timeline(for configuration: ProviderSelectionIntent, in context: Context) async -> Timeline<HeimdallBarWidgetEntry> {
        let snapshot = WidgetSnapshotStore.load() ?? Self.emptySnapshot()
        let entry = HeimdallBarWidgetEntry(date: Date(), provider: configuration.provider.providerID, snapshot: snapshot)
        let cadence = WidgetSelection.cadenceSeconds(snapshot: snapshot, provider: configuration.provider.providerID)
        return Timeline(entries: [entry], policy: .after(.now.addingTimeInterval(cadence)))
    }

    private static func emptySnapshot() -> WidgetSnapshot {
        WidgetSnapshot(generatedAt: ISO8601DateFormatter().string(from: Date()), refreshIntervalSeconds: 900, entries: [])
    }
}

private struct WidgetPalette {
    static let panel = Color.primary.opacity(0.08)
    static let muted = Color.primary.opacity(0.55)
    static let barTrack = Color.primary.opacity(0.14)
    static let barFill = Color.primary
}

private struct UsageBar: View {
    let line: WidgetUsageLine

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack {
                Text(self.line.title)
                    .font(.caption)
                Spacer()
                Text(self.line.valueLabel)
                    .font(.caption.monospacedDigit())
            }
            GeometryReader { proxy in
                ZStack(alignment: .leading) {
                    Capsule()
                        .fill(WidgetPalette.barTrack)
                    Capsule()
                        .fill(WidgetPalette.barFill)
                        .frame(width: max(4, proxy.size.width * CGFloat(self.line.fraction ?? 0)))
                }
            }
            .frame(height: 5)
            if let detailLabel = self.line.detailLabel {
                Text(detailLabel)
                    .font(.caption2)
                    .foregroundStyle(WidgetPalette.muted)
            }
        }
    }
}

private struct WidgetStatusChip: View {
    let entry: WidgetProviderEntry

    var body: some View {
        Text(self.entry.statusLabel.uppercased())
            .font(.caption2.monospaced())
            .padding(.horizontal, 6)
            .padding(.vertical, 3)
            .background(WidgetPalette.panel)
            .clipShape(Capsule())
    }
}

private struct WidgetProviderHeader: View {
    let entry: WidgetProviderEntry

    var body: some View {
        HStack(alignment: .top) {
            VStack(alignment: .leading, spacing: 2) {
                Text(self.entry.title)
                    .font(.headline)
                Text(self.entry.sourceLabel)
                    .font(.caption2)
                    .foregroundStyle(WidgetPalette.muted)
            }
            Spacer()
            WidgetStatusChip(entry: self.entry)
        }
    }
}

private struct WidgetFallbackView: View {
    let entry: WidgetProviderEntry

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            WidgetProviderHeader(entry: self.entry)
            if let unavailableLabel = self.entry.unavailableLabel {
                Text(unavailableLabel)
                    .font(.caption)
            }
            if let warningLabel = self.entry.warningLabel {
                Text(warningLabel)
                    .font(.caption2)
                    .foregroundStyle(WidgetPalette.muted)
            }
            Text(self.entry.refreshLabel)
                .font(.caption2)
                .foregroundStyle(WidgetPalette.muted)
        }
    }
}

struct UsageWidgetView: View {
    let entry: HeimdallBarWidgetEntry

    var body: some View {
        if let provider = WidgetSelection.providerEntry(in: self.entry.snapshot, provider: self.entry.provider) {
            if provider.unavailableLabel != nil {
                WidgetFallbackView(entry: provider)
                    .padding()
            } else {
                VStack(alignment: .leading, spacing: 10) {
                    WidgetProviderHeader(entry: provider)
                    ForEach(provider.usageLines.prefix(3)) { line in
                        UsageBar(line: line)
                    }
                    HStack {
                        Text(provider.todayCostLabel)
                            .font(.caption)
                        Spacer()
                        if let creditsLabel = provider.creditsLabel {
                            Text(creditsLabel)
                                .font(.caption2)
                                .foregroundStyle(WidgetPalette.muted)
                        }
                    }
                    Text(provider.refreshLabel)
                        .font(.caption2)
                        .foregroundStyle(WidgetPalette.muted)
                }
                .padding()
            }
        } else {
            Text("No snapshot")
                .padding()
        }
    }
}

struct HistoryWidgetView: View {
    let entry: HeimdallBarWidgetEntry

    var body: some View {
        if let provider = WidgetSelection.providerEntry(in: self.entry.snapshot, provider: self.entry.provider) {
            VStack(alignment: .leading, spacing: 10) {
                WidgetProviderHeader(entry: provider)
                HStack(alignment: .bottom, spacing: 4) {
                    ForEach(Array(provider.historyFractions.enumerated()), id: \.offset) { item in
                        RoundedRectangle(cornerRadius: 1.5)
                            .fill(WidgetPalette.barTrack)
                            .overlay(alignment: .bottom) {
                                RoundedRectangle(cornerRadius: 1.5)
                                    .fill(WidgetPalette.barFill)
                                    .frame(height: max(3, CGFloat(item.element) * 28))
                            }
                            .frame(maxWidth: .infinity, minHeight: 28, maxHeight: 28)
                    }
                }
                HStack {
                    VStack(alignment: .leading, spacing: 2) {
                        Text(provider.todayCostLabel)
                        Text(provider.todayTokensLabel)
                            .font(.caption2)
                            .foregroundStyle(WidgetPalette.muted)
                    }
                    Spacer()
                    VStack(alignment: .trailing, spacing: 2) {
                        Text(provider.last30DaysCostLabel)
                        Text(provider.activityLabel)
                            .font(.caption2)
                            .foregroundStyle(WidgetPalette.muted)
                    }
                }
                .font(.caption)
            }
            .padding()
        } else {
            Text("No history")
                .padding()
        }
    }
}

struct CompactWidgetView: View {
    let entry: HeimdallBarWidgetEntry

    var body: some View {
        if let provider = WidgetSelection.providerEntry(in: self.entry.snapshot, provider: self.entry.provider) {
            VStack(alignment: .leading, spacing: 6) {
                HStack {
                    Text(provider.title)
                    Spacer()
                    WidgetStatusChip(entry: provider)
                }
                .font(.caption)
                Text(provider.usageLines.first?.valueLabel ?? provider.creditsLabel ?? "—")
                    .font(.title2.monospacedDigit())
                Text(provider.usageLines.first?.detailLabel ?? provider.warningLabel ?? provider.sourceLabel)
                    .font(.caption2)
                    .foregroundStyle(WidgetPalette.muted)
            }
            .padding()
        } else {
            Text("—")
        }
    }
}

struct SwitcherWidgetView: View {
    let entry: HeimdallBarWidgetEntry

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Providers")
                .font(.headline)
            HStack(spacing: 8) {
                ForEach(self.entry.snapshot.entries.prefix(2)) { provider in
                    VStack(alignment: .leading, spacing: 6) {
                        HStack {
                            Text(provider.title)
                                .font(.caption)
                            Spacer()
                            WidgetStatusChip(entry: provider)
                        }
                        Text(provider.usageLines.first?.valueLabel ?? provider.creditsLabel ?? "—")
                            .font(.title3.monospacedDigit())
                        Text(provider.usageLines.first?.title ?? provider.sourceLabel)
                            .font(.caption2)
                            .foregroundStyle(WidgetPalette.muted)
                    }
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .padding(8)
                    .background(WidgetPalette.panel)
                    .clipShape(RoundedRectangle(cornerRadius: 10))
                }
            }
        }
        .padding()
    }
}

@main
struct HeimdallBarWidgetBundle: WidgetBundle {
    var body: some Widget {
        HeimdallBarUsageWidget()
        HeimdallBarHistoryWidget()
        HeimdallBarCompactWidget()
        HeimdallBarSwitcherWidget()
    }
}

struct HeimdallBarUsageWidget: Widget {
    var body: some WidgetConfiguration {
        AppIntentConfiguration(kind: "HeimdallBarUsageWidget", intent: ProviderSelectionIntent.self, provider: HeimdallBarTimelineProvider()) { entry in
            UsageWidgetView(entry: entry)
        }
        .configurationDisplayName("HeimdallBar Usage")
        .description("Live session, weekly usage, and credits.")
        .supportedFamilies([.systemSmall, .systemMedium])
    }
}

struct HeimdallBarHistoryWidget: Widget {
    var body: some WidgetConfiguration {
        AppIntentConfiguration(kind: "HeimdallBarHistoryWidget", intent: ProviderSelectionIntent.self, provider: HeimdallBarTimelineProvider()) { entry in
            HistoryWidgetView(entry: entry)
        }
        .configurationDisplayName("HeimdallBar History")
        .description("Recent activity and cost history.")
        .supportedFamilies([.systemMedium])
    }
}

struct HeimdallBarCompactWidget: Widget {
    var body: some WidgetConfiguration {
        AppIntentConfiguration(kind: "HeimdallBarCompactWidget", intent: ProviderSelectionIntent.self, provider: HeimdallBarTimelineProvider()) { entry in
            CompactWidgetView(entry: entry)
        }
        .configurationDisplayName("HeimdallBar Compact")
        .description("Compact lane or credit state.")
        .supportedFamilies([.systemSmall])
    }
}

struct HeimdallBarSwitcherWidget: Widget {
    var body: some WidgetConfiguration {
        AppIntentConfiguration(kind: "HeimdallBarSwitcherWidget", intent: ProviderSelectionIntent.self, provider: HeimdallBarTimelineProvider()) { entry in
            SwitcherWidgetView(entry: entry)
        }
        .configurationDisplayName("HeimdallBar Switcher")
        .description("A paired Claude and Codex status surface.")
        .supportedFamilies([.systemMedium])
    }
}
