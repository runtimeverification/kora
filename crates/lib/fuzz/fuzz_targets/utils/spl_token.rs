use arbitrary::Arbitrary;
use solana_sdk::pubkey::Pubkey;
use spl_token::instruction::{AuthorityType, TokenInstruction};

// TODO: The Arbitrary derive macro isn't good enough here for Pubkey. We need a manual impl that generates keys/selects them from a preset list.
#[derive(Arbitrary)]
pub struct FuzzPubKey([u8; 32]);

impl From<FuzzPubKey> for Pubkey {
    fn from(value: FuzzPubKey) -> Self {
        Self::new_from_array(value.0)
    }
}

#[derive(Arbitrary)]
pub enum FuzzAuthorityType {
    MintTokens,
    FreezeAccount,
    AccountOwner,
    CloseAccount,
}

impl From<FuzzAuthorityType> for AuthorityType {
    fn from(value: FuzzAuthorityType) -> Self {
        match value {
            FuzzAuthorityType::MintTokens => Self::MintTokens,
            FuzzAuthorityType::FreezeAccount => Self::FreezeAccount,
            FuzzAuthorityType::AccountOwner => Self::AccountOwner,
            FuzzAuthorityType::CloseAccount => Self::CloseAccount,
        }
    }
}

#[derive(Arbitrary)]
pub enum FuzzableSplIx<'a> {
    InitializeMint {
        decimals: u8,
        mint_authority: FuzzPubKey,
        freeze_authority: Option<FuzzPubKey>,
    },
    InitializeAccount,
    InitializeMultisig {
        m: u8,
    },
    Transfer {
        amount: u64,
    },
    Approve {
        amount: u64,
    },
    Revoke,
    SetAuthority {
        authority_type: FuzzAuthorityType,
        new_authority: Option<FuzzPubKey>,
    },
    MintTo {
        amount: u64,
    },
    Burn {
        amount: u64,
    },
    CloseAccount,
    FreezeAccount,
    ThawAccount,
    TransferChecked {
        amount: u64,
        decimals: u8,
    },
    ApproveChecked {
        amount: u64,
        decimals: u8,
    },
    MintToChecked {
        amount: u64,
        decimals: u8,
    },
    BurnChecked {
        amount: u64,
        decimals: u8,
    },
    InitializeAccount2 {
        owner: FuzzPubKey,
    },
    SyncNative,
    InitializeAccount3 {
        owner: FuzzPubKey,
    },
    InitializeMultisig2 {
        m: u8,
    },
    InitializeMint2 {
        decimals: u8,
        mint_authority: FuzzPubKey,
        freeze_authority: Option<FuzzPubKey>,
    },
    GetAccountDataSize,
    InitializeImmutableOwner,
    AmountToUiAmount {
        amount: u64,
    },
    UiAmountToAmount {
        ui_amount: &'a str,
    },
}

impl<'a> From<FuzzableSplIx<'a>> for TokenInstruction<'a> {
    fn from(value: FuzzableSplIx<'a>) -> Self {
        match value {
            FuzzableSplIx::InitializeMint { decimals, mint_authority, freeze_authority } => {
                Self::InitializeMint {
                    decimals: decimals,
                    mint_authority: mint_authority.into(),
                    freeze_authority: freeze_authority.map(|a| a.into()).into(),
                }
            }
            FuzzableSplIx::InitializeAccount => Self::InitializeAccount,
            FuzzableSplIx::InitializeMultisig { m } => Self::InitializeMultisig { m: m },
            FuzzableSplIx::Transfer { amount } => Self::Transfer { amount: amount },
            FuzzableSplIx::Approve { amount } => Self::Approve { amount: amount },
            FuzzableSplIx::Revoke => Self::Revoke,
            FuzzableSplIx::SetAuthority { authority_type, new_authority } => Self::SetAuthority {
                authority_type: authority_type.into(),
                new_authority: new_authority.map(|a| a.into()).into(),
            },
            FuzzableSplIx::MintTo { amount } => Self::MintTo { amount: amount },
            FuzzableSplIx::Burn { amount } => Self::Burn { amount: amount },
            FuzzableSplIx::CloseAccount => Self::CloseAccount,
            FuzzableSplIx::FreezeAccount => Self::FreezeAccount,
            FuzzableSplIx::ThawAccount => Self::ThawAccount,
            FuzzableSplIx::TransferChecked { amount, decimals } => {
                Self::TransferChecked { amount: amount, decimals: decimals }
            }

            FuzzableSplIx::ApproveChecked { amount, decimals } => {
                Self::ApproveChecked { amount: amount, decimals: decimals }
            }
            FuzzableSplIx::MintToChecked { amount, decimals } => {
                Self::MintToChecked { amount: amount, decimals: decimals }
            }
            FuzzableSplIx::BurnChecked { amount, decimals } => {
                Self::BurnChecked { amount: amount, decimals: decimals }
            }
            FuzzableSplIx::InitializeAccount2 { owner } => {
                Self::InitializeAccount2 { owner: owner.into() }
            }
            FuzzableSplIx::SyncNative => Self::SyncNative,
            FuzzableSplIx::InitializeAccount3 { owner } => {
                Self::InitializeAccount3 { owner: owner.into() }
            }
            FuzzableSplIx::InitializeMultisig2 { m } => Self::InitializeMultisig2 { m: m },
            FuzzableSplIx::InitializeMint2 { decimals, mint_authority, freeze_authority } => {
                Self::InitializeMint2 {
                    decimals: decimals,
                    mint_authority: mint_authority.into(),
                    freeze_authority: freeze_authority.map(|a| a.into()).into(),
                }
            }
            FuzzableSplIx::GetAccountDataSize => Self::GetAccountDataSize,
            FuzzableSplIx::InitializeImmutableOwner => Self::InitializeImmutableOwner,
            FuzzableSplIx::AmountToUiAmount { amount } => Self::AmountToUiAmount { amount: amount },
            FuzzableSplIx::UiAmountToAmount { ui_amount } => {
                Self::UiAmountToAmount { ui_amount: ui_amount }
            }
        }
    }
}
