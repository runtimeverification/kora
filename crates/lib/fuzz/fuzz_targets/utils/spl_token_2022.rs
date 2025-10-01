use super::common::FuzzPubkey;
use arbitrary::Arbitrary;
use spl_token_2022::{
    extension::ExtensionType,
    instruction::{AuthorityType, TokenInstruction},
};

#[derive(Arbitrary, Clone, Debug)]
pub enum FuzzAuthorityType {
    MintTokens,
    FreezeAccount,
    AccountOwner,
    CloseAccount,
    TransferFeeConfig,
    WithheldWithdraw,
    CloseMint,
    InterestRate,
    PermanentDelegate,
    ConfidentialTransferMint,
    TransferHookProgramId,
    ConfidentialTransferFeeConfig,
    MetadataPointer,
    GroupPointer,
    GroupMemberPointer,
    ScaledUiAmount,
    Pause,
}

impl From<FuzzAuthorityType> for AuthorityType {
    fn from(value: FuzzAuthorityType) -> Self {
        match value {
            FuzzAuthorityType::MintTokens => Self::MintTokens,
            FuzzAuthorityType::FreezeAccount => Self::FreezeAccount,
            FuzzAuthorityType::AccountOwner => Self::AccountOwner,
            FuzzAuthorityType::CloseAccount => Self::CloseAccount,
            FuzzAuthorityType::TransferFeeConfig => Self::TransferFeeConfig,
            FuzzAuthorityType::WithheldWithdraw => Self::WithheldWithdraw,
            FuzzAuthorityType::CloseMint => Self::CloseMint,
            FuzzAuthorityType::InterestRate => Self::InterestRate,
            FuzzAuthorityType::PermanentDelegate => Self::PermanentDelegate,
            FuzzAuthorityType::ConfidentialTransferMint => Self::ConfidentialTransferMint,
            FuzzAuthorityType::TransferHookProgramId => Self::TransferHookProgramId,
            FuzzAuthorityType::ConfidentialTransferFeeConfig => Self::ConfidentialTransferFeeConfig,
            FuzzAuthorityType::MetadataPointer => Self::MetadataPointer,
            FuzzAuthorityType::GroupPointer => Self::GroupPointer,
            FuzzAuthorityType::GroupMemberPointer => Self::GroupMemberPointer,
            FuzzAuthorityType::ScaledUiAmount => Self::ScaledUiAmount,
            FuzzAuthorityType::Pause => Self::Pause,
        }
    }
}

#[derive(Arbitrary, Clone, Debug)]
pub enum FuzzExtensionType {
    Uninitialized,
    TransferFeeConfig,
    TransferFeeAmount,
    MintCloseAuthority,
    ConfidentialTransferMint,
    ConfidentialTransferAccount,
    DefaultAccountState,
    ImmutableOwner,
    MemoTransfer,
    NonTransferable,
    InterestBearingConfig,
    CpiGuard,
    PermanentDelegate,
    NonTransferableAccount,
    TransferHook,
    TransferHookAccount,
    ConfidentialTransferFeeConfig,
    ConfidentialTransferFeeAmount,
    MetadataPointer,
    TokenMetadata,
    GroupPointer,
    TokenGroup,
    GroupMemberPointer,
    TokenGroupMember,
    ConfidentialMintBurn,
    ScaledUiAmount,
    Pausable,
    PausableAccount,
}

impl From<FuzzExtensionType> for ExtensionType {
    fn from(value: FuzzExtensionType) -> Self {
        match value {
            FuzzExtensionType::Uninitialized => Self::Uninitialized,
            FuzzExtensionType::TransferFeeConfig => Self::TransferFeeConfig,
            FuzzExtensionType::TransferFeeAmount => Self::TransferFeeAmount,
            FuzzExtensionType::MintCloseAuthority => Self::MintCloseAuthority,
            FuzzExtensionType::ConfidentialTransferMint => Self::ConfidentialTransferMint,
            FuzzExtensionType::ConfidentialTransferAccount => Self::ConfidentialTransferAccount,
            FuzzExtensionType::DefaultAccountState => Self::DefaultAccountState,
            FuzzExtensionType::ImmutableOwner => Self::ImmutableOwner,
            FuzzExtensionType::MemoTransfer => Self::MemoTransfer,
            FuzzExtensionType::NonTransferable => Self::NonTransferable,
            FuzzExtensionType::InterestBearingConfig => Self::InterestBearingConfig,
            FuzzExtensionType::CpiGuard => Self::CpiGuard,
            FuzzExtensionType::PermanentDelegate => Self::PermanentDelegate,
            FuzzExtensionType::NonTransferableAccount => Self::NonTransferableAccount,
            FuzzExtensionType::TransferHook => Self::TransferHook,
            FuzzExtensionType::TransferHookAccount => Self::TransferHookAccount,
            FuzzExtensionType::ConfidentialTransferFeeConfig => Self::ConfidentialTransferFeeConfig,
            FuzzExtensionType::ConfidentialTransferFeeAmount => Self::ConfidentialTransferFeeAmount,
            FuzzExtensionType::MetadataPointer => Self::MetadataPointer,
            FuzzExtensionType::TokenMetadata => Self::TokenMetadata,
            FuzzExtensionType::GroupPointer => Self::GroupPointer,
            FuzzExtensionType::TokenGroup => Self::TokenGroup,
            FuzzExtensionType::GroupMemberPointer => Self::GroupMemberPointer,
            FuzzExtensionType::TokenGroupMember => Self::TokenGroupMember,
            FuzzExtensionType::ConfidentialMintBurn => Self::ConfidentialMintBurn,
            FuzzExtensionType::ScaledUiAmount => Self::ScaledUiAmount,
            FuzzExtensionType::Pausable => Self::Pausable,
            FuzzExtensionType::PausableAccount => Self::PausableAccount,
        }
    }
}

impl From<&FuzzExtensionType> for ExtensionType {
    fn from(value: &FuzzExtensionType) -> Self {
        value.clone().into()
    }
}

#[derive(Arbitrary, Clone, Debug)]
pub enum FuzzToken2022Ix<'a> {
    InitializeMint {
        decimals: u8,
        mint_authority: FuzzPubkey,
        freeze_authority: Option<FuzzPubkey>,
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
        new_authority: Option<FuzzPubkey>,
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
        owner: FuzzPubkey,
    },
    SyncNative,
    InitializeAccount3 {
        owner: FuzzPubkey,
    },
    InitializeMultisig2 {
        m: u8,
    },
    InitializeMint2 {
        decimals: u8,
        mint_authority: FuzzPubkey,
        freeze_authority: Option<FuzzPubkey>,
    },
    GetAccountDataSize {
        extension_types: Vec<FuzzExtensionType>,
    },
    InitializeImmutableOwner,
    AmountToUiAmount {
        amount: u64,
    },
    UiAmountToAmount {
        ui_amount: &'a str,
    },
    InitializeMintCloseAuthority {
        close_authority: Option<FuzzPubkey>,
    },
    TransferFeeExtension,
    ConfidentialTransferExtension,
    DefaultAccountStateExtension,
    Reallocate {
        extension_types: Vec<FuzzExtensionType>,
    },
    MemoTransferExtension,
    CreateNativeMint,
    InitializeNonTransferableMint,
    InterestBearingMintExtension,
    CpiGuardExtension,
    InitializePermanentDelegate {
        delegate: FuzzPubkey,
    },
    TransferHookExtension,
    ConfidentialTransferFeeExtension,
    WithdrawExcessLamports,
    MetadataPointerExtension,
    GroupPointerExtension,
    GroupMemberPointerExtension,
    ConfidentialMintBurnExtension,
    ScaledUiAmountExtension,
    PausableExtension,
}

impl<'a> From<FuzzToken2022Ix<'a>> for TokenInstruction<'a> {
    fn from(value: FuzzToken2022Ix<'a>) -> Self {
        match value {
            FuzzToken2022Ix::InitializeMint { decimals, mint_authority, freeze_authority } => {
                Self::InitializeMint {
                    decimals: decimals,
                    mint_authority: mint_authority.into(),
                    freeze_authority: freeze_authority.map(|a| a.into()).into(),
                }
            }
            FuzzToken2022Ix::InitializeAccount => Self::InitializeAccount,
            FuzzToken2022Ix::InitializeMultisig { m } => Self::InitializeMultisig { m: m },
            FuzzToken2022Ix::Transfer { amount } => Self::Transfer { amount: amount },
            FuzzToken2022Ix::Approve { amount } => Self::Approve { amount: amount },
            FuzzToken2022Ix::Revoke => Self::Revoke,
            FuzzToken2022Ix::SetAuthority { authority_type, new_authority } => Self::SetAuthority {
                authority_type: authority_type.into(),
                new_authority: new_authority.map(|a| a.into()).into(),
            },
            FuzzToken2022Ix::MintTo { amount } => Self::MintTo { amount: amount },
            FuzzToken2022Ix::Burn { amount } => Self::Burn { amount: amount },
            FuzzToken2022Ix::CloseAccount => Self::CloseAccount,
            FuzzToken2022Ix::FreezeAccount => Self::FreezeAccount,
            FuzzToken2022Ix::ThawAccount => Self::ThawAccount,
            FuzzToken2022Ix::TransferChecked { amount, decimals } => {
                Self::TransferChecked { amount: amount, decimals: decimals }
            }
            FuzzToken2022Ix::ApproveChecked { amount, decimals } => {
                Self::ApproveChecked { amount: amount, decimals: decimals }
            }
            FuzzToken2022Ix::MintToChecked { amount, decimals } => {
                Self::MintToChecked { amount: amount, decimals: decimals }
            }
            FuzzToken2022Ix::BurnChecked { amount, decimals } => {
                Self::BurnChecked { amount: amount, decimals: decimals }
            }
            FuzzToken2022Ix::InitializeAccount2 { owner } => {
                Self::InitializeAccount2 { owner: owner.into() }
            }
            FuzzToken2022Ix::SyncNative => Self::SyncNative,
            FuzzToken2022Ix::InitializeAccount3 { owner } => {
                Self::InitializeAccount3 { owner: owner.into() }
            }
            FuzzToken2022Ix::InitializeMultisig2 { m } => Self::InitializeMultisig2 { m: m },
            FuzzToken2022Ix::InitializeMint2 { decimals, mint_authority, freeze_authority } => {
                Self::InitializeMint2 {
                    decimals: decimals,
                    mint_authority: mint_authority.into(),
                    freeze_authority: freeze_authority.map(|a| a.into()).into(),
                }
            }
            FuzzToken2022Ix::GetAccountDataSize { extension_types } => Self::GetAccountDataSize {
                extension_types: extension_types.iter().map(|a| a.into()).collect(),
            },
            FuzzToken2022Ix::InitializeImmutableOwner => Self::InitializeImmutableOwner,
            FuzzToken2022Ix::AmountToUiAmount { amount } => {
                Self::AmountToUiAmount { amount: amount }
            }
            FuzzToken2022Ix::UiAmountToAmount { ui_amount } => {
                Self::UiAmountToAmount { ui_amount: ui_amount }
            }
            FuzzToken2022Ix::InitializeMintCloseAuthority { close_authority } => {
                Self::InitializeMintCloseAuthority {
                    close_authority: close_authority.map(|a| a.into()).into(),
                }
            }
            FuzzToken2022Ix::TransferFeeExtension => Self::TransferFeeExtension,
            FuzzToken2022Ix::ConfidentialTransferExtension => Self::ConfidentialTransferExtension,
            FuzzToken2022Ix::DefaultAccountStateExtension => Self::DefaultAccountStateExtension,
            FuzzToken2022Ix::Reallocate { extension_types } => Self::Reallocate {
                extension_types: extension_types.iter().map(|a| a.into()).collect(),
            },
            FuzzToken2022Ix::MemoTransferExtension => Self::MemoTransferExtension,
            FuzzToken2022Ix::CreateNativeMint => Self::CreateNativeMint,
            FuzzToken2022Ix::InitializeNonTransferableMint => Self::InitializeNonTransferableMint,
            FuzzToken2022Ix::InterestBearingMintExtension => Self::InterestBearingMintExtension,
            FuzzToken2022Ix::CpiGuardExtension => Self::CpiGuardExtension,
            FuzzToken2022Ix::InitializePermanentDelegate { delegate } => {
                Self::InitializePermanentDelegate { delegate: delegate.into() }
            }
            FuzzToken2022Ix::TransferHookExtension => Self::TransferHookExtension,
            FuzzToken2022Ix::ConfidentialTransferFeeExtension => {
                Self::ConfidentialTransferFeeExtension
            }
            FuzzToken2022Ix::WithdrawExcessLamports => Self::WithdrawExcessLamports,
            FuzzToken2022Ix::MetadataPointerExtension => Self::MetadataPointerExtension,
            FuzzToken2022Ix::GroupPointerExtension => Self::GroupPointerExtension,
            FuzzToken2022Ix::GroupMemberPointerExtension => Self::GroupMemberPointerExtension,
            FuzzToken2022Ix::ConfidentialMintBurnExtension => Self::ConfidentialMintBurnExtension,
            FuzzToken2022Ix::ScaledUiAmountExtension => Self::ScaledUiAmountExtension,
            FuzzToken2022Ix::PausableExtension => Self::PausableExtension,
        }
    }
}
