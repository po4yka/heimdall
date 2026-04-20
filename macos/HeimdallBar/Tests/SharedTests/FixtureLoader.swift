import Foundation

enum FixtureLoader {
    static func string(_ relativePath: String) throws -> String {
        let url = self.url(relativePath)
        return try String(contentsOf: url, encoding: .utf8)
    }

    static func data(_ relativePath: String) throws -> Data {
        try Data(contentsOf: self.url(relativePath))
    }

    private static func url(_ relativePath: String) -> URL {
        let root = URL(fileURLWithPath: #filePath)
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .deletingLastPathComponent()
        return root.appendingPathComponent(relativePath)
    }
}
