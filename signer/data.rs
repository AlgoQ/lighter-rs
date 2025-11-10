use crate::{signer::ffi::ffisigner};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum GroupingType {
    OneTriggersOther = 1,
    OneCancelsOther = 2,
    OneTriggersAndOneCancelsOther = 3,
}

#[derive(Debug)]
pub struct TxInfo {
    pub data: Option<TxInfoData>,
    pub payload: String, // tx_info
}

#[derive(Debug)]
pub struct TxInfoData {
    pub message: String,
    pub signature: String,
}

#[derive(Debug)]
pub enum TxData {
    ChangePubKey(ChangePubKeyData),
    //SwitchApiKey(SwitchApiKeyData), // I don't think it's strictly necessary to have it in. Leaving it out for now.
    CreateOrder(CreateOrderData),
    SignCreateGroupedOrders(SignCreateGroupedOrdersData),
    SignCancelOrder(SignCancelOrderData),
    SignWithdraw(SignWithdrawData),
    SignCreateSubaccount,
    SignCancelAllOrders(SignCancelAllOrdersData),
    SignModifyOrder(SignModifyOrderData),
    SignTransfer(SignTransferData),
    SignCreatePublicPool(SignCreatePublicPoolData),
    SignUpdatePublicPool(SignUpdatePublicPoolData),
    SignMintShares(SignMintSharesData),
    SignBurnShares(SignBurnSharesData),
    SignUpdateLeverage(SignUpdateLeverageData),
    SignUpdateMargin(SignUpdateMarginData),
}

// ------------------ Requests data structs -------------------

#[derive(Debug)]
pub struct ChangePubKeyData {
    pub new_pubk: String,
}

//NEVER change the type of this data as the function signals match those on the linked C header
#[derive(Debug)]
pub struct CreateOrderData {
    pub market_index: i32,
    pub client_order_index: i64,
    pub base_amount: i64,
    pub price: i32,
    pub is_ask: bool,
    pub order_type: u8,
    pub time_in_force: u8,
    pub reduce_only: bool,
    pub trigger_price: i32,
    pub order_expiry: i64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TimeInForce {
    Gtc, // Good Till Cancelled
    Ioc, // Immediate Or Cancel
    Fok, // Fill Or Kill
    Day, // Good For Day
}

#[derive(Debug)]
pub struct SignCreateGroupedOrdersData {
    pub grouping_type: GroupingType,
    pub orders: Vec<ffisigner::CreateOrderTxReq>,
}

#[derive(Debug)]
pub struct SignCancelOrderData {
    pub market_index: i32,
    pub order_index: i64,
}

#[derive(Debug)]
pub struct SignWithdrawData {
    pub usdc_amount: i64,
}

#[derive(Debug)]
pub struct SignCancelAllOrdersData {
    pub time_in_force: u8,
    pub time: i64,
}

#[derive(Debug)]
pub struct SignModifyOrderData {
    pub market_index: i32,
    pub order_index: i64,
    pub amount: i64,
    pub price: i64,
    pub trigger_price: i64,
}

#[derive(Debug)]
pub struct SignTransferData {
    pub to_account_index: i64,
    pub usdc_amount: i64,
    pub fee: i64,
    pub memo: [u8; 32],
}

#[derive(Debug)]
pub struct SignCreatePublicPoolData {
    pub operator_fee: i64,
    pub initial_total_shares: i64,
    pub min_operator_share_rate: i64,
}

#[derive(Debug)]
pub struct SignUpdatePublicPoolData {
    pub public_pool_index: i64,
    pub status: i32,
    pub operator_fee: i64,
    pub min_operator_share_rate: i64,
}

#[derive(Debug)]
pub struct SignMintSharesData {
    pub public_pool_index: i64,
    pub share_amount: i64,
}

#[derive(Debug)]
pub struct SignBurnSharesData {
    pub public_pool_index: i64,
    pub share_amount: i64,
}

#[derive(Debug)]
pub struct SignUpdateLeverageData {
    pub market_index: i32,
    pub initial_margin_fraction: i32,
    pub margin_mode: i32,
}

#[derive(Debug)]
pub struct SignUpdateMarginData {
    pub market_index: i32,
    pub usdc_amount: i64,
    pub direction: i32,
}


#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Side {
    Buy,
    Sell,
}

impl Side {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Buy => "BUY",
            Self::Sell => "SELL",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderType {
    Market,
    Limit,
    StopLoss,
    TakeProfit,
}

impl OrderType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Market => "MARKET",
            Self::Limit => "LIMIT",
            Self::StopLoss => "STOP_LOSS",
            Self::TakeProfit => "TAKE_PROFIT",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderStatus {
    Pending,
    Open,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
}

impl OrderStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "PENDING",
            Self::Open => "OPEN",
            Self::PartiallyFilled => "PARTIALLY_FILLED",
            Self::Filled => "FILLED",
            Self::Cancelled => "CANCELLED",
            Self::Rejected => "REJECTED",
        }
    }
}

// TODO! consider adding enum
/*
    USDC_TICKER_SCALE = 1e6

    TX_TYPE_CHANGE_PUB_KEY = 8
    TX_TYPE_CREATE_SUB_ACCOUNT = 9
    TX_TYPE_CREATE_PUBLIC_POOL = 10
    TX_TYPE_UPDATE_PUBLIC_POOL = 11
    TX_TYPE_TRANSFER = 12
    TX_TYPE_WITHDRAW = 13
    TX_TYPE_CREATE_ORDER = 14
    TX_TYPE_CANCEL_ORDER = 15
    TX_TYPE_CANCEL_ALL_ORDERS = 16
    TX_TYPE_MODIFY_ORDER = 17
    TX_TYPE_MINT_SHARES = 18
    TX_TYPE_BURN_SHARES = 19
    TX_TYPE_UPDATE_LEVERAGE = 20
    TX_TYPE_CREATE_GROUP_ORDER = 28

    ORDER_TYPE_LIMIT = 0
    ORDER_TYPE_MARKET = 1
    ORDER_TYPE_STOP_LOSS = 2
    ORDER_TYPE_STOP_LOSS_LIMIT = 3
    ORDER_TYPE_TAKE_PROFIT = 4
    ORDER_TYPE_TAKE_PROFIT_LIMIT = 5
    ORDER_TYPE_TWAP = 6

    ORDER_TIME_IN_FORCE_IMMEDIATE_OR_CANCEL = 0
    ORDER_TIME_IN_FORCE_GOOD_TILL_TIME = 1
    ORDER_TIME_IN_FORCE_POST_ONLY = 2

    CANCEL_ALL_TIF_IMMEDIATE = 0
    CANCEL_ALL_TIF_SCHEDULED = 1
    CANCEL_ALL_TIF_ABORT = 2

    NIL_TRIGGER_PRICE = 0
    DEFAULT_28_DAY_ORDER_EXPIRY = -1
    DEFAULT_IOC_EXPIRY = 0
    DEFAULT_10_MIN_AUTH_EXPIRY = -1
    MINUTE = 60

    CROSS_MARGIN_MODE  = 0
    ISOLATED_MARGIN_MODE = 1

    GROUPING_TYPE_ONE_TRIGGERS_THE_OTHER = 1
    GROUPING_TYPE_ONE_CANCELS_THE_OTHER = 2 
    GROUPING_TYPE_ONE_TRIGGERS_A_ONE_CANCELS_THE_OTHER = 3


*/