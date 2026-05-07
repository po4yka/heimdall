import Foundation
import HeimdallDomain
import Observation

public enum PaletteCommandGroup: String, CaseIterable {
    case navigate = "Navigate"
    case action = "Actions"
    case session = "Sessions"
    case project = "Projects"
    case model = "Models"
}

public struct PaletteCommand: Identifiable {
    public let id: String
    public let group: PaletteCommandGroup
    public let label: String
    public let hint: String?
    let searchTerms: String
    @MainActor public let run: () -> Void

    @MainActor
    public init(
        id: String,
        group: PaletteCommandGroup,
        label: String,
        hint: String? = nil,
        searchTerms: String? = nil,
        run: @escaping @MainActor () -> Void
    ) {
        self.id = id
        self.group = group
        self.label = label
        self.hint = hint
        self.searchTerms = (searchTerms ?? label).lowercased()
        self.run = run
    }
}

@MainActor
@Observable
public final class CommandPaletteModel {
    public var query: String = ""
    public var selectedCommandID: String?

    weak var shell: AppShellModel?
    var sessionCommands: [PaletteCommand] = []
    var projectCommands: [PaletteCommand] = []
    var modelCommands: [PaletteCommand] = []

    public init() {}

    public func prepare(
        shell: AppShellModel,
        sessions: [ProviderSession],
        projects: [ProviderProjectRow],
        models: [ProviderModelRow],
        onOpenSettings: @escaping @MainActor () -> Void,
        onOpenBackup: @escaping @MainActor () -> Void
    ) {
        self.shell = shell
        self.query = ""
        self.selectedCommandID = nil

        self.sessionCommands = sessions.prefix(30).map { s in
            PaletteCommand(
                id: "session:\(s.sessionID)",
                group: .session,
                label: s.displayName,
                hint: "\(s.turns) turns · \(s.model ?? "—")",
                searchTerms: "\(s.displayName) \(s.model ?? "") \(s.sessionID)",
                run: { shell.navigationSelection = .sessions }
            )
        }

        self.projectCommands = projects.prefix(30).map { p in
            PaletteCommand(
                id: "project:\(p.project)",
                group: .project,
                label: p.displayName,
                hint: "\(p.sessions) sessions · \(FormatHelpers.formatUSD(p.costUSD))",
                searchTerms: "\(p.project) \(p.displayName)",
                run: { shell.navigationSelection = .projects }
            )
        }

        self.modelCommands = models.map { m in
            PaletteCommand(
                id: "model:\(m.model)",
                group: .model,
                label: m.model,
                hint: FormatHelpers.formatUSD(m.costUSD),
                searchTerms: m.model,
                run: { shell.navigationSelection = .costModels }
            )
        }
    }

    @MainActor
    public var navigationCommands: [PaletteCommand] {
        guard let shell else { return [] }
        return [
            PaletteCommand(id: "nav:overview", group: .navigate, label: "Go to Overview",
                           searchTerms: "overview navigate go", run: { shell.navigationSelection = .overview }),
            PaletteCommand(id: "nav:today", group: .navigate, label: "Go to Today",
                           searchTerms: "today navigate go", run: { shell.navigationSelection = .today }),
            PaletteCommand(id: "nav:activity", group: .navigate, label: "Go to Activity",
                           searchTerms: "activity trends navigate go", run: { shell.navigationSelection = .activity }),
            PaletteCommand(id: "nav:agents", group: .navigate, label: "Go to Agents",
                           searchTerms: "agents subagent navigate go", run: { shell.navigationSelection = .agents }),
            PaletteCommand(id: "nav:cost-models", group: .navigate, label: "Go to Cost & Models",
                           searchTerms: "cost models breakdowns navigate go", run: { shell.navigationSelection = .costModels }),
            PaletteCommand(id: "nav:sessions", group: .navigate, label: "Go to Sessions",
                           searchTerms: "sessions tables navigate go", run: { shell.navigationSelection = .sessions }),
            PaletteCommand(id: "nav:projects", group: .navigate, label: "Go to Projects",
                           searchTerms: "projects navigate go", run: { shell.navigationSelection = .projects }),
            PaletteCommand(id: "nav:live", group: .navigate, label: "Go to Live Monitor",
                           searchTerms: "live monitor real-time navigate go", run: { shell.navigationSelection = .liveMonitor }),
        ]
    }

    public var allCommands: [PaletteCommand] {
        self.navigationCommands + self.sessionCommands + self.projectCommands + self.modelCommands
    }

    public var filteredCommands: [PaletteCommand] {
        let q = self.query.trimmingCharacters(in: .whitespaces).lowercased()
        if q.isEmpty { return self.allCommands }
        let tokens = q.split(separator: " ").map(String.init)
        return self.allCommands.filter { cmd in
            let hay = "\(cmd.label) \(cmd.searchTerms)".lowercased()
            return tokens.allSatisfy { hay.contains($0) }
        }
    }

    public var groupedCommands: [(group: PaletteCommandGroup, commands: [PaletteCommand])] {
        let cmds = self.filteredCommands
        let grouped = Dictionary(grouping: cmds, by: \.group)
        return PaletteCommandGroup.allCases.compactMap { group in
            guard let items = grouped[group], !items.isEmpty else { return nil }
            return (group: group, commands: items)
        }
    }
}
