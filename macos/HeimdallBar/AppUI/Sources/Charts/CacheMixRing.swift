import Charts
import SwiftUI

/// Donut-ring variant of the cache efficiency card. Two `SectorMark`s make
/// "cached vs fresh" read as a share rather than a threshold on a rail.
/// The existing linear `CacheEfficiencyCard` is the primary view; this is an
/// alternate that can be dropped in wherever the sector geometry is preferred.
struct CacheMixRing: View {
    let hitRateToday: Double
    var hitRate30d: Double? = nil
    var savings30dUSD: Double? = nil
    var diameter: CGFloat = 108

    struct Entry: Identifiable, Hashable {
        let slice: String
        let fraction: Double
        var id: String { self.slice }
    }

    var body: some View {
        let entries = Self.entries(hitRate: self.hitRateToday)
        HStack(alignment: .center, spacing: 14) {
            Chart(entries) { entry in
                SectorMark(
                    angle: .value("Fraction", entry.fraction),
                    innerRadius: .ratio(0.66),
                    outerRadius: .ratio(0.98)
                )
                .foregroundStyle(by: .value("Slice", entry.slice))
            }
            .chartForegroundStyleScale(
                domain: ["Cached", "Fresh"],
                range: [Self.tintForRate(self.hitRateToday), Color.primary.opacity(0.15)]
            )
            .chartLegend(.hidden)
            .frame(width: self.diameter, height: self.diameter)
            .overlay {
                Text(Self.rateLabel(self.hitRateToday))
                    .font(.title3.monospacedDigit().weight(.semibold))
                    .foregroundStyle(Self.tintForRate(self.hitRateToday))
                    .minimumScaleFactor(0.6)
                    .lineLimit(1)
            }

            VStack(alignment: .leading, spacing: 3) {
                Text("Cache hit rate")
                    .font(.caption.weight(.semibold))
                    .textCase(.uppercase)
                    .tracking(0.4)
                    .foregroundStyle(.secondary)
                    .help("Fraction of input-side tokens served from cache. Ratio is cache reads / (cache reads + cache writes + fresh input).")
                if let thirty = self.hitRate30d {
                    Text("30-day avg: \(Self.rateLabel(thirty))")
                        .font(.caption.monospacedDigit())
                        .foregroundStyle(.secondary)
                }
                if let savings = self.savings30dUSD, savings > 0 {
                    Text("≈ \(Self.currencyLabel(savings)) saved")
                        .font(.caption.weight(.semibold))
                        .foregroundStyle(.green)
                        .help("Estimated dollar savings over the last 30 days from cache reads being billed at the cache-read rate instead of the input rate.")
                }
            }
            .frame(maxWidth: .infinity, alignment: .leading)

            VStack(alignment: .trailing, spacing: 10) {
                Self.metricBlock(
                    label: "Today",
                    value: Self.rateLabel(self.hitRateToday),
                    tint: Self.tintForRate(self.hitRateToday)
                )
                if let thirty = self.hitRate30d {
                    Self.metricBlock(
                        label: "Vs 30d",
                        value: Self.deltaLabel(today: self.hitRateToday, baseline: thirty),
                        tint: Self.deltaTint(today: self.hitRateToday, baseline: thirty)
                    )
                }
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(8)
        .menuCardBackground(
            opacity: ChartStyle.cardBackgroundOpacity,
            cornerRadius: ChartStyle.cardCornerRadius
        )
        .help(Self.tooltip(hitRateToday: self.hitRateToday, hitRate30d: self.hitRate30d, savings30dUSD: self.savings30dUSD))
        .accessibilityElement(children: .combine)
        .accessibilityLabel("Cache hit rate \(Int((max(0, min(1, self.hitRateToday)) * 100).rounded())) percent")
    }

    // MARK: - Data transform

    nonisolated static func entries(hitRate: Double) -> [Entry] {
        let clamped = hitRate.isNaN ? 0 : max(0, min(1, hitRate))
        return [
            Entry(slice: "Cached", fraction: clamped),
            Entry(slice: "Fresh", fraction: 1 - clamped),
        ]
    }

    // MARK: - Formatting helpers (mirrors CacheEfficiencyCard — kept private)

    nonisolated static func rateLabel(_ value: Double) -> String {
        let clamped = max(0, min(1, value))
        if clamped >= 0.999 {
            return "Fully cached"
        }
        return String(format: "%.1f%%", clamped * 100)
    }

    nonisolated static func currencyLabel(_ usd: Double) -> String {
        if usd >= 100 {
            return String(format: "$%.0f", usd)
        }
        if usd >= 10 {
            return String(format: "$%.1f", usd)
        }
        return String(format: "$%.2f", usd)
    }

    /// Red < 30%, orange 30–60%, monochrome primary otherwise.
    /// Mirrors `CacheEfficiencyCard.tint(for:)` exactly.
    nonisolated static func tintForRate(_ rate: Double) -> Color {
        switch rate {
        case ..<0.3: return .red
        case ..<0.6: return .orange
        default: return Color.primary.opacity(0.82)
        }
    }

    @ViewBuilder
    nonisolated static func metricBlock(label: String, value: String, tint: Color) -> some View {
        VStack(alignment: .trailing, spacing: 2) {
            Text(label)
                .font(.system(size: 9, weight: .medium))
                .foregroundStyle(.secondary)
                .textCase(.uppercase)
                .tracking(0.35)
            Text(value)
                .font(.caption.monospacedDigit().weight(.semibold))
                .foregroundStyle(tint)
                .lineLimit(1)
        }
    }

    nonisolated static func deltaLabel(today: Double, baseline: Double) -> String {
        let delta = (today - baseline) * 100
        if abs(delta) < 0.05 {
            return "Flat"
        }
        return String(format: "%@%.1f pt", delta >= 0 ? "+" : "", delta)
    }

    nonisolated static func deltaTint(today: Double, baseline: Double) -> Color {
        let delta = today - baseline
        if abs(delta) < 0.0005 {
            return .secondary
        }
        return delta > 0 ? .green : .orange
    }

    nonisolated static func tooltip(hitRateToday: Double, hitRate30d: Double?, savings30dUSD: Double?) -> String {
        var lines = ["Today: \(Self.rateLabel(hitRateToday))"]
        if let hitRate30d {
            lines.append("30-day avg: \(Self.rateLabel(hitRate30d))")
            lines.append("Delta: \(Self.deltaLabel(today: hitRateToday, baseline: hitRate30d))")
        }
        if let savings30dUSD, savings30dUSD > 0 {
            lines.append("30-day savings: \(Self.currencyLabel(savings30dUSD))")
        }
        return lines.joined(separator: "\n")
    }
}

// MARK: - Previews

#Preview("Healthy — 95%") {
    CacheMixRing(
        hitRateToday: 0.952,
        hitRate30d: 0.91,
        savings30dUSD: 4.37
    )
    .padding()
    .frame(width: 280)
}

#Preview("Middling — 45%") {
    CacheMixRing(
        hitRateToday: 0.45,
        hitRate30d: 0.48,
        savings30dUSD: 1.12
    )
    .padding()
    .frame(width: 280)
}

#Preview("Low — 15%") {
    CacheMixRing(
        hitRateToday: 0.15,
        hitRate30d: 0.22,
        savings30dUSD: 0.31
    )
    .padding()
    .frame(width: 280)
}
