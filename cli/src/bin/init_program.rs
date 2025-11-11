use ore_api::prelude::*;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    signature::{read_keypair_file, Signer},
    transaction::Transaction,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rpc_url =
        std::env::var("RPC").unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());
    let keypair_path = std::env::var("KEYPAIR")
        .unwrap_or_else(|_| format!("{}/.config/solana/id.json", std::env::var("HOME").unwrap()));

    println!("ORE Program Initialization");
    println!("===========================");
    println!("RPC: {}", rpc_url);
    println!("Keypair: {}", keypair_path);

    let rpc = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());
    let payer = read_keypair_file(&keypair_path)?;

    println!("\nAdmin pubkey: {}", payer.pubkey());
    println!("Program ID: {}", ore_api::ID);

    // Calculate PDAs
    let (config_pda, config_bump) = ore_api::state::config_pda();
    let (board_pda, board_bump) = ore_api::state::board_pda();
    let (treasury_pda, treasury_bump) = ore_api::state::treasury_pda();

    println!("\nCalculated PDAs:");
    println!("  Config:   {} (bump: {})", config_pda, config_bump);
    println!("  Board:    {} (bump: {})", board_pda, board_bump);
    println!("  Treasury: {} (bump: {})", treasury_pda, treasury_bump);

    // Check which accounts exist
    println!("\nChecking existing accounts on chain...");
    let config_account = rpc.get_account(&config_pda).await;
    let board_account = rpc.get_account(&board_pda).await;
    let treasury_account = rpc.get_account(&treasury_pda).await;

    let config_exists = config_account.is_ok();
    let board_exists = board_account.is_ok();
    let treasury_exists = treasury_account.is_ok();

    println!(
        "  Config:   {}",
        if config_exists {
            "✓ EXISTS"
        } else {
            "✗ MISSING"
        }
    );
    println!(
        "  Board:    {}",
        if board_exists {
            "✓ EXISTS"
        } else {
            "✗ MISSING"
        }
    );
    println!(
        "  Treasury: {}",
        if treasury_exists {
            "✓ EXISTS"
        } else {
            "✗ MISSING"
        }
    );

    if config_exists && board_exists && treasury_exists {
        println!("\n✓ All accounts already initialized!");
        return Ok(());
    }

    // Calculate required rent
    println!("\nRequired rent for initialization:");
    let config_space = 8 + std::mem::size_of::<Config>();
    let board_space = 8 + std::mem::size_of::<Board>();
    let treasury_space = 8 + std::mem::size_of::<Treasury>();

    let config_rent = rpc
        .get_minimum_balance_for_rent_exemption(config_space)
        .await?;
    let board_rent = rpc
        .get_minimum_balance_for_rent_exemption(board_space)
        .await?;
    let treasury_rent = rpc
        .get_minimum_balance_for_rent_exemption(treasury_space)
        .await?;

    println!(
        "  Config:   {} bytes = {} lamports",
        config_space, config_rent
    );
    println!(
        "  Board:    {} bytes = {} lamports",
        board_space, board_rent
    );
    println!(
        "  Treasury: {} bytes = {} lamports",
        treasury_space, treasury_rent
    );
    println!(
        "  TOTAL:    {} lamports ({} SOL)",
        config_rent + board_rent + treasury_rent,
        (config_rent + board_rent + treasury_rent) as f64 / 1_000_000_000.0
    );

    // Create initialize instruction
    println!("\n>> Building initialize transaction...");
    let ix = ore_api::sdk::initialize(payer.pubkey());

    // Get recent blockhash
    let blockhash = rpc.get_latest_blockhash().await?;

    // Create and send transaction
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer.pubkey()), &[&payer], blockhash);

    println!(">> Sending transaction...");
    let signature = rpc.send_and_confirm_transaction(&tx).await?;

    println!("\n✓ Program initialized successfully!");
    println!("  Signature: {}", signature);

    Ok(())
}
