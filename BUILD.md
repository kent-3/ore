# Building the ORE Program

This workspace contains multiple packages:
- `program/` - The on-chain Solana program (BPF)
- `api/` - Shared types and utilities
- `cli/` - Command-line interface (runs locally)

## Building the Solana Program

The Solana program must be built for the BPF target. Only the `program` package should be built with `cargo build-sbf`.

### Option 1: Build from the program directory (recommended)
```bash
cd program
cargo build-sbf
```

### Option 2: Build from workspace root with package filter
```bash
cargo build-sbf -- --package ore-program
```

### Common Mistake ❌
**Don't run** `cargo build-sbf` from the workspace root without specifying a package:
```bash
cargo build-sbf  # ❌ This tries to build all workspace members including CLI
```

This will fail because it tries to build the `cli` package for BPF, which has dependencies like `getrandom` and `socket2` that aren't compatible with the BPF target.

## Building Other Packages

The API and CLI packages are regular Rust code:
```bash
cargo build          # Build all packages for your local machine
cargo check          # Fast check without building
cargo test           # Run tests
```

## Solana Version Requirement

This project requires **Solana CLI 3.0.10** to build. If you have multiple Solana versions installed, use the version switcher:

```bash
./switch-solana.sh ore    # Switch to Solana 3.0.10
cd program
cargo build-sbf           # Build the program
```

See `SOLANA_VERSION_SWITCHING.md` for more details about managing multiple Solana versions.

## Testing

```bash
# Test the program
cd program
cargo test-sbf

# Test other packages
cargo test --package ore-api
cargo test --package ore-cli
```

## Output

The compiled program will be at:
```
target/deploy/ore.so
```
