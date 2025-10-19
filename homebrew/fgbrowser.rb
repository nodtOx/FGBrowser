cask "fgbrowser" do
  version "0.1.10"
  
  on_arm do
    sha256 "a0e5b536035a2bd3d22bca0e6eedaaa1c2c3c78b977ad8dbda2d79d1e1c09797"
    url "https://github.com/nodtOx/FGBrowser/releases/download/v#{version}/FGBrowser_#{version}_aarch64.dmg"
  end
  
  on_intel do
    sha256 "65c7e3406e855bae0d1b023f61d182ad1e890d35f9773744b9120e31829acf95"
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

