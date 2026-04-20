import HeimdallBarShared
import SwiftUI

struct CompactWidgetView: View {
    let entry: SingleProviderWidgetEntry

    var body: some View {
        switch self.entry.state {
        case .snapshot(let snapshot):
            let model = WidgetSelection.providerSnapshot(in: snapshot, provider: self.entry.provider)
                .map(WidgetRenderModelBuilder.providerModel(from:))
                ?? WidgetRenderModelBuilder.emptyStateModel(provider: self.entry.provider, message: "No widget payload is available.")
            VStack(alignment: .leading, spacing: 6) {
                HStack {
                    Text(model.title)
                    Spacer()
                    WidgetStatusChip(label: model.statusLabel)
                }
                .font(.caption)
                Text(model.primaryMetric)
                    .font(.title2.monospacedDigit())
                Text(model.warningLabel ?? model.authLabel ?? model.primaryCaption)
                    .font(.caption2)
                    .foregroundStyle(WidgetPalette.muted)
            }
            .padding()
        case .empty:
            WidgetFailureStateView(title: self.entry.provider.title, message: "Waiting for the first widget snapshot.")
        case .failure(let message):
            WidgetFailureStateView(title: self.entry.provider.title, message: message)
        }
    }
}
