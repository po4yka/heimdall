import HeimdallBarShared
import SwiftUI

struct SwitcherWidgetView: View {
    let entry: SwitcherWidgetEntry

    var body: some View {
        switch self.entry.state {
        case .snapshot(let snapshot):
            let models = WidgetRenderModelBuilder.switcherModels(from: snapshot)
            VStack(alignment: .leading, spacing: 8) {
                Text("Providers")
                    .font(.headline)
                HStack(spacing: 8) {
                    ForEach(models.prefix(2)) { model in
                        VStack(alignment: .leading, spacing: 6) {
                            HStack {
                                Text(model.title)
                                    .font(.caption)
                                Spacer()
                                WidgetStatusChip(label: model.statusLabel)
                            }
                            Text(model.primaryMetric)
                                .font(.title3.monospacedDigit())
                            Text(model.warningLabel ?? model.primaryCaption)
                                .font(.caption2)
                                .foregroundStyle(
                                    model.warningLabel == nil
                                        ? WidgetPalette.muted
                                        : WidgetPalette.warning
                                )
                        }
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .padding(8)
                        .background(WidgetPalette.panel)
                        .clipShape(RoundedRectangle(cornerRadius: 10))
                    }
                }
            }
            .padding()
        case .empty:
            WidgetFailureStateView(title: "Providers", message: "Waiting for the first dual-provider snapshot.")
        case .failure(let message):
            WidgetFailureStateView(title: "Providers", message: message)
        }
    }
}
