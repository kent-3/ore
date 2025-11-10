# ORE Protocol Operations Guide

## Overview

ORE is a Solana-based crypto mining protocol that uses a game-theoretic approach. Miners deploy SOL to a 5x5 grid of "squares" each round, and winners are determined by randomness from Solana's slot hashes. The protocol mints ORE tokens as rewards and allows staking for yield.

## Architecture

### Core Concepts

- **Rounds**: Time-bounded mining periods (~150 slots or ~1 minute)
- **Board**: A 5x5 grid (25 squares) where miners deploy SOL
- **Winning Square**: Randomly selected each round using verifiable randomness
- **Treasury**: Holds ORE tokens and manages minting/distribution
- **Staking**: Users can stake ORE tokens to earn yield from protocol fees

### Token Economics

- **Token**: ORE (mint: `oreoU2P8bN6jkk3jbaiVxYnG1dCXcYxwhwyK9jSybcp`)
- **Decimals**: 11 (100 billion indivisible units per ORE, called "grams")
- **Max Supply**: 5,000,000 ORE
- **Mining Reward**: +1 ORE per round (winner takes all or split)
- **Motherlode**: +0.2 ORE per round accumulates, with 1/625 chance to trigger payout

### Program Accounts (PDAs)

- `Board` - Tracks current round, start/end slots (seed: `b"board"`)
- `Round(id)` - Per-round state (seed: `b"round" + round_id`)
- `Config` - Global settings, admin address (seed: `b"config"`)
- `Treasury` - Holds tokens, tracks rewards (seed: `b"treasury"`)
- `Miner(authority)` - Per-user mining state (seed: `b"miner" + user_pubkey`)
- `Stake(authority)` - Per-user staking balance (seed: `b"stake" + user_pubkey`)
- `Automation(authority)` - Bot automation config (seed: `b"automation" + user_pubkey`)

## Setup & Initialization

### 1. Deploy Program (Admin Only)

```bash
# Deploy to devnet
solana config set --url devnet
solana program deploy --program-id <keypair.json> target/deploy/ore.so

# The program ID must match the `declare_id!()` in api/src/lib.rs
```

### 2. Initialize Program (Admin Only)

The program requires initialization to create the global accounts. This is typically done once by the admin address (`HBUh9g46wk2X89CvaNN15UmsznP59rh6od1h8JwYAopk`).

**Required Accounts:**
- Config (PDA)
- Board (PDA)
- Treasury (PDA)
- Treasury token account (ATA for ORE mint)
- Entropy Var account (for randomness)

**Admin Instructions:**
- `NewVar` - Create entropy variable for randomness
- `SetAdmin` - Update admin authority
- `SetFeeCollector` - Set fee collection address
- `SetSwapProgram` - Configure swap program for buy-and-bury
- `SetVarAddress` - Set entropy variable address

## Mining Operations

### How Mining Works

1. **Deploy**: Miners deposit SOL to one or more squares on the board
2. **Round End**: After ~150 slots, a winning square is selected via VRF
3. **Checkpoint**: Miners must checkpoint to calculate their rewards
4. **Claim**: Miners claim their SOL winnings and ORE rewards

### Mining Instructions

#### Deploy (`Deploy`)

Deploys SOL to squares on the current round's board.

**Arguments:**
- `amount: u64` - Lamports to deploy per square
- `squares: u32` - Bitmask of which squares (0-24) to deploy to

**Accounts:**
- Signer (fee payer)
- Authority (miner owner)
- Automation (optional, for bot execution)
- Board
- Miner (creates if needed)
- Round
- System program

**Rules:**
- Round must be active (between start_slot and end_slot)
- First deploy of a round sets the round duration
- Miners pay a one-time checkpoint fee (0.00001 SOL)
- Can only deploy to each square once per round

**Example:**
```rust
// Deploy 0.1 SOL to squares 0, 1, 5, 12
let amount = 100_000_000; // lamports
let mask = (1 << 0) | (1 << 1) | (1 << 5) | (1 << 12);
```

#### Checkpoint (`Checkpoint`)

Calculates and credits rewards from a completed round to a miner's account.

**Accounts:**
- Signer
- Board
- Miner
- Round (from miner's last round)
- Treasury
- System program

**Rewards Calculation:**
- **SOL Rewards**: Original deployment (minus 1% fee) + proportional share of losing squares
- **ORE Rewards**: 
  - Top miner: +1 ORE (if single winner mode)
  - Split mode: +1 ORE divided proportionally among all winners
  - Motherlode: 1/625 chance to win accumulated pool
- **Bot Fee**: If round expires in <12h, bots can checkpoint and earn fee

**Rules:**
- Can only checkpoint after round is reset
- Must checkpoint before round expires (~1 day after reset)
- Rewards are credited to Miner account, not transferred yet

#### Claim SOL (`ClaimSOL`)

Transfers accumulated SOL rewards to the miner's wallet.

**Accounts:**
- Signer (miner authority)
- Miner
- System program

**Notes:**
- Transfers all `rewards_sol` from Miner account to signer
- Updates `last_claim_sol_at` timestamp

#### Claim ORE (`ClaimORE`)

Transfers accumulated ORE rewards to the miner's wallet.

**Accounts:**
- Signer (miner authority)
- Miner
- Mint (ORE token)
- Recipient (signer's ATA, created if needed)
- Treasury
- Treasury tokens
- System program
- Token program
- Associated token program

**Notes:**
- Transfers `rewards_ore` + `refined_ore` from treasury to recipient
- 10% claim fee distributed to other unclaimed miners (anti-hoarding)
- Updates treasury's `total_unclaimed` and `total_refined`

#### Reset (`Reset`)

Ends the current round, selects winners, mints rewards, and starts next round.

**Accounts:**
- Signer (anyone can call)
- Board
- Config
- Fee collector
- Mint
- Round (current)
- Round (next, creates new)
- Top miner (TODO: currently not validated)
- Treasury
- Treasury tokens
- System program
- Token program
- Program (self)
- Slot hashes sysvar
- Entropy Var account
- Entropy program

**Process:**
1. Sample randomness from entropy Var account
2. Determine winning square from RNG
3. Calculate winnings distribution (90% to winners, 10% to vault, 1% admin fee)
4. Mint +1 ORE for winners
5. Mint +0.2 ORE to motherlode pool
6. Check if motherlode triggered (1/625 odds)
7. Emit reset event with results
8. Increment round, create next Round account

**Rules:**
- Can only reset after `end_slot + INTERMISSION_SLOTS` (35 slots cooldown)
- If no deployment, refunds all SOL
- If no one on winning square, vault all deployed SOL

### Automation

Miners can set up bots to auto-deploy each round.

#### Automate (`Automate`)

Creates or updates an automation configuration.

**Arguments:**
- `amount: u64` - SOL to deploy per square
- `deposit: u64` - SOL to add to automation balance
- `fee: u64` - Fee to pay executor per execution
- `mask: u64` - Strategy parameter (see below)
- `strategy: u8` - 0=Random, 1=Preferred

**Strategies:**
- **Random**: Lower 8 bits of mask = number of squares to deploy to (randomly selected)
- **Preferred**: Mask bits indicate which specific squares to deploy to

**Accounts:**
- Signer (authority)
- Automation (PDA, creates if needed)
- Executor (who can execute)
- Miner
- System program

**Notes:**
- Set executor to `Pubkey::default()` to close automation account
- Automation auto-closes when balance insufficient

## Staking Operations

Stake ORE tokens to earn yield from protocol fees.

### Deposit (`Deposit`)

Deposits ORE into staking contract.

**Arguments:**
- `amount: u64` - Amount of ORE to deposit (in base units)

**Accounts:**
- Signer
- Mint
- Sender (signer's ORE token account)
- Stake (creates if needed)
- Stake tokens (stake's ATA)
- Treasury
- System/token programs

**Notes:**
- Creates Stake account on first deposit
- Tokens transferred to stake's associated token account
- Updates `total_staked` in treasury

### Withdraw (`Withdraw`)

Withdraws ORE from staking contract.

**Arguments:**
- `amount: u64` - Amount to withdraw

**Accounts:**
- Signer
- Mint
- Recipient (signer's ATA)
- Stake
- Stake tokens
- Treasury
- System/token programs

### Claim Yield (`ClaimYield`)

Claims staking rewards.

**Arguments:**
- `amount: u64` - Amount to claim (up to available rewards)

**Accounts:**
- Signer
- Mint
- Recipient
- Stake
- Treasury
- Treasury tokens
- System/token programs

**Notes:**
- Rewards calculated based on `treasury.stake_rewards_factor`
- Uses factor-based accounting for fair distribution

## Admin Operations

### Bury (`Bury`)

Executes a buy-and-burn operation using treasury SOL balance.

**Process:**
- Swaps SOL for ORE using configured swap program
- Burns acquired ORE, removing it from circulation
- Reduces total supply

### Wrap (`Wrap`)

Wraps SOL in treasury for swap transactions (WSOL).

### Administrative Settings

- `SetAdmin` - Transfer admin authority
- `SetFeeCollector` - Update fee collection address
- `SetSwapProgram` - Configure swap program
- `SetVarAddress` - Update entropy Var address
- `SetBuffer` - Update buffer parameter

## Key Constants

```rust
TOKEN_DECIMALS = 11
ONE_ORE = 100_000_000_000 // base units
MAX_SUPPLY = 5_000_000 ORE
CHECKPOINT_FEE = 10_000 lamports (0.00001 SOL)
INTERMISSION_SLOTS = 35 (~14 seconds)
ONE_DAY_SLOTS = 216_000
TWELVE_HOURS_SLOTS = 108_000
```

## Typical User Flow

### For Miners:

1. **Deploy** - Call Deploy instruction with SOL amount and square selection
2. **Wait** - Round completes after ~150 slots
3. **Reset** - Anyone calls Reset to end round (bot usually)
4. **Checkpoint** - Call Checkpoint to calculate rewards from completed round
5. **Claim** - Call ClaimSOL and ClaimORE to receive rewards
6. **Repeat** - Deploy to next round

### For Stakers:

1. **Deposit** - Stake ORE tokens
2. **Earn** - Accumulate yield from protocol fees
3. **Claim** - Periodically claim staking rewards
4. **Withdraw** - Unstake when desired

### For Automation Users:

1. **Automate** - Set up bot with strategy and deposit SOL
2. **Monitor** - Bot executes deploys automatically each round
3. **Checkpoint** - Still need to checkpoint and claim manually (or via bot)
4. **Top Up** - Add more SOL to automation balance as needed

## Security Considerations

- All PDAs are validated with seed checks
- Signer validation on all user operations
- Rent-exempt requirements enforced
- Overflow checks enabled in release builds
- Admin operations restricted to authorized addresses
- Entropy from external Var account for verifiable randomness

## Testing Locally

```bash
# Start local validator with mainnet account clones
bash localnet.sh

# This clones:
# - ORE mint and metadata
# - Treasury and config accounts
# - Entropy Var account
# - Swap program accounts
```

## Resources

- Repository: https://github.com/regolith-labs/ore
- Website: https://ore.supply
- Documentation: https://docs.rs/ore-api/latest/ore_api/
