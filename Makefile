SHELL := /bin/bash
export PATH := $(HOME)/.cargo/bin:$(HOME)/.local/share/solana/install/active_release/bin:$(PATH)

.PHONY: all build test check clean help \
        blockchain-build blockchain-deploy blockchain-test blockchain-test-unit \
        server-run server-build server-test \
        client-dev client-build client-preview

all: build test

help:
	@echo "Available commands from root workspace:"
	@echo ""
	@echo "  make build                Build all Cargo workspace crates (blockchain, server, common)"
	@echo "  make test                 Run all workspace unit & integration tests"
	@echo "  make check                Check compilation across the entire workspace"
	@echo "  make clean                Clean build artifacts"
	@echo ""
	@echo "Blockchain:"
	@echo "  make blockchain-build     Build Anchor program (bounty_board)"
	@echo "  make blockchain-deploy    Deploy program to Solana Devnet"
	@echo "  make blockchain-test      Run Anchor program integration tests via Anchor CLI"
	@echo "  make blockchain-test-unit Run Anchor program unit/SVM tests with Cargo"
	@echo ""
	@echo "Server (Backend):"
	@echo "  make server-run           Run the server binary"
	@echo "  make server-build         Build the server binary"
	@echo "  make server-test          Run server tests"
	@echo ""
	@echo "Client (Vite + React):"
	@echo "  make client-dev           Start Vite dev server"
	@echo "  make client-build         Production build"
	@echo "  make client-preview       Preview production build"

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

blockchain-deploy:
	@test -e blockchain/target || ln -s ../target blockchain/target
	PATH="$(HOME)/.cargo/bin:$(HOME)/.local/share/solana/install/active_release/bin:$$PATH" solana config set --url devnet
	PATH="$(HOME)/.cargo/bin:$(HOME)/.local/share/solana/install/active_release/bin:$$PATH" solana program deploy target/deploy/bounty_board.so --program-id target/deploy/bounty_board-keypair.json --url devnet

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

client-dev:
	npm --prefix client run dev

client-build:
	npm --prefix client run build

client-preview:
	npm --prefix client run preview
