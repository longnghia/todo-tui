BINARY_NAME := todo
INSTALL_PATH := /usr/local/bin

all: build install

run:
	cargo run

build:
	@echo "Building release binary..."
	cargo build --release

install:
	@echo "Installing binary to $(INSTALL_PATH)..."
	@sudo cp target/release/$(BINARY_NAME) $(INSTALL_PATH)

clean:
	cargo clean

uninstall:
	@echo "Uninstalling binary from $(INSTALL_PATH)..."
	@sudo rm -f $(INSTALL_PATH)/$(BINARY_NAME)

.PHONY: all build release install clean uninstall
