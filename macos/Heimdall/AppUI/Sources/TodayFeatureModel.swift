import Foundation
import HeimdallDomain
import Observation

@MainActor
@Observable
public final class TodayFeatureModel {
    private let helperPort: Int

    public var response: TodayResponse?
    public var isLoading: Bool = false
    public var errorMessage: String?
    /// nil = today (server resolves); a YYYY-MM-DD string pins a specific date.
    public var pinnedDate: String?

    public init(helperPort: Int) {
        self.helperPort = helperPort
    }

    public func load() async {
        guard !self.isLoading else { return }
        self.isLoading = true
        self.errorMessage = nil
        do {
            let tzOffset = TimeZone.current.secondsFromGMT() / 60
            var query = "tz_offset_min=\(tzOffset)"
            if let date = self.pinnedDate {
                query += "&date=\(date)"
            }
            guard let url = URL(string: "http://127.0.0.1:\(self.helperPort)/api/today?\(query)") else {
                self.isLoading = false
                return
            }
            let (data, response) = try await URLSession.shared.data(from: url)
            guard let http = response as? HTTPURLResponse, (200..<300).contains(http.statusCode) else {
                throw URLError(.badServerResponse)
            }
            self.response = try JSONDecoder().decode(TodayResponse.self, from: data)
        } catch {
            self.errorMessage = error.localizedDescription
        }
        self.isLoading = false
    }

    public func selectDate(_ date: String?) {
        self.pinnedDate = date
        Task { await self.load() }
    }

    public func pinToday() {
        self.pinnedDate = nil
        Task { await self.load() }
    }
}
