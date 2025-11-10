pub mod data;
pub mod error;
pub mod nonce;
pub mod ffi;

use std::str::FromStr;

use alloy::{
    primitives::eip191_hash_message, signers::local::PrivateKeySigner, signers::SignerSync,
};
pub use ffi::FFISigner;
use secrecy::ExposeSecret;
use serde_json::Value;

use crate::{
    signer::data::{
        ChangePubKeyData, CreateOrderData, SignBurnSharesData, SignCancelAllOrdersData,
        SignCancelOrderData, SignCreateGroupedOrdersData, SignCreatePublicPoolData,
        SignMintSharesData, SignModifyOrderData, SignTransferData, SignUpdateLeverageData,
        SignUpdateMarginData, SignUpdatePublicPoolData, SignWithdrawData, TxData, TxInfo,
        TxInfoData,
    },
};

