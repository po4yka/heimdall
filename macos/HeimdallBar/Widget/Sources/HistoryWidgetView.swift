import HeimdallBarShared
import SwiftUI

struct HistoryWidgetView: View {
    let entry: SingleProviderWidgetEntry

    var body: some View {
        switch self.entry.state {
        case .snapshot(let snapshot):
            if let provider = WidgetSelection.providerSnapshot(in: snapshot, provider: self.entry.provider) {
                let model = WidgetRenderModelBuilder.providerModel(from: provider)
                VStack(alignment: .leading, spacing: 10) {
                    WidgetProviderHeaderView(model: model)
                    if model.historyFractions.isEmpty {
                        WidgetUnavailableCard(model: model)
                    } else {
                        HStack(alignment: .bottom, spacing: 4) {
                            ForEach(Array(model.historyFractions.enumerated()), id: \.offset) { item in
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
                                Text(model.todayCostLabel)
                                Text(model.todayTokensLabel)
                                    .font(.caption2)
                                    .foregroundStyle(WidgetPalette.muted)
                            }
                            Spacer()
                            VStack(alignment: .trailing, spacing: 2) {
                                Text(model.last30DaysCostLabel)
                                Text(model.activityLabel)
                                    .font(.caption2)
                                    .foregroundStyle(WidgetPalette.muted)
                            }
                        }
                        .font(.caption)
                    }
                }
                .padding()
            } else {
                WidgetFailureStateView(title: self.entry.provider.title, message: "No history payload is available for this provider.")
            }
        case .empty:
            WidgetFailureStateView(title: self.entry.provider.title, message: "Waiting for the first widget snapshot.")
        case .failure(let message):
            WidgetFailureStateView(title: self.entry.provider.title, message: message)
        }
    }
}
