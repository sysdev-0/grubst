# 📥 GRUBST Installation Guide

## ⚡ Quick Install (3 Commands)

```bash
git clone https://github.com/sysdev-0/grubst.git && cd grubst
cargo build --release
sudo make install
```

Launch from your application menu or run `sudo grubst`.

---

## 🔒 Why Build from Source?

**GRUBST does not provide pre-built binaries for security reasons.**

Building from source ensures:
- ✅ **Code transparency** - Review what you're installing
- ✅ **No third-party binaries** - Build on your own machine
- ✅ **Hardware optimization** - Compiled for your specific system
- ✅ **Full control** - Complete visibility of the build process

---

## 📋 Detailed Installation Steps

### Step 1: Install Rust

If you don't have Rust installed:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

Verify installation:
```bash
rustc --version
cargo --version
```

---

### Step 2: Install System Dependencies

#### Ubuntu/Debian
```bash
sudo apt update
sudo apt install build-essential pkg-config \
  libgtk-3-dev libwebkit2gtk-4.0-dev \
  libayatana-appindicator3-dev librsvg2-dev \
  polkit-1
```

#### Fedora/RHEL
```bash
sudo dnf install gtk3-devel webkit2gtk4.0-devel \
  libappindicator-gtk3-devel librsvg2-devel \
  polkit
```

#### Arch Linux
```bash
sudo pacman -S base-devel gtk3 webkit2gtk \
  libappindicator-gtk3 polkit
```

#### openSUSE
```bash
sudo zypper install gtk3-devel webkit2gtk3-devel \
  libappindicator3-devel librsvg-devel polkit
```

---

### Step 3: Clone the Repository

```bash
git clone https://github.com/sysdev-0/grubst.git
cd grubst
```

---

### Step 4: Build the Project

```bash
cargo build --release
```

This will:
- Download and compile all dependencies
- Build an optimized binary for your system
- Create `target/release/grubst`

**Build time:** 2-5 minutes (depending on your system)

---

### Step 5: Install

```bash
sudo make install
```

This will:
- ✅ Install binary to `/usr/local/bin/grubst`
- ✅ Install GUI launcher to `/usr/local/bin/grubst-gui-launcher`
- ✅ Add desktop entry (appears in application menu)
- ✅ Install icon (`grubst.svg`)
- ✅ Install documentation to `/usr/share/doc/grubst/`
- ✅ Update desktop database
- ✅ Update icon cache

---

### Step 6: Launch

**From Application Menu:**
- Look for "GRUBST" in your applications
- Click to launch (will prompt for sudo password)

**From Terminal:**
```bash
sudo grubst
```

---

## 🔧 Manual Installation (Alternative)

If you prefer not to use the Makefile:

```bash
# Build
cargo build --release

# Install binary
sudo install -Dm755 target/release/grubst /usr/local/bin/grubst

# Install desktop integration
sudo install -Dm755 packaging/grubst-gui-launcher /usr/local/bin/grubst-gui-launcher
sudo install -Dm644 packaging/grubst.desktop /usr/share/applications/grubst.desktop
sudo install -Dm644 assets/grubst_logo.svg /usr/share/icons/hicolor/scalable/apps/grubst.svg

# Update desktop database
sudo update-desktop-database /usr/share/applications || true
sudo gtk-update-icon-cache -f /usr/share/icons/hicolor || true
```

---

## 🗑️ Uninstallation

```bash
cd grubst
sudo make uninstall
```

Or manually:
```bash
sudo rm /usr/local/bin/grubst
sudo rm /usr/local/bin/grubst-gui-launcher
sudo rm /usr/share/applications/grubst.desktop
sudo rm /usr/share/icons/hicolor/scalable/apps/grubst.svg
sudo rm -rf /usr/share/doc/grubst
sudo update-desktop-database /usr/share/applications || true
sudo gtk-update-icon-cache -f /usr/share/icons/hicolor || true
```

---

## 🐛 Troubleshooting

### Desktop entry doesn't appear

```bash
# Update desktop database
sudo update-desktop-database /usr/share/applications

# Log out and log back in
```

### Icon doesn't show

```bash
# Update icon cache
sudo gtk-update-icon-cache -f /usr/share/icons/hicolor

# Restart your desktop environment
```

### "Permission denied" error

Make sure you run with `sudo`:
```bash
sudo grubst
```

### Build fails with dependency errors

Make sure all system dependencies are installed (see Step 2).

### Rust version too old

Update Rust:
```bash
rustup update stable
```

---

## ✅ Verification

After installation, verify everything is in place:

```bash
# Check binary exists
which grubst
# Expected: /usr/local/bin/grubst

# Check launcher exists
which grubst-gui-launcher
# Expected: /usr/local/bin/grubst-gui-launcher

# Check desktop file
ls /usr/share/applications/grubst.desktop

# Check icon
ls /usr/share/icons/hicolor/scalable/apps/grubst.svg

# Check documentation
ls /usr/share/doc/grubst/
```

---

## 📚 Next Steps

After installation:
1. Read the [README.md](README.md) for usage instructions
2. Review [SECURITY.md](SECURITY.md) for security considerations
3. Launch GRUBST: `sudo grubst`
4. Start with the **Security Audit** to assess your current protection level

---

## 🆘 Getting Help

- 📖 Documentation: [README.md](README.md)
- 🐛 Report issues: [GitHub Issues](https://github.com/sysdev-0/grubst/issues)
- 💬 Discussions: [GitHub Discussions](https://github.com/sysdev-0/grubst/discussions)
- 🔒 Security issues: See [SECURITY.md](SECURITY.md)
