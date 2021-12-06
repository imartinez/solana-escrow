use solana_program::program_error::ProgramError;
use std::convert::TryInto;

use crate::error::LuckySolError::InvalidInstruction;

pub enum LuckySolInstruction {
    /// Plays the game and saves the game bid and result data in a Data account
    /// Move deposit amount to Fund account and close Deposit account
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person initializing the game
    /// 1. `[]` Deposit account that should be created and funded prior to this instruction. Owned by the program
    /// 2. `[writable]` The Data account, it will hold all necessary info about the game. Owned by the program
    /// 3. `[writable]` The Funds account. Owned by the program
    /// 4. `[]` The rent sysvar    
    PlayLuckySol {
        /// The bid amount the player is Bidding
        bid: u64,
    },
    /// Admin withdraw of a given quantity from the House Fund
    ///
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` The Admin account, where amount will be sent. Must be the signer
    /// 1. `[writable]` The Funds account, from where amount is withdrawn. Owned by the program    
    AdminWithdrawLuckySol {
        /// The amount being withdrawn
        amount: u64,
    },
    /// Withdraw to the player. If the player won, it will send the player twice the bid amount
    /// from the Funds account
    /// Also closes the Data account and sends the player the rent fees
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer, writable]` The account of the person initializing the game    
    /// 1. `[writable]` The Data account, holding the necessary info about the game. Owned by the program
    /// 2. `[writable]` The Funds account. Owned by the program
    PlayerWithdrawLuckySol { },
}

impl LuckySolInstruction {
    /// Unpacks a byte buffer into a [EscrowInstruction](enum.EscrowInstruction.html).
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;

        Ok(match tag {
            0 => Self::PlayLuckySol {
                bid: Self::unpack_amount(rest)?,
            },
            1 => Self::AdminWithdrawLuckySol {
                amount: Self::unpack_amount(rest)?,
            },
            2 => Self::PlayerWithdrawLuckySol {},
            _ => return Err(InvalidInstruction.into()),
        })
    }

    fn unpack_amount(input: &[u8]) -> Result<u64, ProgramError> {
        let amount = input
            .get(..8)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(InvalidInstruction)?;
        Ok(amount)
    }
}
