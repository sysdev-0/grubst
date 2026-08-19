# Contributing to GRUBST

Thanks for your interest in contributing.

## Ground Rules

- Be careful: this project touches GRUB and block devices.
- Avoid introducing behavior that could destroy data without explicit confirmation.
- Do not log or display secrets (passwords, tokens, hashes).

## Development Setup

### Requirements

- Linux
- Rust toolchain (stable)
- System tools used by GRUBST: `lsblk`, `blkid`, `wipefs`, `fdisk`, `mkfs.vfat`, `update-grub` or `grub-mkconfig`

### Build & test

```bash
cargo test
cargo build
```

### Run (GUI)

Most features require root:

```bash
sudo cargo run
```

## Code Style

- Follow existing patterns in `src/core/` and `src/gui/`.
- Keep functions small and prefer explicit error messages.
- Avoid adding new dependencies unless necessary.

## Pull Requests

- Keep PRs focused (one feature/fix per PR).
- Include manual test steps (VM recommended).
- Mention distro/boot mode tested (UEFI vs Legacy) if relevant.

