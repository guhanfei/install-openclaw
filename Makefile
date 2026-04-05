.PHONY: help check install-deps build-mac build-win build-all clean

APP_NAME := openclaw-helper
CARGO_DIR := src-tauri
BUILD_DIR := ./builds
VERSION := $(shell node -p "require('./package.json').version")
DIST_MAC := $(CARGO_DIR)/target/aarch64-apple-darwin/release/bundle/dmg

help:
	@echo "OpenClaw Helper 打包工具"
	@echo ""
	@echo "可用命令:"
	@echo "  make check         - 检查环境工具是否齐全"
	@echo "  make install-deps  - 安装缺失的依赖（只需执行一次）"
	@echo "  make build-mac     - 打包 Mac 版本 (Apple Silicon)"
	@echo "  make build-win     - 打包 Windows 版本 (x86_64)"
	@echo "  make all           - 同时打包 Mac 和 Windows 版本"
	@echo "  make clean         - 清理构建产物和 builds 目录"

check:
	@echo "检查环境..."
	@which rustc > /dev/null && echo "✓ Rust 已安装" || echo "✗ Rust 未安装"
	@which node > /dev/null && echo "✓ Node.js 已安装" || echo "✗ Node.js 未安装"
	@rustup target list --installed | grep aarch64-apple-darwin > /dev/null && echo "✓ Mac 目标已安装" || echo "✗ Mac 目标未安装 (run: rustup target add aarch64-apple-darwin)"
	@rustup target list --installed | grep x86_64-pc-windows-gnu > /dev/null && echo "✓ Windows 目标已安装" || echo "✗ Windows 目标未安装 (run: rustup target add x86_64-pc-windows-gnu)"
	@which x86_64-w64-mingw32-gcc > /dev/null && echo "✓ MinGW-w64 已安装" || echo "✗ MinGW-w64 未安装 (run: brew install mingw-w64)"

install-deps:
	@echo "安装 Rust Windows 交叉编译目标..."
	@rustup target add x86_64-pc-windows-gnu
	@echo "安装 MinGW-w64..."
	@brew install mingw-w64
	@echo "创建 .cargo/config.toml..."
	@mkdir -p $(CARGO_DIR)/.cargo
	@printf '[target.x86_64-pc-windows-gnu]\nlinker = "x86_64-w64-mingw32-gcc"\nar = "x86_64-w64-mingw32-ar"\n' > $(CARGO_DIR)/.cargo/config.toml
	@echo "完成！"

build-mac:
	@echo "正在打包 Mac 版本..."
	@mkdir -p $(BUILD_DIR)
	@npm run tauri build -- --target aarch64-apple-darwin
	@cp $(DIST_MAC)/*.dmg $(BUILD_DIR)/install-openclaw-$(VERSION)-macos-arm64.dmg
	@echo "Mac 安装包位于: $(BUILD_DIR)/install-openclaw-$(VERSION)-macos-arm64.dmg"

build-win:
	@echo "正在打包 Windows 版本..."
	@mkdir -p $(BUILD_DIR)
	@npm run tauri build -- --target x86_64-pc-windows-gnu || true
	@cp $(CARGO_DIR)/target/x86_64-pc-windows-gnu/release/openclaw-helper.exe $(BUILD_DIR)/install-openclaw-$(VERSION)-windows-x86_64.exe
	@echo "Windows 安装包位于: $(BUILD_DIR)/install-openclaw-$(VERSION)-windows-x86_64.exe"

all: build-mac build-win

clean:
	@echo "清理构建产物..."
	@rm -rf $(CARGO_DIR)/target/aarch64-apple-darwin/release/bundle
	@rm -rf $(CARGO_DIR)/target/x86_64-pc-windows-gnu/release/bundle
	@rm -rf $(BUILD_DIR)
	@echo "完成！"
