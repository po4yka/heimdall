import Foundation
import Testing

struct VersionAlignmentTests {
    @Test
    func cargoVersionMatchesHeimdallBarMarketingVersion() throws {
        let cargo = try FixtureLoader.string("Cargo.toml")
        let project = try FixtureLoader.string("macos/HeimdallBar/project.yml")
        let pbxproj = try FixtureLoader.string("macos/HeimdallBar/HeimdallBar.xcodeproj/project.pbxproj")
        let appInfo = try FixtureLoader.string("macos/HeimdallBar/App/Info.plist")
        let widgetInfo = try FixtureLoader.string("macos/HeimdallBar/Widget/Info.plist")
        let appEntitlements = try FixtureLoader.string("macos/HeimdallBar/App/HeimdallBar.entitlements")
        let widgetEntitlements = try FixtureLoader.string("macos/HeimdallBar/Widget/HeimdallBarWidget.entitlements")

        let cargoVersion = try #require(Self.firstMatch(in: cargo, pattern: #"(?m)^version\s*=\s*"([^"]+)""#))
        let marketingVersion = try #require(Self.firstMatch(in: project, pattern: #"(?m)^\s*MARKETING_VERSION:\s*([^\s]+)\s*$"#))

        #expect(marketingVersion == cargoVersion)
        #expect(pbxproj.contains("MARKETING_VERSION = \(cargoVersion);"))
        #expect(appInfo.contains("$(MARKETING_VERSION)"))
        #expect(appInfo.contains("$(CURRENT_PROJECT_VERSION)"))
        #expect(widgetInfo.contains("$(MARKETING_VERSION)"))
        #expect(widgetInfo.contains("$(CURRENT_PROJECT_VERSION)"))
        #expect(appEntitlements.contains("group.dev.heimdall.heimdallbar"))
        #expect(widgetEntitlements.contains("group.dev.heimdall.heimdallbar"))
    }

    private static func firstMatch(in text: String, pattern: String) -> String? {
        guard let regex = try? NSRegularExpression(pattern: pattern) else {
            return nil
        }
        let range = NSRange(text.startIndex..<text.endIndex, in: text)
        guard let match = regex.firstMatch(in: text, range: range),
              match.numberOfRanges > 1,
              let captureRange = Range(match.range(at: 1), in: text) else {
            return nil
        }
        return String(text[captureRange])
    }
}
