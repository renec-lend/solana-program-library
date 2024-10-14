#![deny(missing_docs)]

//! A lending program for the Solana blockchain.

pub mod error;
pub mod instruction;
pub mod math;
pub mod state;

pub use state::*;

// Export current sdk types for downstream users building with a different sdk version
pub use solana_program;

/// mainnet program id
pub mod solend_mainnet {
    solana_program::declare_id!("9L193MV4yakKcgNT2tN4Kvf1ypn9c1sVMvsRn1Amw2Au");
}

/// devnet program id
pub mod solend_devnet {
    solana_program::declare_id!("9L193MV4yakKcgNT2tN4Kvf1ypn9c1sVMvsRn1Amw2Au");
}

/// Canonical null pubkey. Prints out as "nu11111111111111111111111111111111111111111"
pub const NULL_PUBKEY: solana_program::pubkey::Pubkey =
    solana_program::pubkey::Pubkey::new_from_array([
        11, 193, 238, 216, 208, 116, 241, 195, 55, 212, 76, 22, 75, 202, 40, 216, 76, 206, 27, 169,
        138, 64, 177, 28, 19, 90, 156, 0, 0, 0, 0, 0,
    ]);
    
