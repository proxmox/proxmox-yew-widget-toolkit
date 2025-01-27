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

# always re-create this dir
.PHONY: build
build:
	rm -rf $(BUILDDIR)
	mkdir $(BUILDDIR)
	echo system >$(BUILDDIR)/rust-toolchain
	rm -f pwt-macros/debian/control
	debcargo package \
	  --config "${PWD}/pwt-macros/debian/debcargo.toml" \
	  --changelog-ready --no-overlay-write-back \
	  --directory "${PWD}/$(BUILDDIR)/pwt-macros" \
	  "pwt-macros" "${MACRO_PKG_VER}"
	cp $(BUILDDIR)/pwt-macros/debian/control pwt-macros/debian/control
	echo "3.0 (native)" >  build/pwt-macros/debian/source/format
	rm build/rust-pwt-macros_*.orig.tar.gz
	cd build/pwt-macros; dpkg-buildpackage -S -us -uc -d
	rm -f debian/control
	debcargo package \
	  --config "${PWD}/debian/debcargo.toml" \
	  --changelog-ready --no-overlay-write-back \
	  --directory "${PWD}/$(BUILDDIR)/pwt" "pwt" "${PKG_VER}"
	cp $(BUILDDIR)/pwt/debian/control debian/control
	echo "3.0 (native)" >  build/pwt/debian/source/format
	rm build/rust-pwt_*.orig.tar.gz
	cd build/pwt; dpkg-buildpackage -S -us -uc -d

.PHONY: dsc
dsc: build

.PHONY: deb
deb: build
	cd $(BUILDDIR)/pwt-macros; dpkg-buildpackage -b -uc -us
	# Please install librust-pwt-macros-dev: dpkg -i build/librust-pwt-macros-dev_*_amd64.deb
	cd $(BUILDDIR)/pwt; dpkg-buildpackage -b -uc -us
	lintian ${BUILD_DEBS}

upload: UPLOAD_DIST ?= $(DEB_DISTRIBUTION)
upload: $(BUILD_DEBS)
	cd $(BUILDDIR); tar cf - $(DEBS) | ssh -X repoman@repo.proxmox.com -- upload --product devel --dist $(UPLOAD_DIST)

.PHONY: check
check:
	cargo test

.PHONY: clean
clean:
	cargo clean
	rm -rf $(BUILDDIR) Cargo.lock
	find . -name '*~' -exec rm {} ';'
