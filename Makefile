
all:
	cargo build --target wasm32-unknown-unknown

.PHONY: clean
clean:
	cargo clean
	rm -rf Cargo.lock
	find . -name '*~' -exec rm {} ';'
