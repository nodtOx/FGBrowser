# Setting Up Homebrew Tap for FGBrowser

This guide explains how to create and maintain a Homebrew tap for FGBrowser distribution.

## Benefits of Homebrew Distribution

✅ **No quarantine warnings** - Homebrew handles this automatically  
✅ **No "damaged app" errors** - Users get a clean installation  
✅ **Easy updates** - `brew upgrade fgbrowser`  
✅ **Maintains anonymity** - No code signing required  
✅ **Professional distribution** - Users trust Homebrew

## Prerequisites

### GitHub CLI Authentication

The automated SHA256 update script uses GitHub CLI. Set up a token for your nodtOx account:

1. **Create Personal Access Token:**
   - Go to: https://github.com/settings/tokens
   - Click "Generate new token (classic)"
   - Select scopes: `repo`, `workflow`
   - Generate and copy the token

2. **Add to your shell profile** (`~/.zshrc` or `~/.bashrc`):
   ```bash
   export GH_TOKEN_NODTOX="ghp_your_token_here"
   ```

3. **Reload shell:**
   ```bash
   source ~/.zshrc
   ```

Now the script will use your nodtOx token instead of your primary GitHub account!

## Setup Steps

### 1. Create the Tap Repository

Create a new GitHub repository named `homebrew-fgbrowser`:

```bash
# On GitHub, create a new public repo: homebrew-fgbrowser
```

### 2. Add the Cask File

Copy `fgbrowser.rb` to the tap repository:

```bash
# Clone your new tap repo
git clone https://github.com/nodtOx/homebrew-fgbrowser.git
cd homebrew-fgbrowser

# Create Casks directory
mkdir -p Casks

# Copy the cask file
cp path/to/fit-boy/homebrew/fgbrowser.rb Casks/
```

### 3. Update SHA256 Hashes

For each release, you need to get the SHA256 hash of the DMG files:

```bash
# After GitHub Actions builds your release, download the DMGs
# Then calculate their hashes:

shasum -a 256 FGBrowser_0.1.4_aarch64.dmg
shasum -a 256 FGBrowser_0.1.4_x86_64.dmg
```

Update the hashes in `Casks/fgbrowser.rb`:

```ruby
on_arm do
  sha256 "THE_HASH_YOU_GOT_FROM_SHASUM"
  url "https://github.com/nodtOx/FGBrowser/releases/download/v#{version}/FGBrowser_#{version}_aarch64.dmg"
end

on_intel do
  sha256 "THE_HASH_YOU_GOT_FROM_SHASUM"
  url "https://github.com/nodtOx/FGBrowser/releases/download/v#{version}/FGBrowser_#{version}_x86_64.dmg"
end
```

### 4. Test Locally

```bash
# Add your tap
brew tap nodtox/fgbrowser

# Test installation
brew install --cask fgbrowser

# If it works, test uninstall
brew uninstall --cask fgbrowser
```

### 5. Publish

```bash
cd homebrew-fgbrowser
git add Casks/fgbrowser.rb
git commit -m "Add FGBrowser v0.1.4"
git push
```

## Updating for New Releases

When you release a new version:

1. Update `version` in `fgbrowser.rb`
2. Download the new DMG files
3. Calculate new SHA256 hashes
4. Update the `sha256` values
5. Commit and push

## Automation (Optional)

You could automate the hash calculation in your GitHub Actions workflow:

```yaml
- name: Calculate SHA256 for Homebrew
  run: |
    echo "ARM64 SHA256:"
    shasum -a 256 src-tauri/target/release/bundle/dmg/*_aarch64.dmg
    echo "x86_64 SHA256:"
    shasum -a 256 src-tauri/target/release/bundle/dmg/*_x86_64.dmg
```

## User Installation

Once published, users can install with:

```bash
brew tap nodtox/fgbrowser
brew install --cask fgbrowser
```

No quarantine errors, no security warnings!
