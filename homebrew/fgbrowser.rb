cask "fgbrowser" do
  version "0.1.6"
  
  on_arm do
    sha256 "7d104866b54e617642b197ca499b683ba661e202f43fa45657ff473024438b0a"
    url "https://github.com/nodtOx/FGBrowser/releases/download/v#{version}/FGBrowser_#{version}_aarch64.dmg"
  end
  
  on_intel do
    sha256 "REPLACE_WITH_SHA256_OF_X86_64_DMG"
    url "https://github.com/nodtOx/FGBrowser/releases/download/v#{version}/FGBrowser_#{version}_x86_64.dmg"
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

