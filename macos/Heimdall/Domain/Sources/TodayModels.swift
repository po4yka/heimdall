import Foundation

public struct TodayHourRow: Codable, Sendable {
    public var hour: Int
    public var turns: Int
    public var inputTokens: Int
    public var outputTokens: Int
    public var cacheReadTokens: Int
    public var cacheCreationTokens: Int
    public var costNanos: Int

    public var costUSD: Double { Double(self.costNanos) / 1_000_000_000 }
    public var totalTokens: Int { self.inputTokens + self.outputTokens + self.cacheReadTokens + self.cacheCreationTokens }

    enum CodingKeys: String, CodingKey {
        case hour, turns
        case inputTokens = "input_tokens"
        case outputTokens = "output_tokens"
        case cacheReadTokens = "cache_read_tokens"
        case cacheCreationTokens = "cache_creation_tokens"
        case costNanos = "cost_nanos"
    }
}

public struct DayHourCell: Codable, Sendable {
    public var day: String
    public var hour: Int
    public var turns: Int
    public var costNanos: Int

    public var costUSD: Double { Double(self.costNanos) / 1_000_000_000 }

    enum CodingKeys: String, CodingKey {
        case day, hour, turns
        case costNanos = "cost_nanos"
    }
}

public struct WeekdayHourCell: Codable, Sendable {
    /// 0 = Sunday … 6 = Saturday (sqlite strftime('%w') convention).
    public var dow: Int
    public var hour: Int
    public var turns: Int
    public var costNanos: Int

    public var costUSD: Double { Double(self.costNanos) / 1_000_000_000 }

    enum CodingKeys: String, CodingKey {
        case dow, hour, turns
        case costNanos = "cost_nanos"
    }
}

public struct TodayTotals: Codable, Sendable {
    public var turns: Int
    public var totalTokens: Int
    public var costNanos: Int
    public var peakHour: Int?
    public var peakHourCostNanos: Int

    public var costUSD: Double { Double(self.costNanos) / 1_000_000_000 }
    public var peakHourCostUSD: Double { Double(self.peakHourCostNanos) / 1_000_000_000 }

    enum CodingKeys: String, CodingKey {
        case turns
        case totalTokens = "total_tokens"
        case costNanos = "cost_nanos"
        case peakHour = "peak_hour"
        case peakHourCostNanos = "peak_hour_cost_nanos"
    }
}

public struct TodayResponse: Codable, Sendable {
    /// Resolved calendar day in the client's local timezone (YYYY-MM-DD).
    public var day: String
    public var tzOffsetMin: Int
    /// Always 24 entries (hours 0-23), zero-filled for empty hours.
    public var hours: [TodayHourRow]
    public var totals: TodayTotals
    /// Last 30 days × 24 hours grid anchored at `day`.
    public var daysHours30: [DayHourCell]
    /// Last 7 days × 24 hours grid anchored at `day`.
    public var daysHours7: [DayHourCell]
    /// 7×24 weekday-hour pattern over the last 90 days.
    public var weekdayHour90: [WeekdayHourCell]

    enum CodingKeys: String, CodingKey {
        case day, hours, totals
        case tzOffsetMin = "tz_offset_min"
        case daysHours30 = "days_hours_30"
        case daysHours7 = "days_hours_7"
        case weekdayHour90 = "weekday_hour_90"
    }
}
