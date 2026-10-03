# Bounty Board

A decentralized bounty board application structured cleanly into modular domains with unified root orchestration.

## Repository Structure

```text
bounty_board/
├── blockchain/           # Solana & Anchor smart contracts & on-chain tests
│   ├── programs/         # Solana programs (e.g. bounty_board)
│   ├── tests/            # TypeScript integration tests for programs
│   ├── Anchor.toml       # Anchor framework configuration
│   └── tsconfig.json     # TypeScript config for anchor tests
├── client/               # Frontend (Vite + React + TypeScript)
│   ├── src/
│   ├── index.html
│   ├── package.json
│   └── vite.config.ts
├── server/               # Backend / server application
│   ├── src/
│   └── Cargo.toml
├── common/               # Shared logic & types across crates
│   ├── src/
│   └── Cargo.toml
├── Cargo.toml            # Root Cargo workspace configuration
├── Makefile              # Root task runner & quick commands
├── package.json          # Root NPM scripts & Anchor test deps
└── tsconfig.json         # Root TypeScript configuration (Anchor tests)
```

## Quick Start (Root Commands)

You can build, test, and run any component directly from the root workspace without having to navigate into specific subdirectories.

### Using Make

```bash
# Build Cargo workspace (blockchain program, server, common)
make build

# Run all unit and integration tests across the workspace
make test

# Typecheck and verify compilation across all workspace members
make check

# Run server backend
make server-run

# Run Anchor blockchain integration tests
make blockchain-test

# See all available make targets
make help
```

### Using NPM Scripts

```bash
# Workspace commands
npm run build:all          # Build all workspace crates
npm run test:all           # Run all tests
npm run check:all          # Verify compilation

# Domain-specific commands
npm run blockchain:build   # Build Anchor program
npm run blockchain:test    # Run Anchor integration tests
npm run server:run         # Start backend server
npm run server:build       # Build backend server
npm run client:dev         # Start Vite + React frontend
npm run client:build       # Production build of frontend
```

### Using Cargo

```bash
# Build any specific crate
cargo build -p server
cargo build -p bounty_board

# Run tests
cargo test --workspace
```
