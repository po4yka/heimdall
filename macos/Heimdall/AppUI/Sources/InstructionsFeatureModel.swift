import Foundation
import Observation

// MARK: - Wire types (mirror Rust InstructionFilesReport JSON)

public enum InstructionFrontmatterStatus: String, Codable, Sendable {
    case ok
    case invalid
    case notApplicable = "not_applicable"
}

public struct InstructionFile: Codable, Sendable, Identifiable {
    public let path: String
    public let bytes: UInt64
    public let lineCount: Int
    public let tokens: Int
    public let modified: String
    public let frontmatterStatus: InstructionFrontmatterStatus
    public let isSymlink: Bool

    public var id: String { path }

    enum CodingKeys: String, CodingKey {
        case path, bytes
        case lineCount = "line_count"
        case tokens, modified
        case frontmatterStatus = "frontmatter_status"
        case isSymlink = "is_symlink"
    }
}

public struct InstructionScope: Codable, Sendable, Identifiable {
    public let provider: String
    public let kind: String
    public let root: String
    public let projectLabel: String?
    public let files: [InstructionFile]
    public let bytes: UInt64
    public let tokens: Int

    public var id: String { "\(kind):\(root)" }

    enum CodingKeys: String, CodingKey {
        case provider, kind, root
        case projectLabel = "project_label"
        case files, bytes, tokens
    }

    public var displayLabel: String {
        let kindFormatted = kind.replacingOccurrences(of: "_", with: " ")
        if let label = projectLabel {
            return "\(provider) · \(kindFormatted) [\(label)]"
        }
        return "\(provider) · \(kindFormatted)"
    }
}

public struct InstructionTotals: Codable, Sendable {
    public let fileCount: Int
    public let totalBytes: UInt64
    public let totalTokens: Int
    public let claudeBytes: UInt64
    public let codexBytes: UInt64
    public let projectCount: Int
    public let nestedCount: Int

    enum CodingKeys: String, CodingKey {
        case fileCount = "file_count"
        case totalBytes = "total_bytes"
        case totalTokens = "total_tokens"
        case claudeBytes = "claude_bytes"
        case codexBytes = "codex_bytes"
        case projectCount = "project_count"
        case nestedCount = "nested_count"
    }
}

public struct InstructionFilesReport: Codable, Sendable {
    public let generatedAt: String
    public let tokenizer: String
    public let budgetFraction: Double
    public let scopes: [InstructionScope]
    public let totals: InstructionTotals
    public let budget: [SkillsBudgetRow]

    enum CodingKeys: String, CodingKey {
        case generatedAt = "generated_at"
        case tokenizer
        case budgetFraction = "budget_fraction"
        case scopes, totals, budget
    }
}

// MARK: - Feature model

@MainActor
@Observable
public final class InstructionsFeatureModel {
    private let helperPort: Int

    public var report: InstructionFilesReport?
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
            guard let url = URL(string: "http://127.0.0.1:\(self.helperPort)/api/instruction-files") else {
                self.isLoading = false
                return
            }
            let (data, response) = try await URLSession.shared.data(from: url)
            guard let http = response as? HTTPURLResponse, (200..<300).contains(http.statusCode) else {
                throw URLError(.badServerResponse)
            }
            self.report = try JSONDecoder().decode(InstructionFilesReport.self, from: data)
        } catch {
            self.errorMessage = error.localizedDescription
        }
        self.isLoading = false
    }
}
