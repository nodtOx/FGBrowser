cask "fgbrowser" do
  version "0.1.10"
  
  on_arm do
    sha256 "a2dd075c13e4cf563fbc72e9593623bac8b56b79f7e94e2b800c304220594ba6"
    url "https://github.com/nodtOx/FGBrowser/releases/download/v#{version}/FGBrowser_#{version}_aarch64.dmg"
  end
  
  on_intel do
    sha256 "9fd2888d31ddef335747703f2d6062d336baa7ea6f37d5f6d5e0eea60eb8e4ae"
    url "https://github.com/nodtOx/FGBrowser/releases/download/v#{version}/FGBrowser_#{version}_x64.dmg"
  end

  name "FGBrowser"
  desc "Desktop application for browsing and organizing FitGirl Repack information"
  homepage "https://github.com/nodtOx/FGBrowser"

  livecheck do
    url :url
    strategy :github_latest
  end

  app "FGBrowser.app"

  zap trash: [
    "~/Library/Application Support/com.pc.fgbrowser",
    "~/Library/Caches/com.pc.fgbrowser",
    "~/Library/Preferences/com.pc.fgbrowser.plist",
    "~/Library/Saved Application State/com.pc.fgbrowser.savedState",
  ]
end

