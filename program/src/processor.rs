use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
    sysvar::{rent::Rent, Sysvar},
};

use spl_token::state::Account as TokenAccount;

use crate::{error::LuckySolError, instruction::LuckySolInstruction, state::LuckySol};

pub struct Processor;
impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = LuckySolInstruction::unpack(instruction_data)?;

        match instruction {
            LuckySolInstruction::InitLuckySol { bid } => {
                msg!("Instruction: InitLuckySol");
                Self::process_init_lucky_sol(accounts, bid, program_id)
            }
            LuckySolInstruction::CancelLuckySol { } => {
                msg!("Instruction: CancelLuckySol");
                Self::process_cancel_lucky_sol(accounts, program_id)
            }
            LuckySolInstruction::Exchange { bid } => {
                msg!("Instruction: Exchange");
                Self::process_exchange(accounts, bid, program_id)
            }
        }
    }

    fn process_init_lucky_sol(
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
        let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;

        if !rent.is_exempt(data_account.lamports(), data_account.data_len()) {
            return Err(LuckySolError::NotRentExempt.into());
        }

        let mut lucky_sol_data = LuckySol::unpack_unchecked(&data_account.try_borrow_data()?)?;
        if lucky_sol_data.is_initialized() {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        lucky_sol_data.is_initialized = true;
        lucky_sol_data.initializer_pubkey = *initializer.key;
        lucky_sol_data.initializer_deposit_account_pubkey = *deposit_account.key;
        lucky_sol_data.deposit_amount = bid;

        LuckySol::pack(lucky_sol_data, &mut data_account.try_borrow_mut_data()?)?;

        Ok(())
    }

    fn process_cancel_lucky_sol(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let initializer = next_account_info(account_info_iter)?;

        if !initializer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }        

        let deposit_account = next_account_info(account_info_iter)?;

        let data_account = next_account_info(account_info_iter)?;

        let lucky_sol_data = LuckySol::unpack(&data_account.try_borrow_data()?)?;

        // Check that the sent data account contains the deposit
        if lucky_sol_data.initializer_deposit_account_pubkey != *deposit_account.key {
            return Err(ProgramError::InvalidAccountData);
        }

        // Check that the sent data account contains the initializer
        if lucky_sol_data.initializer_pubkey != *initializer.key {
            return Err(ProgramError::InvalidAccountData);
        }

        msg!("Cancelling, sending deposit and rent fee back to player...");
        **initializer.try_borrow_mut_lamports()? = initializer
            .lamports()
            .checked_add(data_account.lamports())
            .ok_or(LuckySolError::AmountOverflow)?;
        **data_account.try_borrow_mut_lamports()? = 0;
        *data_account.try_borrow_mut_data()? = &mut [];

        **initializer.try_borrow_mut_lamports()? = initializer
            .lamports()
            .checked_add(deposit_account.lamports())
            .ok_or(LuckySolError::AmountOverflow)?;
        **deposit_account.try_borrow_mut_lamports()? = 0;

        Ok(())
    }

    fn process_exchange(
        accounts: &[AccountInfo],
        amount_expected_by_taker: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        /*let account_info_iter = &mut accounts.iter();
        let taker = next_account_info(account_info_iter)?;

        if !taker.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let takers_sending_token_account = next_account_info(account_info_iter)?;

        let takers_token_to_receive_account = next_account_info(account_info_iter)?;

        let pdas_temp_token_account = next_account_info(account_info_iter)?;
        let pdas_temp_token_account_info =
            TokenAccount::unpack(&pdas_temp_token_account.try_borrow_data()?)?;
        let (pda, nonce) = Pubkey::find_program_address(&[b"escrow"], program_id);

        if amount_expected_by_taker != pdas_temp_token_account_info.amount {
            return Err(LuckySolError::ExpectedAmountMismatch.into());
        }

        let initializers_main_account = next_account_info(account_info_iter)?;
        let initializers_token_to_receive_account = next_account_info(account_info_iter)?;
        let data_account = next_account_info(account_info_iter)?;

        let lucky_sol_data = LuckySol::unpack(&data_account.try_borrow_data()?)?;

        if lucky_sol_data.temp_token_account_pubkey != *pdas_temp_token_account.key {
            return Err(ProgramError::InvalidAccountData);
        }

        if lucky_sol_data.initializer_pubkey != *initializers_main_account.key {
            return Err(ProgramError::InvalidAccountData);
        }

        if lucky_sol_data.initializer_token_to_receive_account_pubkey
            != *initializers_token_to_receive_account.key
        {
            return Err(ProgramError::InvalidAccountData);
        }

        let token_program = next_account_info(account_info_iter)?;

        let transfer_to_initializer_ix = spl_token::instruction::transfer(
            token_program.key,
            takers_sending_token_account.key,
            initializers_token_to_receive_account.key,
            taker.key,
            &[&taker.key],
            lucky_sol_data.expected_amount,
        )?;
        msg!("Calling the token program to transfer tokens to the escrow's initializer...");
        invoke(
            &transfer_to_initializer_ix,
            &[
                takers_sending_token_account.clone(),
                initializers_token_to_receive_account.clone(),
                taker.clone(),
                token_program.clone(),
            ],
        )?;

        let pda_account = next_account_info(account_info_iter)?;

        let transfer_to_taker_ix = spl_token::instruction::transfer(
            token_program.key,
            pdas_temp_token_account.key,
            takers_token_to_receive_account.key,
            &pda,
            &[&pda],
            pdas_temp_token_account_info.amount,
        )?;
        msg!("Calling the token program to transfer tokens to the taker...");
        invoke_signed(
            &transfer_to_taker_ix,
            &[
                pdas_temp_token_account.clone(),
                takers_token_to_receive_account.clone(),
                pda_account.clone(),
                token_program.clone(),
            ],
            &[&[&b"escrow"[..], &[nonce]]],
        )?;

        let close_pdas_temp_acc_ix = spl_token::instruction::close_account(
            token_program.key,
            pdas_temp_token_account.key,
            initializers_main_account.key,
            &pda,
            &[&pda],
        )?;
        msg!("Calling the token program to close pda's temp account...");
        invoke_signed(
            &close_pdas_temp_acc_ix,
            &[
                pdas_temp_token_account.clone(),
                initializers_main_account.clone(),
                pda_account.clone(),
                token_program.clone(),
            ],
            &[&[&b"escrow"[..], &[nonce]]],
        )?;

        msg!("Closing the escrow account...");
        **initializers_main_account.try_borrow_mut_lamports()? = initializers_main_account
            .lamports()
            .checked_add(data_account.lamports())
            .ok_or(LuckySolError::AmountOverflow)?;
        **data_account.try_borrow_mut_lamports()? = 0;
        *data_account.try_borrow_mut_data()? = &mut [];*/

        Ok(())
    }
}
