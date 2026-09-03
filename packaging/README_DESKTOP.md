# Desktop integration (Linux)

This folder contains files to integrate GRUBST into desktop app menus and launch it via Polkit (`pkexec`).

## Prerequisites

- A working desktop session (X11 or Wayland)
- Polkit (`pkexec`)
- The GRUBST binary installed at:
  - `/usr/local/bin/grubst`

## Install (system-wide)

1) Copy the desktop entry:

```bash
sudo install -Dm644 packaging/grubst.desktop /usr/share/applications/grubst.desktop
```

2) Copy the GUI launcher (calls pkexec and preserves desktop session env):

```bash
sudo install -Dm755 packaging/grubst-gui-launcher /usr/local/bin/grubst-gui-launcher
```

3) Copy the icon (SVG):

```bash
sudo install -Dm644 packaging/icons/hicolor/scalable/apps/grubst.svg \
  /usr/share/icons/hicolor/scalable/apps/grubst.svg
sudo gtk-update-icon-cache -f /usr/share/icons/hicolor || true
```

4) Ensure the binary exists at:

```text
/usr/local/bin/grubst
```

The desktop entry uses:

```text
Exec=/usr/local/bin/grubst-gui-launcher
```

## Troubleshooting

If the password dialog appears but the UI does not open:

1) Check the latest launcher error log:

```bash
ls -1t /tmp/grubst-desktop-*.err | head -n 1
```

2) If no new `/tmp/grubst-desktop-*` files are created when you click the app icon, your desktop may be using a cached `.desktop` entry. Run the cache update and log out/in:

```bash
sudo update-desktop-database /usr/share/applications || true
```

3) If logs contain `cannot open display:`, GRUBST is being started without the desktop session environment. Ensure the desktop entry executes `grubst-gui-launcher` (not `/usr/local/bin/grubst` directly).

## Notes

- No custom Polkit policy is included yet (by request).
- If your distro caches desktop entries, you may need to log out/in or run:

```bash
update-desktop-database /usr/share/applications || true
```
