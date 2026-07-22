# Homebrew formula for the CFDL CLI (generated — do not hand-edit).
#
# Produced by distribution/scripts/gen_homebrew.sh from a release's assets.
# The placeholders below are filled with the tagged version, the GitHub
# release download URLs, and each binary's sha256. Publishing to a tap is a
# separate, human-approved step (LAUNCH_PLAN rule 5.6) — this file is built
# and audited locally, never pushed to a tap by CI.
class Cfdl < Formula
  desc "Cash Flow Domain Language — compiler, engine, and CLI"
  homepage "https://cfdl.dev"
  version "__VERSION__"
  license "BUSL-1.1"

  on_macos do
    on_arm do
      url "https://github.com/bizarc/cfdl/releases/download/v__VERSION__/cfdl-darwin-arm64"
      sha256 "__SHA_DARWIN_ARM64__"
    end
    on_intel do
      url "https://github.com/bizarc/cfdl/releases/download/v__VERSION__/cfdl-darwin-x64"
      sha256 "__SHA_DARWIN_X64__"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/bizarc/cfdl/releases/download/v__VERSION__/cfdl-linux-x64"
      sha256 "__SHA_LINUX_X64__"
    end
  end

  def install
    # Release assets are bare binaries named per platform; install as `cfdl`.
    bin.install Dir["cfdl-*"].first => "cfdl"
  end

  test do
    assert_match "cfdl", shell_output("#{bin}/cfdl --help 2>&1", 2)
  end
end
