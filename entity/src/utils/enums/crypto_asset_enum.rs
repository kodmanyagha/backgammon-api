use serde::{Deserialize, Serialize};

#[derive(
    strum_macros::Display,
    strum_macros::EnumString,
    strum_macros::EnumIter,
    Debug,
    Clone,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
)]
pub enum CryptoAssetEnum {
    #[strum(serialize = "try")]
    Try,

    #[strum(serialize = "usdt")]
    Usdt,

    #[strum(serialize = "btc")]
    Btc,

    #[strum(serialize = "eth")]
    Eth,

    #[strum(serialize = "xrp")]
    Xrp,

    #[strum(serialize = "bnb")]
    Bnb,

    #[strum(serialize = "sol")]
    Sol,

    #[strum(serialize = "trx")]
    Trx,

    #[strum(serialize = "doge")]
    Doge,

    #[strum(serialize = "hype")]
    Hype,

    #[strum(serialize = "leo")]
    Leo,

    #[strum(serialize = "ada")]
    Ada,

    #[strum(serialize = "aave")]
    Aave,

    #[strum(serialize = "apt")]
    Apt,

    #[strum(serialize = "near")]
    Near,

    #[strum(serialize = "1inch")]
    OneInch,

    #[strum(serialize = "bch")]
    Bch,

    #[strum(serialize = "xmr")]
    Xmr,

    #[strum(serialize = "link")]
    Link,

    #[strum(serialize = "zec")]
    Zec,

    #[strum(serialize = "xlm")]
    Xlm,

    #[strum(serialize = "dai")]
    Dai,

    #[strum(serialize = "ltc")]
    Ltc,

    #[strum(serialize = "avax")]
    Avax,

    #[strum(serialize = "hbar")]
    Hbar,

    #[strum(serialize = "sui")]
    Sui,

    #[strum(serialize = "shib")]
    Shib,

    #[strum(serialize = "ton")]
    Ton,

    #[strum(serialize = "xaut")]
    Xaut,

    #[strum(serialize = "wlfi")]
    Wlfi,

    #[strum(serialize = "tao")]
    Tao,

    #[strum(serialize = "mnt")]
    Mnt,

    #[strum(serialize = "dot")]
    Dot,
}
