PRODUCT = aprovecha
DIR = bin
TMP = $(DIR)/$@
CARGOBUILD = cargo build --release
BINARY = target/$(1)/release/$(PRODUCT)
PREFIX = /usr/local

LINUX_ARCHES = x86_64 aarch64
LINUX_GOALS = $(addprefix linux-,$(LINUX_ARCHES))
LINUX_TARGET = $*-unknown-linux-musl

DISTR_LINUX = $(PRODUCT)-linux-$*
DISTR_MACOS = $(PRODUCT)-macos-universal
CODESIGN_ID_APP = AFB2E179D076A5FE9111BE28A2F4E0F52BDDF7C4

.PHONY: build install uninstall clean release linux $(LINUX_GOALS) macos
.DEFAULT_GOAL := build


$(DIR):
	mkdir $(DIR)


build: $(DIR)
	$(CARGOBUILD)
	@cp -v $(BINARY) $(DIR)/

install: build
	sudo cp -v $(BINARY) $(PREFIX)/bin/

uninstall:
	sudo rm -f $(PREFIX)/bin/$(PRODUCT)

clean:
	cargo clean
	rm -rf $(DIR)

release: $(LINUX_GOALS) macos


linux: $(LINUX_GOALS)
$(LINUX_GOALS): linux-%: $(DIR)
	cargo zigbuild --release --target $(LINUX_TARGET)
	mkdir $(TMP)
	@cp -v $(call BINARY,$(LINUX_TARGET)) $(TMP)/
	tar -czf $(DIR)/$(DISTR_LINUX).tar.gz -C $(TMP) $(PRODUCT)
	rm -rf $(TMP)


macos: $(DIR)
	$(CARGOBUILD) --target aarch64-apple-darwin
	$(CARGOBUILD) --target x86_64-apple-darwin
	rm -rf $(TMP) && mkdir $(TMP) && xattr -cr $(TMP)
	lipo -create -output $(TMP)/$(PRODUCT) $(call BINARY,*-apple-darwin)
	xcrun codesign -s "$(CODESIGN_ID_APP)" --options=runtime --timestamp $(TMP)/$(PRODUCT)
	rm -f $(DIR)/$(DISTR_MACOS).dmg
	hdiutil create -fs HFS+ -volname $(DISTR_MACOS) -srcfolder $(TMP) $(DIR)/$(DISTR_MACOS).dmg
	rm -rf $(TMP)
	xcrun codesign -s "$(CODESIGN_ID_APP)" $(DIR)/$(DISTR_MACOS).dmg
	xcrun notarytool submit $(DIR)/$(DISTR_MACOS).dmg --keychain-profile "personal" --wait
	xcrun stapler staple $(DIR)/$(DISTR_MACOS).dmg
