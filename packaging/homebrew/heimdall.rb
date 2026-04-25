# Homebrew cask formula for Heimdall.
#
# TODO: Before publishing this formula:
#   1. Create a new GitHub repository named "homebrew-tap" under your org:
#        https://github.com/YOUR_ORG/homebrew-tap
#   2. Copy this file to `Casks/heimdall.rb` in that repo.
#   3. After each release, update `version`, `sha256`, and confirm the `url`
#      points at the correct universal tarball on the GitHub Releases page.
#
# Users install via:
#   brew tap YOUR_ORG/tap
#   brew install YOUR_ORG/tap/heimdall

cask "heimdall" do
  version "0.1.0"  # bumped manually or via a release script
  sha256 "SHA256_PLACEHOLDER_REPLACE_ON_RELEASE"

  # TODO: replace with actual GitHub org before publishing
  url "https://github.com/YOUR_ORG/heimdall/releases/download/v#{version}/heimdall-#{version}-universal-apple-darwin.tar.gz"
  name "Heimdall"
  desc "Local analytics dashboard for coding agent usage (Claude Code, Codex, Cursor, OpenCode, Pi, Copilot, Xcode, Cowork, Amp)"
  homepage "https://github.com/YOUR_ORG/heimdall"

  binary "claude-usage-tracker"
  binary "heimdall-hook"

  preflight do
    # Uninstall any existing scheduler entry before upgrading.
    system_command "#{staged_path}/claude-usage-tracker",
                   args: ["scheduler", "uninstall"],
                   must_succeed: false
  end

  postflight do
    # Strip quarantine so Gatekeeper doesn't block on first run.
    system_command "/usr/bin/xattr",
                   args: ["-cr", "#{staged_path}"],
                   must_succeed: false
    # Optional: auto-install the hourly scheduler.
    # Uncomment the lines below to enable automatic scheduler setup on install.
    # system_command "#{staged_path}/claude-usage-tracker",
    #                args: ["scheduler", "install", "--interval", "hourly"],
    #                must_succeed: false
  end

  zap trash: [
    "~/.claude/usage-tracker.toml",
    "~/.config/heimdall",
    "~/.cache/heimdall",
    "~/Library/LaunchAgents/dev.heimdall.scan.plist",
    "~/Library/LaunchAgents/dev.heimdall.daemon.plist",
  ]
end
