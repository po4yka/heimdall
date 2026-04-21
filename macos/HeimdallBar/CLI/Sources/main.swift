import Darwin
import Foundation
import HeimdallCLI
import HeimdallPlatformMac

do {
    try await HeimdallCLIEntrypoint.run(
        arguments: CommandLine.arguments,
        dependencies: MacPlatformFactory.cliDependencies()
    )
} catch {
    fputs("\(error.localizedDescription)\n", stderr)
    Darwin.exit(1)
}
