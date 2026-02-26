BINARY := clipboard-hack
INSTALL_BIN := $(HOME)/.local/bin/$(BINARY)
INSTALL_ICON := $(HOME)/.local/share/icons/hicolor/256x256/apps/$(BINARY).png
INSTALL_DESKTOP := $(HOME)/.local/share/applications/$(BINARY).desktop

.PHONY: build install uninstall

build:
	cargo build --release

install: build
	@mkdir -p $(HOME)/.local/bin
	@mkdir -p $(HOME)/.local/share/icons/hicolor/256x256/apps
	@mkdir -p $(HOME)/.local/share/applications
	cp target/release/$(BINARY) $(INSTALL_BIN)
	cp assets/icon.png $(INSTALL_ICON)
	sed "s|{{EXEC}}|$(INSTALL_BIN)|g" assets/clipboard-hack.desktop > $(INSTALL_DESKTOP)
	@echo "Installed. Run: $(INSTALL_BIN)"
	@echo "You may need to run: update-desktop-database ~/.local/share/applications"

uninstall:
	rm -f $(INSTALL_BIN)
	rm -f $(INSTALL_ICON)
	rm -f $(INSTALL_DESKTOP)
	@echo "Uninstalled."
