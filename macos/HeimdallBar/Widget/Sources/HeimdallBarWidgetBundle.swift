import HeimdallBarShared
import SwiftUI
import WidgetKit

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
        AppIntentConfiguration(
            kind: "HeimdallBarUsageWidget",
            intent: ProviderSelectionIntent.self,
            provider: SingleProviderTimelineProvider()
        ) { entry in
            UsageWidgetView(entry: entry)
        }
        .configurationDisplayName("HeimdallBar Usage")
        .description("Live quota, auth state, and credits for one provider.")
        .supportedFamilies([.systemSmall, .systemMedium])
    }
}

struct HeimdallBarHistoryWidget: Widget {
    var body: some WidgetConfiguration {
        AppIntentConfiguration(
            kind: "HeimdallBarHistoryWidget",
            intent: ProviderSelectionIntent.self,
            provider: SingleProviderTimelineProvider()
        ) { entry in
            HistoryWidgetView(entry: entry)
        }
        .configurationDisplayName("HeimdallBar History")
        .description("Recent cost and activity for one provider.")
        .supportedFamilies([.systemMedium])
    }
}

struct HeimdallBarCompactWidget: Widget {
    var body: some WidgetConfiguration {
        AppIntentConfiguration(
            kind: "HeimdallBarCompactWidget",
            intent: ProviderSelectionIntent.self,
            provider: SingleProviderTimelineProvider()
        ) { entry in
            CompactWidgetView(entry: entry)
        }
        .configurationDisplayName("HeimdallBar Compact")
        .description("A compact auth and quota summary for one provider.")
        .supportedFamilies([.systemSmall])
    }
}

struct HeimdallBarSwitcherWidget: Widget {
    var body: some WidgetConfiguration {
        StaticConfiguration(kind: "HeimdallBarSwitcherWidget", provider: SwitcherTimelineProvider()) { entry in
            SwitcherWidgetView(entry: entry)
        }
        .configurationDisplayName("HeimdallBar Switcher")
        .description("A dual-provider status surface sorted by severity.")
        .supportedFamilies([.systemMedium])
    }
}
