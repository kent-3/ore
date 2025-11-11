use ore_api::prelude::*;
use steel::*;

/// Initializes the program.
pub fn process_initialize(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let [signer_info, board_info, config_info, mint_info, round_info, treasury_info, treasury_tokens_info, system_program, token_program, associated_token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?.has_address(&ADMIN_ADDRESS)?;
    board_info.has_seeds(&[BOARD], &ore_api::ID)?;
    config_info.has_seeds(&[CONFIG], &ore_api::ID)?;
    mint_info.has_address(&MINT_ADDRESS)?.as_mint()?;
    round_info.has_seeds(&[ROUND, &0u64.to_le_bytes()], &ore_api::ID)?;
    treasury_info.has_seeds(&[TREASURY], &ore_api::ID)?;
    treasury_tokens_info.has_address(&treasury_tokens_address())?;
    system_program.is_program(&system_program::ID)?;
    token_program.is_program(&spl_token::ID)?;
    associated_token_program.is_program(&spl_associated_token_account::ID)?;

    // Create board account.
    if board_info.data_is_empty() {
        create_program_account::<Board>(
            board_info,
            system_program,
            signer_info,
            &ore_api::ID,
            &[BOARD],
        )?;
        let board = board_info.as_account_mut::<Board>(&ore_api::ID)?;
        board.round_id = 0;
        board.start_slot = 0;
        board.end_slot = u64::MAX;
    } else {
        board_info.as_account::<Board>(&ore_api::ID)?;
    }

    // Create config account.
    if config_info.data_is_empty() {
        create_program_account::<Config>(
            config_info,
            system_program,
            signer_info,
            &ore_api::ID,
            &[CONFIG],
        )?;
        let config = config_info.as_account_mut::<Config>(&ore_api::ID)?;
        config.admin = *signer_info.key;
        config.bury_authority = *signer_info.key;
        config.fee_collector = *signer_info.key;
        config.swap_program = Pubkey::default();
        config.var_address = Pubkey::default();
        config.buffer = 0;
    } else {
        config_info.as_account::<Config>(&ore_api::ID)?;
    }

    // Create round 0 account.
    if round_info.data_is_empty() {
        create_program_account::<Round>(
            round_info,
            system_program,
            signer_info,
            &ore_api::ID,
            &[ROUND, &0u64.to_le_bytes()],
        )?;
        let round = round_info.as_account_mut::<Round>(&ore_api::ID)?;
        // All fields default to 0 except:
        round.expires_at = u64::MAX; // Signal: waiting for first deploy
        round.rent_payer = *signer_info.key;
    } else {
        round_info.as_account::<Round>(&ore_api::ID)?;
    }

    // Create treasury account.
    if treasury_info.data_is_empty() {
        create_program_account::<Treasury>(
            treasury_info,
            system_program,
            signer_info,
            &ore_api::ID,
            &[TREASURY],
        )?;
        let treasury = treasury_info.as_account_mut::<Treasury>(&ore_api::ID)?;
        treasury.balance = 0;
    } else {
        treasury_info.as_account::<Treasury>(&ore_api::ID)?;
    }

    // Initialize vault token account.
    if treasury_tokens_info.data_is_empty() {
        create_associated_token_account(
            signer_info,
            treasury_info,
            treasury_tokens_info,
            mint_info,
            system_program,
            token_program,
            associated_token_program,
        )?;
    } else {
        treasury_tokens_info.as_associated_token_account(treasury_info.key, mint_info.key)?;
    }

    Ok(())
}
