use rust_decimal::Decimal;
use serde::{self, Deserialize, Serialize};

use crate::error::{Error, Result};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Transaction {
    #[serde(rename = "type")]
    pub tx_type: TransactionType,
    #[serde(rename = "client")]
    pub account_id: u16,
    #[serde(rename = "tx")]
    pub tx_id: u32,
    pub amount: Option<Decimal>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Chargeback,
    Deposit,
    Dispute,
    Resolve,
    Withdrawal,
}

// lightweight tx type for storage
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TxRecord {
    // type not necessary here--keeping for sanity
    pub tx_type: TransactionType,
    pub account_id: u16,
    pub amount: Decimal,
}

impl TryFrom<&Transaction> for TxRecord {
    type Error = Error;

    fn try_from(tx: &Transaction) -> Result<Self> {
        Ok(TxRecord {
            tx_type: tx.tx_type,
            account_id: tx.account_id,
            amount: tx
                .amount
                .ok_or(Error::TransactionError("Invalid transaction amount."))?,
        })
    }
}
