use solana_program::{
    account_info::AccountInfo, msg, program_error::ProgramError, sysvar::clock::Clock,
};
use solend_sdk::{
    error::LendingError,
    math::{Decimal, TryDiv},
};
use std::{convert::TryFrom, convert::TryInto, result::Result};

use borsh::BorshDeserialize;
use solana_program::pubkey::Pubkey;

use crate::{renec_oracle_mainnet, REUSD_REVND};

#[derive(BorshDeserialize, Debug, PartialEq)]
/// Asset Type
pub enum AssetType {
    /// Forex
    Forex,
    /// Crypto
    Crypto,
}

impl Default for AssetType {
    fn default() -> Self {
        AssetType::Crypto
    }
}

#[derive(BorshDeserialize, Debug, PartialEq)]
/// Product Status
pub enum ProductStatus {
    /// Unknown
    Unknown,
    /// Offline
    Offline,
    /// Online
    Online,
}

impl Default for ProductStatus {
    fn default() -> Self {
        ProductStatus::Unknown
    }
}

// #[account]
#[derive(BorshDeserialize, Debug, Default)]
/// Product
pub struct Product {
    /// Version
    pub version: u16,
    /// Product Status, size = 1
    pub status: ProductStatus,
    /// Asset Type, size = 1
    pub asset_type: AssetType,

    /// Quote currence, e.g: USD; STR_MAX_LEN
    pub quote_currency: String,
    /// Quote mint
    pub quote_mint: Pubkey,
    /// Base currency, e.g: VND; STR_MAX_LEN
    pub base_currency: String,
    /// Base mint
    pub base_mint: Pubkey,

    /// Price account
    pub price_account: Pubkey,
    /// Expo
    pub expo: i32,
    /// Max price, e.g: price: 2546734 and expo: -2 => represents: 25467,34 (price * 10^(expo))
    pub max_price: u64,
    /// Min price
    pub min_price: u64,
    /// Window size
    pub window_size: u64,
    /// Controller
    pub controller: Pubkey,
    /// Bump
    pub bump: [u8; 1],
}

#[derive(BorshDeserialize, Debug, PartialEq)]
/// Price Status
pub enum PriceStatus {
    /// Unknow
    Unknown,
    /// Offline
    Offline,
    /// Online
    Online,
}

impl Default for PriceStatus {
    fn default() -> Self {
        PriceStatus::Unknown
    }
}

#[derive(BorshDeserialize, Debug, Default)]
/// Price
pub struct RenecPrice {
    /// Version
    pub version: u16,
    /// Price status
    pub status: PriceStatus,
    /// Product Account
    pub product_account: Pubkey,

    /// Price
    pub price: u64,
    /// Num Publishers
    pub num_publishers: u16,
    /// Timestamp
    pub timestamp: u64,

    /// Prev price
    pub prev_price: u64,
    /// Prev timestamp
    pub prev_timestamp: u64,
    /// Bump
    pub bump: [u8; 1],
}

pub struct PriceCalculator {
    pub price: u64,
    pub expo: i32,
}

impl PriceCalculator {
    pub fn new(price: u64, expo: i32) -> Result<Self, ProgramError> {
        if expo > 0 {
            return Err(LendingError::ExpoPositiveNonSupport.into());
        }

        Ok(Self { price, expo })
    }
}

pub fn get_renec_oracle_price(
    price_info: &AccountInfo,
    price_product: &AccountInfo,
    clock: &Clock,
) -> Result<(Decimal, Decimal), ProgramError> {
    if !is_renec_oracle(price_info) {
        return Err(LendingError::InvalidRenecOracleAccount.into());
    }
    if !is_renec_oracle(price_product) {
        return Err(LendingError::InvalidRenecOracleAccount.into());
    }
    const STALE_AFTER_SLOTS_ELAPSED: u64 = 60;

    let product_data: &[u8] = &price_product.try_borrow_data()?;
    let mut oracle_product_data: &[u8] = &product_data[8..];
    let oracle_product_info: Product = BorshDeserialize::deserialize(&mut oracle_product_data)
        .unwrap_or_else(|error| {
            msg!("Product deserialize error: {:?}", error);
            Product::default()
        });

    if !oracle_product_info.price_account.eq(&price_info.key) {
        return Err(LendingError::InvalidPriceOfProductOracle.into());
    }

    if *price_info.key == solend_sdk::NULL_PUBKEY {
        return Err(LendingError::NullOracleConfig.into());
    }

    if oracle_product_info.status != ProductStatus::Online {
        return Err(LendingError::UnavailableProduct.into());
    }

    let price_data: &[u8] = &price_info.try_borrow_data()?;
    let mut oracle_price_data: &[u8] = &price_data[8..];
    let oracle_price_info: RenecPrice = BorshDeserialize::deserialize(&mut oracle_price_data)
        .unwrap_or_else(|error| {
            msg!("Price deserialize error: {:?}", error);
            RenecPrice::default()
        });

    if oracle_price_info.status != PriceStatus::Online {
        return Err(LendingError::UnavailablePriceInfo.into());
    }
    let now = to_timestamp_u64(clock.unix_timestamp)?;
    // price must be not older than over 60s
    if now - STALE_AFTER_SLOTS_ELAPSED > oracle_price_info.timestamp {
        return Err(LendingError::PriceTooOld.into());
    }

    // 24_500_000_000, expo = -6
    let price_calculator = PriceCalculator::new(oracle_price_info.price, oracle_product_info.expo)?;

    let price_key = price_info.key.to_string();
    let is_reverse_pair = price_key == REUSD_REVND;
    let market_price = price_calculator_to_decimal(&price_calculator, is_reverse_pair)?;
    let ema_price = market_price.clone();
    Ok((market_price, ema_price))
}

// Get price from renec oracle with default expo = -2 without checking staleness or variance. only used
pub fn get_renec_oracle_price_unchecked_with_default_expo(price_info: &AccountInfo) -> Result<Decimal, ProgramError> {
    let default_expo = -2;

    if !is_renec_oracle(price_info) {
        return Err(LendingError::InvalidRenecOracleAccount.into());
    }

    if *price_info.key == solend_sdk::NULL_PUBKEY {
        return Err(LendingError::NullOracleConfig.into());
    }

    let price_data: &[u8] = &price_info.try_borrow_data()?;
    let mut oracle_price_data: &[u8] = &price_data[8..];
    let oracle_price_info: RenecPrice =
        BorshDeserialize::deserialize(&mut oracle_price_data).map_err(|error| {
            msg!("Price deserialize error: {:?}", error);
            error
        })?;

    if oracle_price_info.status != PriceStatus::Online {
        return Err(LendingError::UnavailablePriceInfo.into());
    }

    let price_calculator = PriceCalculator::new(oracle_price_info.price, default_expo)?;

    let price_key = price_info.key.to_string();
    let is_reverse_pair = price_key == REUSD_REVND;
    let market_price = price_calculator_to_decimal(&price_calculator, is_reverse_pair)?;
    Ok(market_price)
}

pub fn validate_renec_keys(price_info: &AccountInfo, product_info: &AccountInfo) -> Result<(), ProgramError> {
    if *price_info.key == solend_sdk::NULL_PUBKEY {
        return Ok(());
    }

    if !is_renec_oracle(price_info) {
        msg!("Price info is not a renec oracle");
        return Err(LendingError::InvalidRenecOracleAccount.into());
    }
    if !is_renec_oracle(product_info) {
        msg!("Product info is not a renec oracle");
        return Err(LendingError::InvalidRenecOracleAccount.into());
    }

    let product_data: &[u8] = &product_info.try_borrow_data()?;
    let mut oracle_product_data: &[u8] = &product_data[8..];
    let oracle_product_info: Product = BorshDeserialize::deserialize(&mut oracle_product_data)
        .unwrap_or_else(|error| {
            msg!("Product deserialize error: {:?}", error);
            Product::default()
        });

    if !oracle_product_info.price_account.eq(&price_info.key) {
        return Err(LendingError::InvalidPriceOfProductOracle.into());
    }

    Ok(())
}

pub fn is_renec_oracle(account: &AccountInfo) -> bool {
    account.owner == &renec_oracle_mainnet::id()
}

fn to_timestamp_u64(t: i64) -> Result<u64, LendingError> {
    u64::try_from(t).or(Err(LendingError::InvalidTimestampConversion))
}

fn price_calculator_to_decimal(
    price_calculator: &PriceCalculator,
    is_reverse: bool,
) -> Result<Decimal, ProgramError> {
    let price: u64 = price_calculator.price.try_into().map_err(|_| {
        msg!("Oracle price cannot be negative");
        LendingError::InvalidOracleConfig
    })?;

    let exponent = price_calculator
        .expo
        .checked_abs()
        .ok_or(LendingError::MathOverflow)?
        .try_into()
        .map_err(|_| LendingError::MathOverflow)?;
    let decimals = 10u64
        .checked_pow(exponent)
        .ok_or(LendingError::MathOverflow)?;
    if is_reverse {
        msg!("Get reverse price");
        Decimal::from(decimals).try_div(price)
    } else {
        Decimal::from(price).try_div(decimals)
    }
}
