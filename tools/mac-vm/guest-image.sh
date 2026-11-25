#!/bin/bash
# Runs inside the guest while `tools/mac-vm.sh build-image` builds a golden
# image from the Cua macOS base. Installs the tools and the agents Mash
# detects, settles macOS and Raycast settings, and writes the made-up demo
# histories. It installs no Mash, Raycast extension or desktop app, and signs
# in to nothing: those belong to each run, not to the image.
#
# Usage: guest-image.sh system | apps
#   system  settings that need a reboot to take effect (run first)
#   apps    Homebrew, Node 22, Git, Raycast and the agents (after the reboot)
set -euo pipefail
HERE=$(cd "$(dirname "$0")" && pwd)

system() {
  # The Cua image writes /etc/kcpassword without the cipher's padding, so
  # automatic login hands the keychain a wrong password and macOS replaces the
  # login keychain on every boot. Raycast keeps its database key there, so its
  # data would be lost at each start. Encode "lume" the way loginwindow expects.
  /usr/bin/python3 - <<'EOF'
key = [0x7D, 0x89, 0x52, 0x23, 0xD2, 0xBC, 0xDD, 0xEA, 0xA3, 0xB9, 0x1F]
data = b"lume" + b"\0" * 8
open("/tmp/kcpassword", "wb").write(bytes(b ^ key[i % len(key)] for i, b in enumerate(data)))
EOF
  sudo install -m 600 -o root -g wheel /tmp/kcpassword /etc/kcpassword
  rm /tmp/kcpassword
  # No macOS updates behind the image's back.
  sudo softwareupdate --schedule off >/dev/null || true
  sudo defaults write /Library/Preferences/com.apple.SoftwareUpdate AutomaticDownload -bool false
  sudo defaults write /Library/Preferences/com.apple.SoftwareUpdate AutomaticallyInstallMacOSUpdates -bool false
  sudo defaults write /Library/Preferences/com.apple.commerce AutoUpdate -bool false
  # A plain desktop for screenshots: no widgets, and a click on the wallpaper
  # (closing a notice that already went away) does not sweep windows aside.
  defaults write com.apple.WindowManager StandardHideWidgets -bool true
  defaults write com.apple.WindowManager StageManagerHideWidgets -bool true
  defaults write com.apple.WindowManager EnableStandardClickToShowDesktop -bool false
}

apps() {
  if [ ! -x /opt/homebrew/bin/brew ]; then
    NONINTERACTIVE=1 /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
  fi
  eval "$(/opt/homebrew/bin/brew shellenv)"
  grep -q 'brew shellenv' ~/.zprofile 2>/dev/null || {
    echo 'eval "$(/opt/homebrew/bin/brew shellenv)"'
    echo 'export PATH="/opt/homebrew/opt/node@22/bin:$HOME/.local/bin:$PATH"'
  } >> ~/.zprofile
  export HOMEBREW_NO_AUTO_UPDATE=1 HOMEBREW_NO_ANALYTICS=1 HOMEBREW_NO_INSTALL_CLEANUP=1
  brew install node@22 git opencode displayplacer
  # The agents whose histories Mash reads. Installed only: nothing is signed
  # in and no key is added, so none of them can talk to a model.
  brew install --cask raycast claude-code codex copilot-cli cursor antigravity
  # Opened over SSH, a quarantined app waits behind Gatekeeper's "downloaded
  # from the Internet" question that nobody is there to answer.
  for app in Raycast Cursor Antigravity; do
    xattr -dr com.apple.quarantine "/Applications/$app.app" 2>/dev/null || true
  done
  brew cleanup --prune=all >/dev/null 2>&1 || true

  # The guest display stays 1440x900 at 1x so screenshots and the scripted
  # clicks share one coordinate space.
  displayplacer "id:$(displayplacer list | awk '/Persistent screen id:/ {print $4; exit}') res:1440x900 scaling:off" || true

  # Raycast: past onboarding, no automatic updates. DisableAutomaticUpdates is
  # the key Raycast reads for managed installs.
  defaults write com.raycast.macos raycast_OnboardingCompleted -bool true
  defaults write com.raycast.macos raycast_WelcomeTosAccepted -bool true
  defaults write com.raycast.macos DisableAutomaticUpdates -bool true

  "$HERE/demo_history.py" "$HOME"
}

"$@"
