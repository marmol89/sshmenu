BINARY := sshmenu
INSTALL_DIR ?= $(HOME)/.local/bin

.PHONY: all build release install uninstall clean test run help

all: release

build:
	cargo build

release:
	cargo build --release

install: release
	install -m 0755 target/release/$(BINARY) $(INSTALL_DIR)/$(BINARY)
	@echo "Installed $(BINARY) to $(INSTALL_DIR)/$(BINARY)"

uninstall:
	rm -f $(INSTALL_DIR)/$(BINARY)

run: build
	cargo run

test:
	cargo test

clean:
	cargo clean

help:
	@echo "Targets:"
	@echo "  build      Debug build"
	@echo "  release    Optimized build"
	@echo "  install    Build release and install to \$$(HOME)/.local/bin"
	@echo "  uninstall  Remove installed binary"
	@echo "  run        Debug build and run"
	@echo "  test       Run tests"
	@echo "  clean      Clean build artifacts"