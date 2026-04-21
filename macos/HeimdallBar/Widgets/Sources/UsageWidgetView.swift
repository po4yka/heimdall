import HeimdallDomain
import SwiftUI

public struct UsageWidgetView: View {
    let entry: SingleProviderWidgetEntry

    public init(entry: SingleProviderWidgetEntry) {
        self.entry = entry
    }

    public var body: some View {
        switch self.entry.state {
        case .snapshot(let snapshot):
            if let provider = WidgetSelection.providerSnapshot(in: snapshot, provider: self.entry.provider) {
                let model = WidgetRenderModelBuilder.providerModel(from: provider)
                if model.unavailableLabel != nil {
                    WidgetUnavailableCard(model: model)
                        .padding()
                } else {
                    VStack(alignment: .leading, spacing: 10) {
                        WidgetProviderHeaderView(model: model)
                        Text(model.primaryMetric)
                            .font(.title2.monospacedDigit())
                        Text(model.primaryCaption)
                            .font(.caption)
                            .foregroundStyle(WidgetPalette.muted)
                        ForEach(model.usageRows.prefix(3)) { row in
                            UsageBarView(row: row)
                        }
                        HStack {
                            Text(model.todayCostLabel)
                                .font(.caption)
                            Spacer()
                            if let creditsLabel = model.creditsLabel {
                                Text(creditsLabel)
                                    .font(.caption2)
                                    .foregroundStyle(WidgetPalette.muted)
                            }
                        }
                        if let warningLabel = model.warningLabel {
                            Text(warningLabel)
                                .font(.caption2)
                                .foregroundStyle(WidgetPalette.warning)
                        }
                        Text(model.refreshLabel)
                            .font(.caption2)
                            .foregroundStyle(WidgetPalette.muted)
                    }
                    .padding()
                }
            } else {
                WidgetFailureStateView(title: self.entry.provider.title, message: "No widget payload is available for this provider.")
            }
        case .empty:
            WidgetFailureStateView(title: self.entry.provider.title, message: "Waiting for the first widget snapshot.")
        case .failure(let message):
            WidgetFailureStateView(title: self.entry.provider.title, message: message)
        }
    }
}
