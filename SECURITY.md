# Security Policy

## Supported Versions

This project is currently in early development. Only the latest `main` branch is supported.

## Threat Model (Scope)

GRUBST is designed to harden GRUB interactive actions (editing entries, GRUB command line) and provide an optional USB-based maintenance unlock.

### Attacker capabilities (assumed)

- Can physically access the machine while it is powered off or at the GRUB menu.
- Can attempt to edit GRUB menu entries or use the GRUB command line.
- Can steal or clone the USB rescue key contents (treat the USB as a bearer item).

### Attacker capabilities (out of scope / not prevented by GRUBST alone)

- A fully privileged attacker with root on the running OS can disable or bypass most bootloader-based protections.
- If Secure Boot is not used, an attacker with physical access may boot external media, modify the EFI/boot partition, or tamper with `/boot` offline (depending on platform configuration).
- If disk encryption (e.g., LUKS) is not used, offline tampering with OS files remains possible even if GRUB menu editing is locked.

### Recommended pairing

- Use GRUBST together with Secure Boot (when feasible) and full-disk encryption (LUKS) for a stronger chain of trust.
- Consider disabling external boot in firmware and protecting firmware settings with a password, if your threat model includes physical access.

## Reporting a Vulnerability

Please report security issues responsibly.

### Preferred: Private disclosure

- Use GitHub Security Advisories (recommended if this repository is hosted on GitHub).

### If private reporting is not available

- Create a new issue and clearly mark it as security-related.
- Do not include sensitive details (proof-of-concept, exploit steps, or secrets). Provide high-level impact first, then we will request details.

## Scope Notes

GRUBST modifies bootloader configuration and formats USB devices. Any report involving:

- privilege escalation
- unintended disk/USB destruction
- bypassing GRUB authentication
- leaking GRUB password hashes / tokens

is considered security-relevant.

## Update Guard Notes

GRUBST installs an Update Guard script that runs during `update-grub` generation and checks whether critical GRUBST components still exist and look consistent. It is primarily a detection and warning mechanism; it does not attempt to fully self-heal every possible GRUB update/tamper scenario.
