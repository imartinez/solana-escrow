use solana_program::program_error::ProgramError;
use std::convert::TryInto;

use crate::error::LuckySolError::InvalidInstruction;

pub enum LuckySolInstruction {
    /// Starts the game by saving the game data in a Data account
    ///
    ///
    /// Accounts expected:
    ///
    /// 0. `[]` The account of the person initializing the game
    /// 1. `[]` Deposit account that should be created and funded prior to this instruction and owned by the program
    /// 3. `[writable]` The Data account, it will hold all necessary info about the game.
    /// 4. `[]` The rent sysvar    
    InitLuckySol {
        /// The bid amount the player is Bidding
        bid: u64,
    },
    /// Cancels the game, by emptying both the Deposit account and the Data account in favour of initializer of game
    ///
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person cancelling the game
    /// 1. `[writable]` Deposit account where the deposit is, if initialized
    /// 3. `[writable]` The Data account, holding the necessary info about the game.    
    CancelLuckySol { },
    /// Accepts a trade
    ///
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person taking the trade
    /// 1. `[writable]` The taker's token account for the token they send
    /// 2. `[writable]` The taker's token account for the token they will receive should the trade go through
    /// 3. `[writable]` The PDA's temp token account to get tokens from and eventually close
    /// 4. `[writable]` The initializer's main account to send their rent fees to
    /// 5. `[writable]` The initializer's token account that will receive tokens
    /// 6. `[writable]` The escrow account holding the escrow info
    /// 7. `[]` The token program
    /// 8. `[]` The PDA account
    Exchange {
        /// the bid the taker expects to be paid in the other token, as a u64 because that's the max possible supply of a token
        bid: u64,
    },
}

impl LuckySolInstruction {
    /// Unpacks a byte buffer into a [EscrowInstruction](enum.EscrowInstruction.html).
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;

        Ok(match tag {
            0 => Self::InitLuckySol {
                bid: Self::unpack_bid(rest)?,
            },
            1 => Self::CancelLuckySol {},
            2 => Self::Exchange {
                bid: Self::unpack_bid(rest)?,
            },
            _ => return Err(InvalidInstruction.into()),
        })
    }

    fn unpack_bid(input: &[u8]) -> Result<u64, ProgramError> {
        let bid = input
            .get(..8)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(InvalidInstruction)?;
        Ok(bid)
    }
}
