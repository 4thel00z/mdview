.PHONY: test app install default clean

test:
	cargo test
	cargo clippy --all-targets -- -D warnings

app:
	scripts/bundle.sh

install:
	scripts/install.sh

default:
	swift scripts/set-default.swift "$${MDVIEW_INSTALL_DIR:-$$HOME/Applications}/mdview.app"

clean:
	rm -rf dist
	cargo clean
