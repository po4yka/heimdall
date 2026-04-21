import Darwin
import Foundation
import HeimdallCLI
import HeimdallPlatformMac

do {
    let compositionRoot = MacPlatformCompositionRoot()
    try await HeimdallCLIEntrypoint.run(
        arguments: CommandLine.arguments,
        dependencies: compositionRoot.cliDependencies()
    )
} catch {
    fputs("\(error.localizedDescription)\n", stderr)
    Darwin.exit(1)
}
