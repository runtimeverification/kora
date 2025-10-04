use arbitrary::{Arbitrary, Result, Unstructured};
use kora_lib::state::get_config;
use solana_sdk::pubkey::Pubkey;
use std::{str::FromStr, sync::LazyLock};

static FUZZ_KEYS: LazyLock<Vec<Pubkey>> = LazyLock::new(|| {
    let count = 5;
    let mut res = Vec::with_capacity(count);
    for _ in 1..=count {
        res.push(Pubkey::new_unique());
    }

    let config = get_config().expect("Could not retrieve global config");
    if let Some(payment_address) = &config.kora.payment_address {
        let signer_key =
            Pubkey::from_str(payment_address.as_str()).expect("Could not parse payment address");
        res.push(signer_key);
    }

    res
});

#[derive(Clone, Debug)]
pub struct FuzzPubkey(Pubkey);

impl<'a> Arbitrary<'a> for FuzzPubkey {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        let keys = &FUZZ_KEYS;
        let index = u.int_in_range(1..=keys.len())? - 1;
        Ok(Self(keys[index]))
    }
}

impl From<FuzzPubkey> for Pubkey {
    fn from(value: FuzzPubkey) -> Self {
        value.0
    }
}
