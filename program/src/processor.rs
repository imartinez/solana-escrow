use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
    clock::Clock,
    sysvar::{rent::Rent, Sysvar},
};

use crate::{error::LuckySolError, instruction::LuckySolInstruction, state::LuckySol};

const ADMIN_ACCOUNT_PUBKEY: &str = "2nBdDWiHtr6cyEdM7YeSgh9KQhLTnJYCHgUaYtf63V9Q";
const FUNDS_ACCOUNT_PUBKEY: &str = "5vHfcNh9oZW6wochBLf97ZpXekqxQo3kJywUHyAtb1X1";

pub struct Processor;
impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = LuckySolInstruction::unpack(instruction_data)?;

        match instruction {
            LuckySolInstruction::PlayLuckySol { bid } => {
                msg!("Instruction: PlayLuckySol");
                Self::process_play_lucky_sol(accounts, bid, program_id)
            }
            LuckySolInstruction::AdminWithdrawLuckySol { amount } => {
                msg!("Instruction: AdminWithdrawLuckySol");
                Self::process_admin_withdraw_lucky_sol(accounts, amount, program_id)
            }
            LuckySolInstruction::PlayerWithdrawLuckySol { } => {
                msg!("Instruction: PlayerWithdrawLuckySol");
                Self::process_player_withdraw_lucky_sol(accounts, program_id)
            }
        }
    }

    fn process_play_lucky_sol(
        accounts: &[AccountInfo],
        bid: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let initializer = next_account_info(account_info_iter)?;

        if !initializer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let deposit_account = next_account_info(account_info_iter)?;

        let data_account = next_account_info(account_info_iter)?;

        let funds_account = next_account_info(account_info_iter)?;        

        // Check accounts are owned by program
        if *deposit_account.owner != *program_id {
            return Err(ProgramError::InvalidAccountData);
        }

        if *data_account.owner != *program_id {
            return Err(ProgramError::InvalidAccountData);
        }

        if *funds_account.owner != *program_id {
            return Err(ProgramError::InvalidAccountData);
        }

        // Check FUNDS account is the expected one
        if &funds_account.key.to_string() != FUNDS_ACCOUNT_PUBKEY {
            return Err(ProgramError::InvalidAccountData);   
        }

        // Check the bid matches the Deposit exactly
        if bid != deposit_account.lamports() {
            return Err(LuckySolError::ExpectedAmountMismatch.into());
        }        

        let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;

        if !rent.is_exempt(data_account.lamports(), data_account.data_len()) {
            return Err(LuckySolError::NotRentExempt.into());
        }

        let mut lucky_sol_data = LuckySol::unpack_unchecked(&data_account.try_borrow_data()?)?;
        if lucky_sol_data.is_initialized() {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        // Play random game
        let clock = Clock::get()?;
        let rnd1 = clock.unix_timestamp % 10;
        msg!(&rnd1.to_string());

        let rent = Rent::get()?;
        let rnd2 = rent.lamports_per_byte_year % 10;
        msg!(&rnd2.to_string());
        msg!(&rent.lamports_per_byte_year.to_string());

        // Transfer deposit to Fund account and close Deposit account
        msg!("Player lost, sending the deposit to the House Fund account...");
        **funds_account.try_borrow_mut_lamports()? = funds_account
            .lamports()
            .checked_add(deposit_account.lamports())
            .ok_or(LuckySolError::AmountOverflow)?;
        **deposit_account.try_borrow_mut_lamports()? = 0;

        // Store game data
        if rnd1 > 4 {
            lucky_sol_data.won = true;    
        } else {
            lucky_sol_data.won = false;    
        }
        lucky_sol_data.is_initialized = true;
        lucky_sol_data.initializer_pubkey = *initializer.key;
        lucky_sol_data.bid_amount = bid;

        LuckySol::pack(lucky_sol_data, &mut data_account.try_borrow_mut_data()?)?;

        Ok(())
    }

    fn process_admin_withdraw_lucky_sol(
        accounts: &[AccountInfo],
        amount: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let admin_account = next_account_info(account_info_iter)?;

        if !admin_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }        

        let funds_account = next_account_info(account_info_iter)?;

        // Check accounts are owned by program
        if *funds_account.owner != *program_id {
            return Err(ProgramError::InvalidAccountData);
        }

        // Check FUNDS account is the expected one
        if &funds_account.key.to_string() != FUNDS_ACCOUNT_PUBKEY {
            return Err(ProgramError::InvalidAccountData);   
        }

        // Check Admin account is the expected one
        if &admin_account.key.to_string() != ADMIN_ACCOUNT_PUBKEY {
            return Err(ProgramError::InvalidAccountData);   
        }

        // TODO implement safe checks of both accounts

        msg!("Transfering SOL from Funds to Admin account...");
        **funds_account.try_borrow_mut_lamports()? -= amount;            
        **admin_account.try_borrow_mut_lamports()? += amount;

        Ok(())
    }

    fn process_player_withdraw_lucky_sol(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let initializer = next_account_info(account_info_iter)?;

        if !initializer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }        

        let data_account = next_account_info(account_info_iter)?;

        let funds_account = next_account_info(account_info_iter)?;

        // Check FUNDS account is the expected one
        if &funds_account.key.to_string() != FUNDS_ACCOUNT_PUBKEY {
            return Err(ProgramError::InvalidAccountData);   
        }

        // Check accounts are owned by program
        if *data_account.owner != *program_id {
            return Err(ProgramError::InvalidAccountData);
        }

        if *funds_account.owner != *program_id {
            return Err(ProgramError::InvalidAccountData);
        }

        let lucky_sol_data = LuckySol::unpack(&data_account.try_borrow_data()?)?;

        // Check that the sent data account contains the initializer
        if lucky_sol_data.initializer_pubkey != *initializer.key {
            return Err(ProgramError::InvalidAccountData);
        }

        // TODO Implement safe check as anybody could create a fake Data account 

        // Manage win
        if lucky_sol_data.won {
            msg!("Player won, sending the prize to the player account...");
            // Send player same won amount (double the bid), from House Fund
            **funds_account.try_borrow_mut_lamports()? -= 2 * lucky_sol_data.bid_amount;            
            **initializer.try_borrow_mut_lamports()? += 2 * lucky_sol_data.bid_amount;
        } 
        
        msg!("Sending Data account rent fee back to player...");
        // Send player Data rent, and close Data account
        **initializer.try_borrow_mut_lamports()? = initializer
            .lamports()
            .checked_add(data_account.lamports())
            .ok_or(LuckySolError::AmountOverflow)?;
        **data_account.try_borrow_mut_lamports()? = 0;
        *data_account.try_borrow_mut_data()? = &mut [];

        Ok(())
    }

}
