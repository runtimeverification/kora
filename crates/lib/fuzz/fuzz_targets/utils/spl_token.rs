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
pub enum FuzzTokenIx<'a> {
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

impl<'a> From<FuzzTokenIx<'a>> for TokenInstruction<'a> {
    fn from(value: FuzzTokenIx<'a>) -> Self {
        match value {
            FuzzTokenIx::InitializeMint { decimals, mint_authority, freeze_authority } => {
                Self::InitializeMint {
                    decimals: decimals,
                    mint_authority: mint_authority.into(),
                    freeze_authority: freeze_authority.map(|a| a.into()).into(),
                }
            }
            FuzzTokenIx::InitializeAccount => Self::InitializeAccount,
            FuzzTokenIx::InitializeMultisig { m } => Self::InitializeMultisig { m: m },
            FuzzTokenIx::Transfer { amount } => Self::Transfer { amount: amount },
            FuzzTokenIx::Approve { amount } => Self::Approve { amount: amount },
            FuzzTokenIx::Revoke => Self::Revoke,
            FuzzTokenIx::SetAuthority { authority_type, new_authority } => Self::SetAuthority {
                authority_type: authority_type.into(),
                new_authority: new_authority.map(|a| a.into()).into(),
            },
            FuzzTokenIx::MintTo { amount } => Self::MintTo { amount: amount },
            FuzzTokenIx::Burn { amount } => Self::Burn { amount: amount },
            FuzzTokenIx::CloseAccount => Self::CloseAccount,
            FuzzTokenIx::FreezeAccount => Self::FreezeAccount,
            FuzzTokenIx::ThawAccount => Self::ThawAccount,
            FuzzTokenIx::TransferChecked { amount, decimals } => {
                Self::TransferChecked { amount: amount, decimals: decimals }
            }

            FuzzTokenIx::ApproveChecked { amount, decimals } => {
                Self::ApproveChecked { amount: amount, decimals: decimals }
            }
            FuzzTokenIx::MintToChecked { amount, decimals } => {
                Self::MintToChecked { amount: amount, decimals: decimals }
            }
            FuzzTokenIx::BurnChecked { amount, decimals } => {
                Self::BurnChecked { amount: amount, decimals: decimals }
            }
            FuzzTokenIx::InitializeAccount2 { owner } => {
                Self::InitializeAccount2 { owner: owner.into() }
            }
            FuzzTokenIx::SyncNative => Self::SyncNative,
            FuzzTokenIx::InitializeAccount3 { owner } => {
                Self::InitializeAccount3 { owner: owner.into() }
            }
            FuzzTokenIx::InitializeMultisig2 { m } => Self::InitializeMultisig2 { m: m },
            FuzzTokenIx::InitializeMint2 { decimals, mint_authority, freeze_authority } => {
                Self::InitializeMint2 {
                    decimals: decimals,
                    mint_authority: mint_authority.into(),
                    freeze_authority: freeze_authority.map(|a| a.into()).into(),
                }
            }
            FuzzTokenIx::GetAccountDataSize => Self::GetAccountDataSize,
            FuzzTokenIx::InitializeImmutableOwner => Self::InitializeImmutableOwner,
            FuzzTokenIx::AmountToUiAmount { amount } => Self::AmountToUiAmount { amount: amount },
            FuzzTokenIx::UiAmountToAmount { ui_amount } => {
                Self::UiAmountToAmount { ui_amount: ui_amount }
            }
        }
    }
}
