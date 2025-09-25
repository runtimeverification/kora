use arbitrary::{Arbitrary, Unstructured};
use kora_lib::{
    config::{self, EnabledMethods},
    fee::price::{PriceConfig, PriceModel},
    tests::config_mock::ConfigMockBuilder,
    Config,
};

pub struct FuzzConfig {
    pub kora_config: Config,
}

impl<'a> Arbitrary<'a> for FuzzConfig {
    fn arbitrary(u: &mut Unstructured<'a>) -> libfuzzer_sys::arbitrary::Result<Self> {
        let mut config = ConfigMockBuilder::new().with_cache_enabled(false).build();

        // Validation config
        let validation = &mut config.validation;
        validation.max_allowed_lamports = u64::arbitrary(u)?;
        validation.max_signatures = u64::arbitrary(u)?;

        validation.fee_payer_policy = config::FeePayerPolicy::arbitrary(u)?;

        let n: u64 = u.arbitrary()?;
        let margin = (n as f64) / (u64::MAX as f64);
        validation.price = PriceConfig { model: PriceModel::Margin { margin: margin } };

        validation.token_2022 = config::Token2022Config::arbitrary(u)?;

        // Kora config
        let kora = &mut config.kora;
        kora.enabled_methods = EnabledMethods::arbitrary(u)?;

        Ok(Self { kora_config: config })
    }
}
