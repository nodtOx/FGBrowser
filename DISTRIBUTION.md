# macOS Distribution Strategy for FGBrowser

## Problem

- macOS blocks unsigned apps with "damaged and can't be opened" error
- Signing with Apple Developer certificate exposes your identity ($99/year)
- You want to remain anonymous

## Solution: Homebrew Distribution

### Why Homebrew?

✅ **No quarantine warnings** - Homebrew automatically handles this  
✅ **No "damaged app" errors** - Clean installation  
✅ **Professional distribution** - Users trust Homebrew  
✅ **Easy updates** - `brew upgrade fgbrowser`  
✅ **Maintains anonymity** - No code signing needed  
✅ **Same as Transmission** - Major torrent clients use this approach

## Implementation

### 1. Current Setup (Already Done)

- ✅ Ad-hoc signing configured (`signingIdentity: "-"`)
- ✅ Entitlements file created
- ✅ GitHub Actions builds DMG files
- ✅ SHA256 calculation added to workflow
- ✅ README updated with Homebrew instructions

### 2. Create Homebrew Tap (To Do)

1. **Create GitHub repo**: `homebrew-fgbrowser`

   ```bash
   # On GitHub, create new public repo: homebrew-fgbrowser
   ```

2. **Add the cask file**:

   ```bash
   git clone https://github.com/nodtOx/homebrew-fgbrowser.git
   cd homebrew-fgbrowser
   mkdir -p Casks
   cp path/to/fit-boy/homebrew/fgbrowser.rb Casks/
   ```

3. **Wait for v0.1.4 release to complete**, then:

   - Check the GitHub Actions logs for SHA256 hashes
   - Update the hashes in `Casks/fgbrowser.rb`
   - Commit and push to `homebrew-fgbrowser` repo

4. **Test**:
   ```bash
   brew tap nodtox/fgbrowser
   brew install --cask fgbrowser
   ```

### 3. User Installation

Users install with:

```bash
brew tap nodtox/fgbrowser
brew install --cask fgbrowser
```

No security warnings, no quarantine errors!

## Comparison with Other Approaches

### Option 1: Homebrew (✅ Chosen)

- **Cost**: Free
- **Privacy**: Complete anonymity
- **User Experience**: Excellent (one command, no warnings)
- **Maintenance**: Low (just update SHA256 on releases)

### Option 2: Apple Developer Signing

- **Cost**: $99/year
- **Privacy**: Your identity visible in app signature
- **User Experience**: Perfect (no warnings at all)
- **Maintenance**: Moderate (manage certificates)

### Option 3: Business/LLC Signing

- **Cost**: $200-500 setup + $99/year Apple
- **Privacy**: Business name visible (partial anonymity)
- **User Experience**: Perfect
- **Maintenance**: High (legal entity management)

### Option 4: Unsigned + Instructions

- **Cost**: Free
- **Privacy**: Complete anonymity
- **User Experience**: Poor (scary warnings, manual steps)
- **Maintenance**: None

## What Transmission Does

From their GitHub Actions workflow analysis:

- They build unsigned macOS apps
- They have certificate infrastructure but skip it if not configured
- They rely on Homebrew and user workarounds

**You're following industry best practices for anonymous, free distribution.**

## Release Workflow

### Simple 2-Step Process

**Step 1: Bump version**

```bash
make bump-patch  # or bump-minor, bump-major
```

**Step 2: Create release** (this triggers GitHub Actions to build)

```bash
make release
```

**Step 3: Wait for build & update Homebrew** (automatic)

```bash
make update-homebrew-sha
```

This command will:

- ⏳ Wait for GitHub Actions to complete (~10 min)
- 📥 Download DMG files from the release
- 🔐 Calculate SHA256 hashes
- ✅ Update `homebrew/fgbrowser.rb` automatically

**Step 4: Commit and push SHA256 update**

```bash
git add homebrew/fgbrowser.rb
git commit -m "chore: update Homebrew SHA256 for vX.X.X"
git push
```

**Step 5: Copy to homebrew-fgbrowser repo**

```bash
cp homebrew/fgbrowser.rb ../homebrew-fgbrowser/Casks/
cd ../homebrew-fgbrowser
git add Casks/fgbrowser.rb
git commit -m "Update FGBrowser to vX.X.X"
git push
```

## Next Steps

1. ✅ Local SHA256 calculation (done)
2. ✅ GitHub Actions reverted to v0.1.3 (done)
3. 🍺 Create `homebrew-fgbrowser` repo (to do)
4. 📦 Test full workflow with next release
5. 🎉 Users can install via Homebrew!

## Automation Ideas

Future improvements:

- Auto-update Homebrew cask on release
- Use GitHub Actions to calculate and commit hashes
- Create PR to homebrew-fgbrowser automatically

For now, manual updates on each release are simple and reliable.
