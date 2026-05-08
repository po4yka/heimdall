import Foundation
import Observation

// MARK: - Wire types (mirror Rust SkillsReport JSON)

public struct SkillsReport: Codable, Sendable {
    public let generatedAt: String
    public let tokenizer: String
    public let maxDescChars: Int
    public let budgetFraction: Double
    public let scopes: [SkillScopeResponse]
    public let totals: SkillsTotals
    public let budget: [SkillsBudgetRow]
    public let duplicates: [SkillsDuplicateGroup]

    enum CodingKeys: String, CodingKey {
        case generatedAt = "generated_at"
        case tokenizer
        case maxDescChars = "max_desc_chars"
        case budgetFraction = "budget_fraction"
        case scopes, totals, budget, duplicates
    }
}

public struct SkillsTotals: Codable, Sendable {
    public let skillsCount: Int
    public let totalBytes: UInt64
    public let totalListingTokens: Int
    public let claudeBytes: UInt64
    public let codexBytes: UInt64
    public let projectCount: Int
    public let duplicateCount: Int
    public let duplicateWastedBytes: UInt64

    enum CodingKeys: String, CodingKey {
        case skillsCount = "skills_count"
        case totalBytes = "total_bytes"
        case totalListingTokens = "total_listing_tokens"
        case claudeBytes = "claude_bytes"
        case codexBytes = "codex_bytes"
        case projectCount = "project_count"
        case duplicateCount = "duplicate_count"
        case duplicateWastedBytes = "duplicate_wasted_bytes"
    }
}

public struct SkillsDuplicateOccurrence: Codable, Sendable {
    public let provider: String
    public let scopeKind: String
    public let root: String
    public let projectLabel: String?
    public let bytes: UInt64
    public let listingTokens: Int
    public let frontmatterStatus: String
    public let descriptionExcerpt: String?
    public let isSymlink: Bool

    enum CodingKeys: String, CodingKey {
        case provider
        case scopeKind = "scope_kind"
        case root
        case projectLabel = "project_label"
        case bytes
        case listingTokens = "listing_tokens"
        case frontmatterStatus = "frontmatter_status"
        case descriptionExcerpt = "description_excerpt"
        case isSymlink = "is_symlink"
    }
}

public struct SkillsDuplicateGroup: Codable, Sendable, Identifiable {
    public let name: String
    public let count: Int
    public let wastedBytes: UInt64
    public let wastedTokens: Int
    public let occurrences: [SkillsDuplicateOccurrence]

    public var id: String { name }

    enum CodingKeys: String, CodingKey {
        case name, count
        case wastedBytes = "wasted_bytes"
        case wastedTokens = "wasted_tokens"
        case occurrences
    }

    public var allSameDescription: Bool {
        let first = occurrences.first?.descriptionExcerpt
        return occurrences.allSatisfy { $0.descriptionExcerpt == first }
    }
}

public struct SkillScopeResponse: Codable, Sendable, Identifiable {
    public let provider: String
    public let kind: String
    public let root: String
    public let projectLabel: String?
    public let skills: [SkillRow]
    public let bytes: UInt64
    public let listingTokens: Int

    public var id: String { "\(kind):\(root)" }

    enum CodingKeys: String, CodingKey {
        case provider, kind, root
        case projectLabel = "project_label"
        case skills, bytes
        case listingTokens = "listing_tokens"
    }

    public var displayLabel: String {
        let kindFormatted = kind.replacingOccurrences(of: "_", with: " ")
        if let label = projectLabel {
            return "\(provider) · \(kindFormatted) [\(label)]"
        }
        return "\(provider) · \(kindFormatted)"
    }
}

public struct SkillInvocationStats: Codable, Sendable {
    public let totalCalls: UInt64
    public let lastUsed: String?
    public let distinctSessions: UInt64

    enum CodingKeys: String, CodingKey {
        case totalCalls = "total_calls"
        case lastUsed = "last_used"
        case distinctSessions = "distinct_sessions"
    }
}

public struct SkillRow: Codable, Sendable, Identifiable {
    public let name: String
    public let path: String
    public let description: String?
    public let descriptionChars: Int
    public let descriptionTruncated: Bool
    public let bytes: UInt64
    public let fileCount: Int
    public let listingTokens: Int
    public let frontmatterStatus: String
    public let isSymlink: Bool
    public let symlinkTarget: String?
    public let usage: SkillInvocationStats?
    public let isDormant: Bool

    public var id: String { path }

    enum CodingKeys: String, CodingKey {
        case name, path, description
        case descriptionChars = "description_chars"
        case descriptionTruncated = "description_truncated"
        case bytes
        case fileCount = "file_count"
        case listingTokens = "listing_tokens"
        case frontmatterStatus = "frontmatter_status"
        case isSymlink = "is_symlink"
        case symlinkTarget = "symlink_target"
        case usage
        case isDormant = "is_dormant"
    }
}

public struct SkillsBudgetRow: Codable, Sendable, Identifiable {
    public let modelLabel: String
    public let contextSize: Int
    public let fraction: Double
    public let budgetTokens: Int
    public let usedTokens: Int
    public let headroomTokens: Int
    public let simulatedDropCount: Int
    public let simulatedDropOrder: [String]

    public var id: String { modelLabel }

    enum CodingKeys: String, CodingKey {
        case modelLabel = "model_label"
        case contextSize = "context_size"
        case fraction
        case budgetTokens = "budget_tokens"
        case usedTokens = "used_tokens"
        case headroomTokens = "headroom_tokens"
        case simulatedDropCount = "simulated_drop_count"
        case simulatedDropOrder = "simulated_drop_order"
    }

    public var isOver: Bool { headroomTokens < 0 }
    public var fillFraction: Double {
        guard budgetTokens > 0 else { return 0 }
        return min(1.0, Double(usedTokens) / Double(budgetTokens))
    }
}

// MARK: - Feature model

@MainActor
@Observable
public final class SkillsFeatureModel {
    private let helperPort: Int

    public var report: SkillsReport?
    public var isLoading: Bool = false
    public var errorMessage: String?

    public init(helperPort: Int) {
        self.helperPort = helperPort
    }

    public func reload() async {
        guard !self.isLoading else { return }
        self.isLoading = true
        self.errorMessage = nil
        do {
            guard let url = URL(string: "http://127.0.0.1:\(self.helperPort)/api/skills") else {
                self.isLoading = false
                return
            }
            let (data, response) = try await URLSession.shared.data(from: url)
            guard let http = response as? HTTPURLResponse, (200..<300).contains(http.statusCode) else {
                throw URLError(.badServerResponse)
            }
            self.report = try JSONDecoder().decode(SkillsReport.self, from: data)
        } catch {
            self.errorMessage = error.localizedDescription
        }
        self.isLoading = false
    }
}
