# Agent Guidelines for ORE Solana Program

## Build & Test Commands
- **Build**: `cargo build-sbf` (Solana program build)
- **Test**: `cargo test-sbf` (run all tests)
- **Coverage**: `cargo llvm-cov` (line coverage report)
- **Check**: `cargo check` (verify compilation without building)
- **Format**: `cargo fmt` (auto-format code)
- **Lint**: `cargo clippy` (static analysis)

## Workspace Structure
This is a Cargo workspace with 3 members: `api/`, `program/`, `cli/`. The `api` crate defines state, instructions, errors, and SDK. The `program` crate contains the Solana BPF program logic.

## Code Style & Conventions
- **Rust version**: 1.82.0 (locked via rust-toolchain.toml)
- **Imports**: Group by `mod` declarations first, then wildcard re-exports (`pub use foo::*`), then external crates. Use `use steel::*;` for the steel framework.
- **Module structure**: Declare submodules at top, import with `use module::*;` below
- **Account validation**: Use steel's fluent validation: `account_info.is_signer()?.is_writable()?.has_seeds(...)?.as_account_mut::<T>()?.assert_mut(|x| condition)?`
- **Error handling**: Use `?` operator liberally. Custom errors defined in `api/src/error.rs` with `#[derive(Error)]` and `error!()` macro
- **PDAs**: Define helper functions like `miner_pda()`, `board_pda()` in `api/src/state/mod.rs` with pattern `Pubkey::find_program_address(&[SEED, ...], &crate::ID)`
- **Naming**: Snake_case for functions/variables, PascalCase for types/enums, SCREAMING_SNAKE for constants
- **Account structs**: Use `#[repr(C)]`, `#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable, Serialize, Deserialize)]`, and `account!(AccountType, StructName)` macro
- **Comments**: Document public items with `///` doc comments explaining purpose and behavior
- **Logging**: Use `sol_log()` from `solana_program::log::sol_log` for runtime logging

## Architecture Notes
- Uses the `steel` framework extensively for account validation and program structure
- Program ID declared via `declare_id!()` macro in `api/src/lib.rs`
- Instruction processing pattern: parse args → load/validate accounts → execute business logic → return `Ok(())`
