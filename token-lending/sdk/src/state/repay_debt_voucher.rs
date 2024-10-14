use std::convert::TryInto;

use borsh::BorshDeserialize;
use solana_program::{account_info::AccountInfo, msg, program_error::ProgramError, pubkey::Pubkey};

/// Repay depbt voucher from nft voucher program
#[derive(Clone, Debug, Default, PartialEq, BorshDeserialize)]
pub struct RepayDebtVoucher {
    /// Discriminator created nft voucher program
    pub discriminator: [u8; 8],
    /// Discount percentage
    pub discount_percentage: u8,
    /// Maximum amount to discount, in usd
    pub maximum_amount: u64,
    /// Start_time that voucher is applicable
    pub start_time: i64,
    /// End_time that voucher is applicable
    pub end_time: i64,
    /// NFT Mint address
    pub nft_mint: Pubkey,
    /// Authority of the nft
    pub authority: Pubkey,
    /// Market address
    pub market: Pubkey,
    /// Flag boolean indicate voucher is apply for all tokens
    pub is_universal_discount: bool,
    /// Reserve part 1
    pub _reserve_field: [u8; 15],
    /// Rereserve part 2
    pub _reserve: [u128; 3],
    /// List of eligible tokens
    pub eligible_tokens: Vec<Pubkey>,
}

impl RepayDebtVoucher {
    /// Discriminator got from nft voucher program
    pub fn discriminator() -> [u8; 8] {
        let concatenated_bytes = [b"account:".to_vec(), b"RepayDebtVoucher".to_vec()].concat();
        let hashed = solana_program::hash::hash(&concatenated_bytes);
        hashed.to_bytes()[0..8].try_into().unwrap()
    }

    /// Deserialize repay_debt_voucher
    pub fn try_from_account(repay_debt_voucher: &AccountInfo) -> Result<Self, ProgramError> {
        let repay_data = RepayDebtVoucher::try_from_slice(&repay_debt_voucher.data.borrow())
            .map_err(|error| {
                msg!("Repay debt voucher deserialized error: {:?}", error);
                error
            })?;
        return Ok(repay_data);
    }
}
