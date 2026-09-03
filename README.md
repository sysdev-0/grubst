<div dir="rtl">

# 🛡️ GRUBST — Boot Security Hardening

</div>

> **GRUBST** locks down your GRUB bootloader with password protection and provides a physical USB rescue key for emergency access.

![License](https://img.shields.io/badge/license-BSL--1.1-blue)
![Rust](https://img.shields.io/badge/rust-stable-orange)
![Platform](https://img.shields.io/badge/platform-linux-green)
[![Donate with PayPal](https://img.shields.io/badge/Donate-PayPal-00457C?logo=paypal&logoColor=white)](https://paypal.me/AamirSaeed713)

---

## ⚠️ WARNING

**This tool modifies your GRUB bootloader configuration.**
Incorrect usage may prevent your system from booting.
Use at your own risk. The author assumes **NO liability**.
Always have a backup recovery plan before using this tool.

---

## What does GRUBST do?

| Feature | Description |
|---------|-------------|
| 🔒 **Lock GRUB** | Prevents editing boot parameters, accessing recovery mode, or using the GRUB command line without a password |
| 🔑 **USB Rescue Key** | A physical USB drive acts as an instant unlock key — just plug it in before booting |
| 🔐 **Backup Password** | A secondary password you choose, in case you lose the USB key |
| 🛡️ **Update Guard** | Protection survives `update-grub` and kernel updates automatically |
| 💾 **Auto Backup** | GRUB configs are backed up before any changes, with instant restore |
| 🔍 **Security Audit** | Scans for disk encryption, Secure Boot, BIOS settings, and more |

### What it does NOT do

- ❌ Does **not** replace your OS login screen password
- ❌ Does **not** encrypt your disk (use LUKS for that)
- ❌ Does **not** interfere with normal daily booting — you boot as usual

---

## 📥 Installation

> **🔒 Security Notice:** GRUBST does not provide pre-built binaries. You must build from source on your own machine for security and transparency.

### ⚡ Quick Install

Follow these commands **one by one**:

```bash
# 1. Install Git (if not installed)
sudo apt install git

# 2. Clone the repository
git clone https://github.com/sysdev-0/grubst.git

# 3. Enter the directory
cd grubst

# 4. Install Rust (if not installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 5. Install system dependencies
sudo apt update
sudo apt install build-essential pkg-config libgtk-3-dev \
  libwebkit2gtk-4.0-dev libayatana-appindicator3-dev \
  librsvg2-dev polkit-1

# 6. Build the project (takes 2-5 minutes)
cargo build --release

# 7. Install with desktop integration
sudo make install
```

**Done!** GRUBST will now appear in your application menu.

Launch from your application menu or run `sudo grubst`.

---

### 🔄 Updating GRUBST

After pulling new updates from Git, rebuild and reinstall to update the desktop app:

```bash
cd grubst
git pull
cargo build --release
sudo make install
```

This will update the installed binary and refresh the desktop integration.

---

### 📦 Requirements

- **Supported OS:** Ubuntu/Debian (other distributions may work but are not officially tested)
- Linux with GRUB2 bootloader
- Git
- Rust toolchain
- GTK3 and WebKit2GTK libraries
- Polkit (for GUI)
- A USB drive (≥ 4 GB, **will be formatted during setup**)


---

## Usage

**GRUBST must be run as root** for lock/unlock operations:

```bash
sudo grubst
```

This opens the GUI where you can:

1. **Home** — View protection status and system overview
2. **Lock** — Step-by-step wizard to lock GRUB and program a USB rescue key
3. **Unlock** — Remove protection and restore original GRUB config
4. **Audit** — Run a comprehensive boot security scan

---

## How the USB Rescue Key works

1. You provide a USB drive (≥ 4 GB) — **all data will be erased**
2. GRUBST formats it as FAT32 and writes authentication data
3. The USB is **machine-bound** (via a stored machine fingerprint used by GRUB) — it won't auto-unlock on other computers
4. At boot time, GRUB checks if the USB is connected:
   - ✅ **USB found** → GRUB unlocks automatically for maintenance
   - ❌ **USB not found** → GRUB stays locked (password required to edit)
   - ❌ **Wrong USB / wrong machine** → Access denied

---

## How protection works

```
Normal boot (no USB):
┌──────────┐    ┌──────────────┐    ┌────────────────┐
│  BIOS    │───▶│  GRUB Menu   │───▶│  Linux boots   │
│          │    │  (locked)    │    │  normally ✅    │
└──────────┘    │  Can't edit  │    └────────────────┘
                │  Can't recover│
                └──────────────┘

With USB rescue key:
┌──────────┐    ┌──────────────┐    ┌────────────────┐
│  BIOS    │───▶│  GRUB Menu   │───▶│  Full access   │
│          │    │  (unlocked)  │    │  Edit, recover │
└──────────┘    │  USB detected│    │  maintenance ✅│
                └──────────────┘    └────────────────┘
```

### GRUB Superuser Account

GRUBST creates a standardized GRUB superuser account:
- **Username:** `grubst_admin` (fixed, cannot be changed)
- **Password:** Auto-generated secure password (stored internally)

The fixed username prevents confusion and ensures consistency across installations. You don't need to remember it — the USB key handles authentication automatically.

---

## Project Structure

```
src/
├── main.rs                 # Entry point (launches GUI)
├── lib.rs                  # Library exports (core + gui)
├── gui/
│   ├── mod.rs              # Dioxus app shell + navigation
│   ├── style.css           # UI styling
│   └── screens/
│       ├── home.rs         # Dashboard screen
│       ├── lock.rs         # Lock wizard (4 steps)
│       ├── unlock.rs       # Unlock screen
│       └── audit.rs        # Security audit screen
├── core/
│   ├── crypto.rs           # PBKDF2 hashing, token generation
│   ├── usb.rs              # USB detection, formatting, writing
│   ├── grub.rs             # GRUB config management
│   ├── fingerprint.rs      # Machine fingerprint (hardware binding)
│   ├── audit.rs            # Security checks (8 checks)
│   ├── backup.rs           # Backup & restore
│   └── update_guard.rs     # update-grub protection hook
```

---

## 💖 Support & Donations

If you find **GRUBST** valuable and want to support its ongoing development, improvements, and maintenance, you can donate via PayPal:

[![Donate with PayPal](https://img.shields.io/badge/Donate-PayPal-00457C?style=for-the-badge&logo=paypal&logoColor=white)](https://paypal.me/AamirSaeed713)

Your support is greatly appreciated!

---

## License

This project is licensed under the **Business Source License 1.1 (BSL 1.1)** — see the [LICENSE](LICENSE) file for details.

- **Non-Commercial / Individual Use:** Free for personal, educational, research, testing, and evaluation purposes.
- **Commercial / Enterprise Use:** Commercial use within businesses, enterprises, or commercial products/services requires a commercial license agreement from the authors.

## Security Model

See [SECURITY.md](SECURITY.md) for the threat model scope and update-guard notes.

**THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND.**
The author is not responsible for any damage caused by using this software.
