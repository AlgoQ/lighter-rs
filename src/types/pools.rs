//! Pool-related transaction types

use super::{OrderInfo, TxInfo};
use crate::constants::*;
use crate::errors::{FFIError, LighterError, Result};
use crate::types::common::ffisigner;
use crate::types::common::{self, parse_result};
use serde::{Deserialize, Serialize};
use std::ffi::{c_int, c_longlong, CStr, CString};
/// Create Public Pool Transaction Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePublicPoolTxReq {
    pub operator_fee: i64,
    pub initial_total_shares: i64,
    pub min_operator_share_rate: i64,
}

/// Update Public Pool Transaction Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePublicPoolTxReq {
    pub public_pool_index: i64,
    pub status: u8,
    pub operator_fee: i64,
    pub min_operator_share_rate: i64,
}

/// Mint Shares Transaction Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MintSharesTxReq {
    pub public_pool_index: i64,
    pub share_amount: i64,
}

/// Burn Shares Transaction Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurnSharesTxReq {
    pub public_pool_index: i64,
    pub share_amount: i64,
}

/// L2 Create Public Pool Transaction Info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L2CreatePublicPoolTxInfo {
    pub account_index: i64,
    pub api_key_index: u8,
    pub operator_fee: i64,
    pub initial_total_shares: i64,
    pub min_operator_share_rate: i64,
    pub expired_at: i64,
    pub nonce: i64,
    #[serde(skip)]
    pub signed_hash: Option<String>,
}

impl TxInfo for L2CreatePublicPoolTxInfo {
    fn get_tx_type(&self) -> u8 {
        TX_TYPE_L2_CREATE_PUBLIC_POOL
    }

    fn get_tx_info(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }

    fn get_tx_hash(&self) -> Option<String> {
        self.signed_hash.clone()
    }

    fn validate(&self) -> Result<()> {
        if self.account_index < MIN_ACCOUNT_INDEX || self.account_index > MAX_ACCOUNT_INDEX {
            return Err(LighterError::AccountIndexTooLow(self.account_index));
        }
        if self.operator_fee <= 0 || self.operator_fee > FEE_TICK {
            return Err(LighterError::InvalidPoolOperatorFee);
        }
        if self.initial_total_shares < MIN_INITIAL_TOTAL_SHARES {
            return Err(LighterError::PoolInitialTotalSharesTooLow(
                self.initial_total_shares,
            ));
        }
        if self.initial_total_shares > MAX_INITIAL_TOTAL_SHARES {
            return Err(LighterError::PoolInitialTotalSharesTooHigh(
                self.initial_total_shares,
            ));
        }
        if self.min_operator_share_rate <= 0 || self.min_operator_share_rate > SHARE_TICK {
            return Err(LighterError::PoolMinOperatorShareRateTooLow);
        }
        if self.nonce < MIN_NONCE {
            return Err(LighterError::NonceTooLow(self.nonce));
        }
        Ok(())
    }

    fn hash(&self) -> Result<String> {
        // DONE: Implement Poseidon2 hashing
        let hash_or_err = unsafe {
            ffisigner::SignCreatePublicPool(
                self.operator_fee,
                self.initial_total_shares,
                self.min_operator_share_rate,
                self.nonce,
            )
        };

        parse_result(hash_or_err)
    }
}

/// L2 Update Public Pool Transaction Info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L2UpdatePublicPoolTxInfo {
    pub account_index: i64,
    pub api_key_index: u8,
    pub public_pool_index: i64,
    pub status: u8,
    pub operator_fee: i64,
    pub min_operator_share_rate: i64,
    pub expired_at: i64,
    pub nonce: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signed_hash: Option<String>,
}

impl TxInfo for L2UpdatePublicPoolTxInfo {
    fn get_tx_type(&self) -> u8 {
        TX_TYPE_L2_UPDATE_PUBLIC_POOL
    }

    fn get_tx_info(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }

    fn get_tx_hash(&self) -> Option<String> {
        self.signed_hash.clone()
    }

    fn validate(&self) -> Result<()> {
        if self.account_index < MIN_ACCOUNT_INDEX || self.account_index > MAX_ACCOUNT_INDEX {
            return Err(LighterError::AccountIndexTooLow(self.account_index));
        }
        if self.public_pool_index < MIN_ACCOUNT_INDEX || self.public_pool_index > MAX_ACCOUNT_INDEX
        {
            return Err(LighterError::PublicPoolIndexTooLow(self.public_pool_index));
        }
        if self.status != 0 && self.status != 1 {
            return Err(LighterError::InvalidPoolStatus);
        }
        if self.nonce < MIN_NONCE {
            return Err(LighterError::NonceTooLow(self.nonce));
        }
        Ok(())
    }

    fn hash(&self) -> Result<String> {
        // DONE: Implement Poseidon2 hashing
        let hash_or_err = unsafe {
            ffisigner::SignUpdatePublicPool(
                self.public_pool_index,
                self.status as i32,
                self.operator_fee,
                self.min_operator_share_rate,
                self.nonce,
            )
        };
        parse_result(hash_or_err)
    }
}

/// L2 Mint Shares Transaction Info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L2MintSharesTxInfo {
    pub account_index: i64,
    pub api_key_index: u8,
    pub public_pool_index: i64,
    pub share_amount: i64,
    pub expired_at: i64,
    pub nonce: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signed_hash: Option<String>,
}

impl TxInfo for L2MintSharesTxInfo {
    fn get_tx_type(&self) -> u8 {
        TX_TYPE_L2_MINT_SHARES
    }

    fn get_tx_info(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }

    fn get_tx_hash(&self) -> Option<String> {
        self.signed_hash.clone()
    }

    fn validate(&self) -> Result<()> {
        if self.account_index < MIN_ACCOUNT_INDEX || self.account_index > MAX_ACCOUNT_INDEX {
            return Err(LighterError::AccountIndexTooLow(self.account_index));
        }
        if self.share_amount < MIN_POOL_SHARES_TO_MINT_OR_BURN {
            return Err(LighterError::PoolMintShareAmountTooLow(self.share_amount));
        }
        if self.share_amount > MAX_POOL_SHARES_TO_MINT_OR_BURN {
            return Err(LighterError::PoolMintShareAmountTooHigh(self.share_amount));
        }
        if self.nonce < MIN_NONCE {
            return Err(LighterError::NonceTooLow(self.nonce));
        }
        Ok(())
    }

    fn hash(&self) -> Result<String> {
        // DONE: Implement Poseidon2 hashing
        let hash_or_err = unsafe {
            ffisigner::SignMintShares(self.public_pool_index, self.share_amount, self.nonce)
        };
        parse_result(hash_or_err)
    }
}

/// L2 Burn Shares Transaction Info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L2BurnSharesTxInfo {
    pub account_index: i64,
    pub api_key_index: u8,
    pub public_pool_index: i64,
    pub share_amount: i64,
    pub expired_at: i64,
    pub nonce: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signed_hash: Option<String>,
}

impl TxInfo for L2BurnSharesTxInfo {
    fn get_tx_type(&self) -> u8 {
        TX_TYPE_L2_BURN_SHARES
    }

    fn get_tx_info(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }

    fn get_tx_hash(&self) -> Option<String> {
        self.signed_hash.clone()
    }

    fn validate(&self) -> Result<()> {
        if self.account_index < MIN_ACCOUNT_INDEX || self.account_index > MAX_ACCOUNT_INDEX {
            return Err(LighterError::AccountIndexTooLow(self.account_index));
        }
        if self.share_amount < MIN_POOL_SHARES_TO_MINT_OR_BURN {
            return Err(LighterError::PoolBurnShareAmountTooLow(self.share_amount));
        }
        if self.share_amount > MAX_POOL_SHARES_TO_MINT_OR_BURN {
            return Err(LighterError::PoolBurnShareAmountTooHigh(self.share_amount));
        }
        if self.nonce < MIN_NONCE {
            return Err(LighterError::NonceTooLow(self.nonce));
        }
        Ok(())
    }

    fn hash(&self) -> Result<String> {
        // DONE: Implement Poseidon2 hashing
        let hash_or_err = unsafe {
            ffisigner::SignBurnShares(self.public_pool_index, self.share_amount, self.nonce)
        };
        parse_result(hash_or_err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_public_pool_validation_success() {
        let tx_info = L2CreatePublicPoolTxInfo {
            account_index: 12345,
            api_key_index: 0,
            operator_fee: 10000,
            initial_total_shares: 1000000000,
            min_operator_share_rate: 5000,
            expired_at: 1000000,
            nonce: 1,
            signed_hash: None,
        };

        assert!(tx_info.validate().is_ok());
        assert_eq!(tx_info.get_tx_type(), TX_TYPE_L2_CREATE_PUBLIC_POOL);
    }

    #[test]
    fn test_create_public_pool_invalid_operator_fee() {
        let tx_info = L2CreatePublicPoolTxInfo {
            account_index: 12345,
            api_key_index: 0,
            operator_fee: -1,
            initial_total_shares: 1000000000,
            min_operator_share_rate: 5000,
            expired_at: 1000000,
            nonce: 1,
            signed_hash: None,
        };

        let result = tx_info.validate();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LighterError::InvalidPoolOperatorFee
        ));
    }

    #[test]
    fn test_create_public_pool_shares_too_low() {
        let tx_info = L2CreatePublicPoolTxInfo {
            account_index: 12345,
            api_key_index: 0,
            operator_fee: 10000,
            initial_total_shares: 100,
            min_operator_share_rate: 5000,
            expired_at: 1000000,
            nonce: 1,
            signed_hash: None,
        };

        let result = tx_info.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_update_public_pool_validation_success() {
        let tx_info = L2UpdatePublicPoolTxInfo {
            account_index: 12345,
            api_key_index: 0,
            public_pool_index: 100,
            status: 1,
            operator_fee: 10000,
            min_operator_share_rate: 5000,
            expired_at: 1000000,
            nonce: 1,
            signed_hash: None,
        };

        assert!(tx_info.validate().is_ok());
        assert_eq!(tx_info.get_tx_type(), TX_TYPE_L2_UPDATE_PUBLIC_POOL);
    }

    #[test]
    fn test_update_public_pool_invalid_status() {
        let tx_info = L2UpdatePublicPoolTxInfo {
            account_index: 12345,
            api_key_index: 0,
            public_pool_index: 100,
            status: 2,
            operator_fee: 10000,
            min_operator_share_rate: 5000,
            expired_at: 1000000,
            nonce: 1,
            signed_hash: None,
        };

        let result = tx_info.validate();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LighterError::InvalidPoolStatus
        ));
    }

    #[test]
    fn test_mint_shares_validation_success() {
        let tx_info = L2MintSharesTxInfo {
            account_index: 12345,
            api_key_index: 0,
            public_pool_index: 100,
            share_amount: 100000,
            expired_at: 1000000,
            nonce: 1,
            signed_hash: None,
        };

        assert!(tx_info.validate().is_ok());
        assert_eq!(tx_info.get_tx_type(), TX_TYPE_L2_MINT_SHARES);
    }

    #[test]
    fn test_mint_shares_amount_too_low() {
        let tx_info = L2MintSharesTxInfo {
            account_index: 12345,
            api_key_index: 0,
            public_pool_index: 100,
            share_amount: 0,
            expired_at: 1000000,
            nonce: 1,
            signed_hash: None,
        };

        let result = tx_info.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_burn_shares_validation_success() {
        let tx_info = L2BurnSharesTxInfo {
            account_index: 12345,
            api_key_index: 0,
            public_pool_index: 100,
            share_amount: 100000,
            expired_at: 1000000,
            nonce: 1,
            signed_hash: None,
        };

        assert!(tx_info.validate().is_ok());
        assert_eq!(tx_info.get_tx_type(), TX_TYPE_L2_BURN_SHARES);
    }

    #[test]
    fn test_burn_shares_amount_too_low() {
        let tx_info = L2BurnSharesTxInfo {
            account_index: 12345,
            api_key_index: 0,
            public_pool_index: 100,
            share_amount: 0,
            expired_at: 1000000,
            nonce: 1,
            signed_hash: None,
        };

        let result = tx_info.validate();
        assert!(result.is_err());
    }
}
