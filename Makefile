# Makefile for GRUBST installation
# GRUBST - Boot Security Hardening Tool

PREFIX ?= /usr/local
BINDIR = $(PREFIX)/bin
APPLICATIONSDIR = /usr/share/applications
ICONSDIR = /usr/share/icons/hicolor/scalable/apps
DOCDIR = /usr/share/doc/grubst

.PHONY: install uninstall

install:
	@echo "🔨 Installing GRUBST..."
	@echo ""
	
	# Install main binary
	@echo "   → Installing binary to $(BINDIR)/grubst"
	@install -Dm755 target/release/grubst $(DESTDIR)$(BINDIR)/grubst
	
	# Install GUI launcher
	@echo "   → Installing GUI launcher"
	@install -Dm755 packaging/grubst-gui-launcher $(DESTDIR)$(BINDIR)/grubst-gui-launcher
	
	# Install desktop file
	@echo "   → Installing desktop entry"
	@install -Dm644 packaging/grubst.desktop $(DESTDIR)$(APPLICATIONSDIR)/grubst.desktop
	
	# Install icon
	@echo "   → Installing icon"
	@install -Dm644 assets/grubst_logo.svg $(DESTDIR)$(ICONSDIR)/grubst.svg
	
	# Install documentation
	@echo "   → Installing documentation"
	@install -Dm644 README.md $(DESTDIR)$(DOCDIR)/README.md
	@install -Dm644 SECURITY.md $(DESTDIR)$(DOCDIR)/SECURITY.md
	@install -Dm644 CONTRIBUTING.md $(DESTDIR)$(DOCDIR)/CONTRIBUTING.md
	@install -Dm644 CODE_OF_CONDUCT.md $(DESTDIR)$(DOCDIR)/CODE_OF_CONDUCT.md
	@install -Dm644 LICENSE $(DESTDIR)$(DOCDIR)/LICENSE
	
	# Update desktop database
	@echo "   → Updating desktop database"
	@which update-desktop-database > /dev/null 2>&1 && update-desktop-database $(DESTDIR)$(APPLICATIONSDIR) || true
	
	# Update icon cache
	@echo "   → Updating icon cache"
	@which gtk-update-icon-cache > /dev/null 2>&1 && gtk-update-icon-cache -f /usr/share/icons/hicolor || true
	
	@echo ""
	@echo "✅ GRUBST installed successfully!"
	@echo ""
	@echo "   Launch from your application menu or run:"
	@echo "   sudo grubst"
	@echo ""

uninstall:
	@echo "🗑️  Uninstalling GRUBST..."
	@echo ""
	@rm -f $(DESTDIR)$(BINDIR)/grubst
	@rm -f $(DESTDIR)$(BINDIR)/grubst-gui-launcher
	@rm -f $(DESTDIR)$(APPLICATIONSDIR)/grubst.desktop
	@rm -f $(DESTDIR)$(ICONSDIR)/grubst.svg
	@rm -rf $(DESTDIR)$(DOCDIR)
	@which update-desktop-database > /dev/null 2>&1 && update-desktop-database $(DESTDIR)$(APPLICATIONSDIR) || true
	@which gtk-update-icon-cache > /dev/null 2>&1 && gtk-update-icon-cache -f /usr/share/icons/hicolor || true
	@echo ""
	@echo "✅ GRUBST uninstalled successfully!"
	@echo ""
