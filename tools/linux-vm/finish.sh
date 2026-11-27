#!/bin/bash
# Last step of the Ubuntu golden image's autoinstall, run inside the installed
# system (curtin in-target) before the first boot. Makes the GNOME desktop
# log mash on by itself and stay quiet and unlocked, so a run can start the
# app on screen and screenshot it over VNC. Everything here is test-VM
# configuration; none of it suits a real machine.
set -euo pipefail

# Log mash on at boot. Wayland stays on: the screenshots come from QEMU's
# display, not from inside the session, and the USB tablet works there.
cat >/etc/gdm3/custom.conf <<'EOF'
[daemon]
AutomaticLoginEnable=true
AutomaticLogin=mash
EOF

# Runs install the build under test with apt over SSH, without a password.
echo 'mash ALL=(ALL) NOPASSWD: ALL' >/etc/sudoers.d/mash
chmod 440 /etc/sudoers.d/mash

# GNOME defaults for every user: dark style, a plain dark background, no
# banners, no lock, no blanking, no sleep, and no welcome tour.
mkdir -p /etc/dconf/profile /etc/dconf/db/local.d
printf 'user-db:user\nsystem-db:local\n' >/etc/dconf/profile/user
cat >/etc/dconf/db/local.d/00-mash-vm <<'EOF'
[org/gnome/desktop/interface]
color-scheme='prefer-dark'
gtk-theme='Yaru-dark'
enable-animations=false

[org/gnome/desktop/background]
picture-uri=''
picture-uri-dark=''
picture-options='none'
color-shading-type='solid'
primary-color='#202226'

[org/gnome/desktop/notifications]
show-banners=false

[org/gnome/desktop/session]
idle-delay=uint32 0

[org/gnome/desktop/screensaver]
lock-enabled=false
idle-activation-enabled=false

[org/gnome/settings-daemon/plugins/power]
sleep-inactive-ac-type='nothing'
idle-dim=false

[org/gnome/shell]
welcome-dialog-last-shown-version='999'
EOF
dconf update

# No "welcome to Ubuntu" wizard at the first logon.
apt-get remove -y gnome-initial-setup || true
mkdir -p /home/mash/.config
echo yes >/home/mash/.config/gnome-initial-setup-done
chown -R mash:mash /home/mash/.config

# No update checks, upgrade prompts or crash reports behind a test.
apt-get remove -y update-notifier update-manager || true
cat >/etc/apt/apt.conf.d/99mash-vm <<'EOF'
APT::Periodic::Update-Package-Lists "0";
APT::Periodic::Unattended-Upgrade "0";
APT::Periodic::Download-Upgradeable-Packages "0";
EOF
systemctl disable unattended-upgrades.service apt-daily.timer apt-daily-upgrade.timer || true
sed -i 's/^enabled=1/enabled=0/' /etc/default/apport || true
sed -i 's/^Prompt=.*/Prompt=never/' /etc/update-manager/release-upgrades 2>/dev/null || true
