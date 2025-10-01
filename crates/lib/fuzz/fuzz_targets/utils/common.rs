use arbitrary::Arbitrary;
use solana_sdk::pubkey::Pubkey;

// TODO: The Arbitrary derive macro isn't good enough here for Pubkey. We need a manual impl that generates keys/selects them from a preset list.
#[derive(Arbitrary, Clone, Debug)]
pub struct FuzzPubkey([u8; 32]);

impl From<FuzzPubkey> for Pubkey {
    fn from(value: FuzzPubkey) -> Self {
        Self::new_from_array(value.0)
    }
}
