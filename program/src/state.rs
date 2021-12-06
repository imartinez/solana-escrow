use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};

pub struct LuckySol {
    pub is_initialized: bool,
    pub initializer_pubkey: Pubkey,
    pub initializer_deposit_account_pubkey: Pubkey,
    pub deposit_amount: u64,
}

impl Sealed for LuckySol {}

impl IsInitialized for LuckySol {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Pack for LuckySol {
    const LEN: usize = 73;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, LuckySol::LEN];
        let (
            is_initialized,
            initializer_pubkey,
            initializer_deposit_account_pubkey,
            deposit_amount,
        ) = array_refs![src, 1, 32, 32, 8];
        let is_initialized = match is_initialized {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };

        Ok(LuckySol {
            is_initialized,
            initializer_pubkey: Pubkey::new_from_array(*initializer_pubkey),
            initializer_deposit_account_pubkey: Pubkey::new_from_array(*initializer_deposit_account_pubkey),
            deposit_amount: u64::from_le_bytes(*deposit_amount),
        })
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, LuckySol::LEN];
        let (
            is_initialized_dst,
            initializer_pubkey_dst,
            initializer_deposit_account_pubkey_dst,
            deposit_amount_dst,
        ) = mut_array_refs![dst, 1, 32, 32, 8];

        let LuckySol {
            is_initialized,
            initializer_pubkey,
            initializer_deposit_account_pubkey,
            deposit_amount,
        } = self;

        is_initialized_dst[0] = *is_initialized as u8;
        initializer_pubkey_dst.copy_from_slice(initializer_pubkey.as_ref());
        initializer_deposit_account_pubkey_dst.copy_from_slice(initializer_deposit_account_pubkey.as_ref());
        *deposit_amount_dst = deposit_amount.to_le_bytes();
    }
}
