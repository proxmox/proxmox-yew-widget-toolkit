PKG_VER != dpkg-parsechangelog -l ${PWD}/debian/changelog -SVersion | sed -e 's/-.*//'
MACRO_PKG_VER != dpkg-parsechangelog -l ${PWD}/pwt-macros/debian/changelog -SVersion | sed -e 's/-.*//'

all:
	cargo build --target wasm32-unknown-unknown

.PHONY: deb
deb:
	rm -rf build
	mkdir build
	echo system >build/rust-toolchain
	debcargo package --config "${PWD}/pwt-macros/debian/debcargo.toml" --changelog-ready --no-overlay-write-back --directory "${PWD}/build/pwt-macros" "pwt-macros" "${MACRO_PKG_VER}"
	cd build/pwt-macros; dpkg-buildpackage -b -uc -us
	dpkg -i build/librust-pwt-macros-dev_*_amd64.deb
	debcargo package --config "${PWD}/debian/debcargo.toml" --changelog-ready --no-overlay-write-back --directory "${PWD}/build/pwt" "pwt" "${PKG_VER}"
	cd build/pwt; dpkg-buildpackage -b -uc -us


.PHONY: check
check:
	cargo test

.PHONY: clean
clean:
	cargo clean
	rm -rf build Cargo.lock
	find . -name '*~' -exec rm {} ';'
