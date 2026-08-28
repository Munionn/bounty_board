SHELL := /bin/bash
export PATH := $(HOME)/.cargo/bin:$(HOME)/.local/share/solana/install/active_release/bin:$(PATH)

.PHONY: all build test check clean help \
        blockchain-build blockchain-test blockchain-test-unit \
        server-run server-build server-test \
        client-build client-test

all: build test

help:
	@echo "Available commands from root workspace:"
	@echo ""
	@echo "  make build                Build all workspace components (blockchain, server, client, common)"
	@echo "  make test                 Run all workspace unit & integration tests"
	@echo "  make check                Check compilation across the entire workspace"
	@echo "  make clean                Clean build artifacts"
	@echo ""
	@echo "Blockchain:"
	@echo "  make blockchain-build     Build Anchor program (bounty_board)"
	@echo "  make blockchain-test      Run Anchor program integration tests via Anchor CLI"
	@echo "  make blockchain-test-unit Run Anchor program unit/SVM tests with Cargo"
	@echo ""
	@echo "Server (Backend):"
	@echo "  make server-run           Run the server binary"
	@echo "  make server-build         Build the server binary"
	@echo "  make server-test          Run server tests"
	@echo ""
	@echo "Client (Frontend):"
	@echo "  make client-build         Build the client library/app"
	@echo "  make client-test          Run client tests"

build:
	PATH="$(HOME)/.cargo/bin:$(HOME)/.local/share/solana/install/active_release/bin:$$PATH" cargo build --workspace

test:
	PATH="$(HOME)/.cargo/bin:$(HOME)/.local/share/solana/install/active_release/bin:$$PATH" cargo test --workspace

check:
	PATH="$(HOME)/.cargo/bin:$(HOME)/.local/share/solana/install/active_release/bin:$$PATH" cargo check --workspace

clean:
	PATH="$(HOME)/.cargo/bin:$(HOME)/.local/share/solana/install/active_release/bin:$$PATH" cargo clean
	rm -rf blockchain/target

blockchain-build:
	@test -e blockchain/target || ln -s ../target blockchain/target
	PATH="$(HOME)/.cargo/bin:$(HOME)/.local/share/solana/install/active_release/bin:$$PATH" cd blockchain && anchor build

blockchain-test:
	@test -e blockchain/target || ln -s ../target blockchain/target
	PATH="$(HOME)/.cargo/bin:$(HOME)/.local/share/solana/install/active_release/bin:$$PATH" cd blockchain && anchor test

blockchain-test-unit:
	PATH="$(HOME)/.cargo/bin:$(HOME)/.local/share/solana/install/active_release/bin:$$PATH" cargo test -p bounty_board

server-run:
	PATH="$(HOME)/.cargo/bin:$(HOME)/.local/share/solana/install/active_release/bin:$$PATH" cargo run -p server

server-build:
	PATH="$(HOME)/.cargo/bin:$(HOME)/.local/share/solana/install/active_release/bin:$$PATH" cargo build -p server

server-test:
	PATH="$(HOME)/.cargo/bin:$(HOME)/.local/share/solana/install/active_release/bin:$$PATH" cargo test -p server

client-build:
	PATH="$(HOME)/.cargo/bin:$(HOME)/.local/share/solana/install/active_release/bin:$$PATH" cargo build -p client

client-test:
	PATH="$(HOME)/.cargo/bin:$(HOME)/.local/share/solana/install/active_release/bin:$$PATH" cargo test -p client
