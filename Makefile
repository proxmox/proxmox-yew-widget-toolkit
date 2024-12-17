include /usr/share/dpkg/pkg-info.mk

PKG_VER != dpkg-parsechangelog -l ${PWD}/debian/changelog -SVersion | sed -e 's/-.*//'
MACRO_PKG_VER != dpkg-parsechangelog -l ${PWD}/pwt-macros/debian/changelog -SVersion | sed -e 's/-.*//'

BUILDDIR?=build

PWT_DEB=librust-pwt-dev_$(PKG_VER)_amd64.deb
PWT_MACROS_DEB=librust-pwt-macros-dev_$(MACRO_PKG_VER)_amd64.deb

DEBS=$(PWT_DEB) $(PWT_MACROS_DEB)
BUILD_DEBS=$(addprefix $(BUILDDIR)/,$(DEBS))

all:
	cargo build --target wasm32-unknown-unknown

$(BUILD_DEBS): deb

.PHONY: deb
deb:
	rm -rf $(BUILDDIR)
	mkdir $(BUILDDIR)
	echo system >build/rust-toolchain
	rm -f pwt-macros/debian/control
	debcargo package \
	  --config "${PWD}/pwt-macros/debian/debcargo.toml" \
	  --changelog-ready --no-overlay-write-back \
	  --directory "${PWD}/$(BUILDDIR)/pwt-macros" \
	  "pwt-macros" "${MACRO_PKG_VER}"
	cd $(BUILDDIR)/pwt-macros; dpkg-buildpackage -b -uc -us
	cp $(BUILDDIR)/pwt-macros/debian/control pwt-macros/debian/control
	# Please install librust-pwt-macros-dev: dpkg -i build/librust-pwt-macros-dev_*_amd64.deb
	rm -f debian/control
	debcargo package \
	  --config "${PWD}/debian/debcargo.toml" \
	  --changelog-ready --no-overlay-write-back \
	  --directory "${PWD}/$(BUILDDIR)/pwt" "pwt" "${PKG_VER}"
	cd $(BUILDDIR)/pwt; dpkg-buildpackage -b -uc -us
	cp $(BUILDDIR)/pwt/debian/control debian/control

upload: UPLOAD_DIST ?= $(DEB_DISTRIBUTION)
upload: $(BUILD_DEBS)
	(cd $(BUILDDIR); \
	  tar cf - $(DEBS) | ssh -X repoman@repo.proxmox.com -- upload --product devel --dist $(UPLOAD_DIST) \
	)

.PHONY: check
check:
	cargo test

.PHONY: clean
clean:
	cargo clean
	rm -rf build Cargo.lock
	find . -name '*~' -exec rm {} ';'
