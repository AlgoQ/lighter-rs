//! Order-related transaction types

use super::{OrderInfo, TxInfo};
use crate::constants::*;
use crate::errors::{LighterError, Result, FFIError};
use crate::types::common::ffisigner;
use crate::types::common::{self, parse_result};
use serde::{Deserialize, Serialize};
use std::ffi::{c_int, c_longlong, CStr, CString};

/// Create Order Transaction Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOrderTxReq {
    pub market_index: u8,
    pub client_order_index: i64,
    pub base_amount: i64,
    pub price: u32,
    pub is_ask: u8,
    pub order_type: u8,
    pub time_in_force: u8,
    pub reduce_only: u8,
    pub trigger_price: u32,
    pub order_expiry: i64,
}

/// L2 Create Order Transaction Info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L2CreateOrderTxInfo {
    pub account_index: i64,
    pub api_key_index: u8,
    pub order_info: OrderInfo,
    pub expired_at: i64,
    pub nonce: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signed_hash: Option<String>,
}

impl TxInfo for L2CreateOrderTxInfo {
    fn get_tx_type(&self) -> u8 {
        TX_TYPE_L2_CREATE_ORDER
    }

    fn get_tx_info(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }

    fn get_tx_hash(&self) -> Option<String> {
        self.signed_hash.clone()
    }

    fn validate(&self) -> Result<()> {
        // Validate account index
        if self.account_index < MIN_ACCOUNT_INDEX {
            return Err(LighterError::AccountIndexTooLow(self.account_index));
        }
        if self.account_index > MAX_ACCOUNT_INDEX {
            return Err(LighterError::AccountIndexTooHigh(self.account_index));
        }

        // Validate API key index
        if self.api_key_index > MAX_API_KEY_INDEX {
            return Err(LighterError::ApiKeyIndexTooHigh(self.api_key_index));
        }

        // Validate order info
        self.validate_order_info()?;

        // Validate nonce
        if self.nonce < MIN_NONCE {
            return Err(LighterError::NonceTooLow(self.nonce));
        }

        Ok(())
    }

    fn hash(&self) -> Result<String> {
        // DONE: Implement Poseidon2 hashing
        // This should hash all fields using the Goldilocks field
        let hash_or_err = unsafe {
            ffisigner::SignCreateOrder(
                self.order_info.market_index as i32,
                self.order_info.client_order_index as i64,
                self.order_info.base_amount as i64,
                self.order_info.price as i32,
                self.order_info.is_ask as c_int,
                self.order_info.order_type as c_int,
                self.order_info.time_in_force as c_int,
                self.order_info.reduce_only as c_int,
                self.order_info.trigger_price as i32,
                self.order_info.order_expiry as c_longlong,
                1,
            )
        };
        parse_result(hash_or_err)
    }
}

impl L2CreateOrderTxInfo {
    fn validate_order_info(&self) -> Result<()> {
        let order = &self.order_info;

        // Market index
        if order.market_index > MAX_MARKET_INDEX {
            return Err(LighterError::MarketIndexTooHigh(order.market_index));
        }

        // Price
        if order.price < MIN_ORDER_PRICE {
            return Err(LighterError::PriceTooLow(order.price));
        }

        // IsAsk
        if order.is_ask != 0 && order.is_ask != 1 {
            return Err(LighterError::IsAskInvalid);
        }

        Ok(())
    }
}

/// Cancel Order Transaction Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelOrderTxReq {
    pub market_index: u8,
    pub index: i64,
}

/// Modify Order Transaction Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModifyOrderTxReq {
    pub market_index: u8,
    pub index: i64,
    pub base_amount: i64,
    pub price: u32,
    pub trigger_price: u32,
}

/// Cancel All Orders Transaction Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelAllOrdersTxReq {
    pub time_in_force: u8,
    pub time: i64,
}

/// Create Grouped Orders Transaction Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGroupedOrdersTxReq {
    pub grouping_type: u8,
    pub orders: Vec<CreateOrderTxReq>,
}

/// L2 Cancel Order Transaction Info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L2CancelOrderTxInfo {
    pub account_index: i64,
    pub api_key_index: u8,
    pub market_index: u8,
    pub index: i64,
    pub expired_at: i64,
    pub nonce: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signed_hash: Option<String>,
}

impl TxInfo for L2CancelOrderTxInfo {
    fn get_tx_type(&self) -> u8 {
        TX_TYPE_L2_CANCEL_ORDER
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
        if self.market_index > MAX_MARKET_INDEX {
            return Err(LighterError::MarketIndexTooHigh(self.market_index));
        }
        if self.nonce < MIN_NONCE {
            return Err(LighterError::NonceTooLow(self.nonce));
        }
        Ok(())
    }

    fn hash(&self) -> Result<String> {
        // DONE: Implement Poseidon2 hashing
        let hash_or_err =
            unsafe { ffisigner::SignCancelOrder(self.market_index as i32, self.index, self.nonce) };
        parse_result(hash_or_err)
    }
}

/// L2 Modify Order Transaction Info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L2ModifyOrderTxInfo {
    pub account_index: i64,
    pub api_key_index: u8,
    pub market_index: u8,
    pub index: i64,
    pub base_amount: i64,
    pub price: u32,
    pub trigger_price: u32,
    pub expired_at: i64,
    pub nonce: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signed_hash: Option<String>,
}

impl TxInfo for L2ModifyOrderTxInfo {
    fn get_tx_type(&self) -> u8 {
        TX_TYPE_L2_MODIFY_ORDER
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
        if self.nonce < MIN_NONCE {
            return Err(LighterError::NonceTooLow(self.nonce));
        }
        Ok(())
    }

    fn hash(&self) -> Result<String> {
        // DONE: Implement Poseidon2 hashing
        let hash_or_err = unsafe {
            ffisigner::SignModifyOrder(
                self.market_index as i32,
                self.index as i64,
                self.base_amount as i64,
                self.price as i64,
                self.trigger_price as i64,
                self.nonce,
            )
        };
        parse_result(hash_or_err)
    }
}

/// L2 Cancel All Orders Transaction Info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L2CancelAllOrdersTxInfo {
    pub account_index: i64,
    pub api_key_index: u8,
    pub time_in_force: u8,
    pub time: i64,
    pub expired_at: i64,
    pub nonce: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signed_hash: Option<String>,
}

impl TxInfo for L2CancelAllOrdersTxInfo {
    fn get_tx_type(&self) -> u8 {
        TX_TYPE_L2_CANCEL_ALL_ORDERS
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
        if self.nonce < MIN_NONCE {
            return Err(LighterError::NonceTooLow(self.nonce));
        }
        Ok(())
    }

    fn hash(&self) -> Result<String> {
        // DONE: Implement Poseidon2 hashing
        let hash_or_err = unsafe {
            ffisigner::SignCancelAllOrders(self.time_in_force as c_int, self.time, self.nonce)
        };
        parse_result(hash_or_err)
    }
}

/// L2 Create Grouped Orders Transaction Info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L2CreateGroupedOrdersTxInfo {
    pub account_index: i64,
    pub api_key_index: u8,
    pub grouping_type: u8,
    pub orders: Vec<OrderInfo>,
    pub expired_at: i64,
    pub nonce: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signed_hash: Option<String>,
}

//helper function for GroupedOrdersTxInfo
//type conversion form the lighter-rs OrderInfo to ffisigner struct is paramount 
impl From<common::OrderInfo> for ffisigner::CreateOrderTxReq {
    fn from(order_info: OrderInfo) -> ffisigner::CreateOrderTxReq {
        ffisigner::CreateOrderTxReq {
            MarketIndex: order_info.market_index,
            ClientOrderIndex: order_info.client_order_index,
            BaseAmount: order_info.base_amount,
            Price: order_info.price,
            IsAsk: order_info.is_ask,
            Type: order_info.order_type,
            TimeInForce: order_info.time_in_force,
            ReduceOnly: order_info.reduce_only,
            TriggerPrice: order_info.trigger_price,
            OrderExpiry: order_info.order_expiry,
        }
    }
}

impl TxInfo for L2CreateGroupedOrdersTxInfo {
    fn get_tx_type(&self) -> u8 {
        TX_TYPE_L2_CREATE_GROUPED_ORDERS
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
        if self.orders.len() > MAX_GROUPED_ORDER_COUNT as usize {
            return Err(LighterError::OrderGroupSizeInvalid);
        }
        if self.nonce < MIN_NONCE {
            return Err(LighterError::NonceTooLow(self.nonce));
        }
        Ok(())
    }

    fn hash(&self) -> Result<String> {
        // DONE: Implement Poseidon2 hashing
        // TODO: Extensive review required as the structure of the function is undesireable
        let mut vec_createordertxreq: Vec<ffisigner::CreateOrderTxReq> = self
            .orders
            .clone()
            .into_iter()
            .map(|order_info| ffisigner::CreateOrderTxReq::from(order_info))
            .collect();

        let orders_len = vec_createordertxreq.len();
        let orders_ptr = vec_createordertxreq.as_mut_ptr();
        let hash_or_err = unsafe {
            ffisigner::SignCreateGroupedOrders(
                self.grouping_type as u8,
                orders_ptr,
                orders_len as i32,
                self.nonce,
            )
        };
        parse_result(hash_or_err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_valid_order_info() -> OrderInfo {
        OrderInfo {
            market_index: 0,
            client_order_index: 1,
            base_amount: 1000000,
            price: 100000000,
            is_ask: 0,
            order_type: ORDER_TYPE_LIMIT,
            time_in_force: TIME_IN_FORCE_GOOD_TILL_TIME,
            reduce_only: 0,
            trigger_price: 0,
            order_expiry: 0,
        }
    }

    #[test]
    fn test_create_order_validation_success() {
        let tx_info = L2CreateOrderTxInfo {
            account_index: 12345,
            api_key_index: 0,
            order_info: create_valid_order_info(),
            expired_at: 1000000,
            nonce: 1,
            signed_hash: None,
        };

        assert!(tx_info.validate().is_ok());
    }

    #[test]
    fn test_create_order_account_index_too_low() {
        let tx_info = L2CreateOrderTxInfo {
            account_index: -1,
            api_key_index: 0,
            order_info: create_valid_order_info(),
            expired_at: 1000000,
            nonce: 1,
            signed_hash: None,
        };

        let result = tx_info.validate();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LighterError::AccountIndexTooLow(_)
        ));
    }

    #[test]
    fn test_create_order_account_index_too_high() {
        let tx_info = L2CreateOrderTxInfo {
            account_index: MAX_ACCOUNT_INDEX + 1,
            api_key_index: 0,
            order_info: create_valid_order_info(),
            expired_at: 1000000,
            nonce: 1,
            signed_hash: None,
        };

        let result = tx_info.validate();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LighterError::AccountIndexTooHigh(_)
        ));
    }

    #[test]
    fn test_create_order_api_key_index_too_high() {
        let tx_info = L2CreateOrderTxInfo {
            account_index: 12345,
            api_key_index: 255,
            order_info: create_valid_order_info(),
            expired_at: 1000000,
            nonce: 1,
            signed_hash: None,
        };

        let result = tx_info.validate();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LighterError::ApiKeyIndexTooHigh(_)
        ));
    }

    #[test]
    fn test_create_order_price_too_low() {
        let mut order_info = create_valid_order_info();
        order_info.price = 0;

        let tx_info = L2CreateOrderTxInfo {
            account_index: 12345,
            api_key_index: 0,
            order_info,
            expired_at: 1000000,
            nonce: 1,
            signed_hash: None,
        };

        let result = tx_info.validate();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LighterError::PriceTooLow(_)));
    }

    #[test]
    fn test_create_order_is_ask_invalid() {
        let mut order_info = create_valid_order_info();
        order_info.is_ask = 2;

        let tx_info = L2CreateOrderTxInfo {
            account_index: 12345,
            api_key_index: 0,
            order_info,
            expired_at: 1000000,
            nonce: 1,
            signed_hash: None,
        };

        let result = tx_info.validate();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LighterError::IsAskInvalid));
    }

    #[test]
    fn test_create_order_nonce_too_low() {
        let tx_info = L2CreateOrderTxInfo {
            account_index: 12345,
            api_key_index: 0,
            order_info: create_valid_order_info(),
            expired_at: 1000000,
            nonce: -1,
            signed_hash: None,
        };

        let result = tx_info.validate();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LighterError::NonceTooLow(_)));
    }

    #[test]
    fn test_create_order_tx_type() {
        let tx_info = L2CreateOrderTxInfo {
            account_index: 12345,
            api_key_index: 0,
            order_info: create_valid_order_info(),
            expired_at: 1000000,
            nonce: 1,
            signed_hash: None,
        };

        assert_eq!(tx_info.get_tx_type(), TX_TYPE_L2_CREATE_ORDER);
    }

    #[test]
    fn test_cancel_order_validation_success() {
        let tx_info = L2CancelOrderTxInfo {
            account_index: 12345,
            api_key_index: 0,
            market_index: 0,
            index: 123456,
            expired_at: 1000000,
            nonce: 1,
            signed_hash: None,
        };

        assert!(tx_info.validate().is_ok());
        assert_eq!(tx_info.get_tx_type(), TX_TYPE_L2_CANCEL_ORDER);
    }

    #[test]
    fn test_cancel_order_market_index_too_high() {
        let tx_info = L2CancelOrderTxInfo {
            account_index: 12345,
            api_key_index: 0,
            market_index: 255,
            index: 123456,
            expired_at: 1000000,
            nonce: 1,
            signed_hash: None,
        };

        let result = tx_info.validate();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LighterError::MarketIndexTooHigh(_)
        ));
    }

    #[test]
    fn test_modify_order_validation_success() {
        let tx_info = L2ModifyOrderTxInfo {
            account_index: 12345,
            api_key_index: 0,
            market_index: 0,
            index: 123456,
            base_amount: 2000000,
            price: 105000000,
            trigger_price: 0,
            expired_at: 1000000,
            nonce: 1,
            signed_hash: None,
        };

        assert!(tx_info.validate().is_ok());
        assert_eq!(tx_info.get_tx_type(), TX_TYPE_L2_MODIFY_ORDER);
    }

    #[test]
    fn test_cancel_all_orders_validation_success() {
        let tx_info = L2CancelAllOrdersTxInfo {
            account_index: 12345,
            api_key_index: 0,
            time_in_force: CANCEL_ALL_IMMEDIATE,
            time: 1000000,
            expired_at: 1000000,
            nonce: 1,
            signed_hash: None,
        };

        assert!(tx_info.validate().is_ok());
        assert_eq!(tx_info.get_tx_type(), TX_TYPE_L2_CANCEL_ALL_ORDERS);
    }

    #[test]
    fn test_create_grouped_orders_validation_success() {
        let tx_info = L2CreateGroupedOrdersTxInfo {
            account_index: 12345,
            api_key_index: 0,
            grouping_type: GROUPING_TYPE_ONE_CANCELS_THE_OTHER,
            orders: vec![create_valid_order_info(), create_valid_order_info()],
            expired_at: 1000000,
            nonce: 1,
            signed_hash: None,
        };

        assert!(tx_info.validate().is_ok());
        assert_eq!(tx_info.get_tx_type(), TX_TYPE_L2_CREATE_GROUPED_ORDERS);
    }

    #[test]
    fn test_create_grouped_orders_too_many_orders() {
        let tx_info = L2CreateGroupedOrdersTxInfo {
            account_index: 12345,
            api_key_index: 0,
            grouping_type: GROUPING_TYPE_ONE_CANCELS_THE_OTHER,
            orders: vec![
                create_valid_order_info(),
                create_valid_order_info(),
                create_valid_order_info(),
                create_valid_order_info(),
            ],
            expired_at: 1000000,
            nonce: 1,
            signed_hash: None,
        };

        let result = tx_info.validate();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LighterError::OrderGroupSizeInvalid
        ));
    }

    #[test]
    fn test_tx_info_serialization() {
        let tx_info = L2CreateOrderTxInfo {
            account_index: 12345,
            api_key_index: 0,
            order_info: create_valid_order_info(),
            expired_at: 1000000,
            nonce: 1,
            signed_hash: None,
        };

        let json_result = tx_info.get_tx_info();
        assert!(json_result.is_ok());

        let json = json_result.unwrap();
        assert!(json.contains("account_index"));
        assert!(json.contains("12345"));
    }
}
