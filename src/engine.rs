use std::collections::HashMap;

use crate::{
    account::Account,
    error::Result,
    transaction::{Transaction, TransactionType, TxRecord},
};

pub struct PaymentsEngine {
    pub accounts: HashMap<u16, Account>,
    pub transactions: HashMap<u32, TxRecord>,
}

impl PaymentsEngine {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            transactions: HashMap::new(),
        }
    }

    pub fn process_tx(&mut self, tx: &Transaction) -> Result<()> {
        match tx.tx_type {
            TransactionType::Deposit => self.process_deposit(&tx),
            TransactionType::Withdrawal => self.process_withdrawal(&tx),
            TransactionType::Dispute => self.process_dispute(&tx),
            TransactionType::Resolve => self.process_resolve(&tx),
            TransactionType::Chargeback => self.process_chargeback(&tx),
        }
    }

    fn process_deposit(&mut self, tx: &Transaction) -> Result<()> {
        let account = self
            .accounts
            .entry(tx.account_id)
            .or_insert(Account::new(tx.account_id));
        let tx_info = TxRecord::try_from(tx)?;

        account.deposit(tx_info.amount)?;
        self.transactions.insert(tx.tx_id, tx_info);

        Ok(())
    }

    fn process_withdrawal(&mut self, tx: &Transaction) -> Result<()> {
        let account = self
            .accounts
            .entry(tx.account_id)
            .or_insert(Account::new(tx.account_id));
        let tx_info = TxRecord::try_from(tx)?;

        account.withdrawal(tx_info.amount)?;
        self.transactions.insert(tx.tx_id, tx_info.into());

        Ok(())
    }

    fn process_dispute(&mut self, tx: &Transaction) -> Result<()> {
        let account = self
            .accounts
            .entry(tx.account_id)
            .or_insert(Account::new(tx.account_id));
        match self.transactions.get(&tx.tx_id) {
            Some(tx_info) => {
                // ensure tx belongs to the same account
                account.validate_tx_account_id(tx_info.account_id)?;
                account.dispute(tx_info.amount)?;

                Ok(())
            }
            // tx not found--ignore
            None => Ok(()),
        }
    }

    fn process_resolve(&mut self, tx: &Transaction) -> Result<()> {
        let account = self
            .accounts
            .entry(tx.account_id)
            .or_insert(Account::new(tx.account_id));
        match self.transactions.get(&tx.tx_id) {
            Some(tx_info) => {
                // ensure tx belongs to the same account
                account.validate_tx_account_id(tx_info.account_id)?;
                account.resolve(tx_info.amount)?;

                Ok(())
            }
            // tx not found--ignore
            None => Ok(()),
        }
    }

    fn process_chargeback(&mut self, tx: &Transaction) -> Result<()> {
        let account = self
            .accounts
            .entry(tx.account_id)
            .or_insert(Account::new(tx.account_id));
        match self.transactions.get(&tx.tx_id) {
            Some(tx_info) => {
                // ensure tx belongs to the same account
                account.validate_tx_account_id(tx_info.account_id)?;
                account.chargeback(tx_info.amount)?;

                Ok(())
            }
            // tx not found--ignore
            None => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::{Transaction, TransactionType};
    use rust_decimal::{Decimal, dec};

    fn new_tx(
        tx_type: TransactionType,
        account_id: u16,
        tx_id: u32,
        amount: Option<Decimal>,
    ) -> Transaction {
        Transaction {
            tx_type,
            account_id,
            tx_id,
            amount,
        }
    }

    fn new_engine_with_deposit(account_id: u16, tx_id: u32, amount: Decimal) -> PaymentsEngine {
        let mut engine = PaymentsEngine::new();
        let deposit = new_tx(TransactionType::Deposit, account_id, tx_id, Some(amount));
        engine.process_tx(&deposit).unwrap();

        engine
    }

    #[test]
    fn test_deposit_success() {
        let mut engine = PaymentsEngine::new();
        let deposit_tx = new_tx(TransactionType::Deposit, 1, 1, Some(dec!(100)));

        engine.process_tx(&deposit_tx).unwrap();

        let account = engine.accounts.get(&1).unwrap();
        assert_eq!(account.available, dec!(100));
        assert_eq!(account.held, dec!(0));
    }

    #[test]
    fn test_withdrawal_success() {
        let mut engine = new_engine_with_deposit(1, 1, dec!(100));
        let withdrawal_tx = new_tx(TransactionType::Withdrawal, 1, 2, Some(dec!(60)));

        engine.process_tx(&withdrawal_tx).unwrap();

        let account = engine.accounts.get(&1).unwrap();
        assert_eq!(account.available, dec!(40));
    }

    #[test]
    fn test_dispute_success() {
        let mut engine = new_engine_with_deposit(1, 1, dec!(100));
        let dispute_tx = new_tx(TransactionType::Dispute, 1, 1, None);

        engine.process_tx(&dispute_tx).unwrap();

        let account = engine.accounts.get(&1).unwrap();
        assert_eq!(account.available, dec!(0));
        assert_eq!(account.held, dec!(100));
    }

    #[test]
    fn test_resolve_success() {
        let mut engine = new_engine_with_deposit(1, 1, dec!(100));
        let dispute_tx = new_tx(TransactionType::Dispute, 1, 1, None);
        let resolve_tx = new_tx(TransactionType::Resolve, 1, 1, None);

        engine.process_tx(&dispute_tx).unwrap();
        engine.process_tx(&resolve_tx).unwrap();

        let account = engine.accounts.get(&1).unwrap();
        assert_eq!(account.available, dec!(100));
        assert_eq!(account.held, dec!(0));
    }

    #[test]
    fn test_chargeback_success() {
        let mut engine = new_engine_with_deposit(1, 1, dec!(100));
        let dispute_tx = new_tx(TransactionType::Dispute, 1, 1, None);
        let chargeback_tx = &new_tx(TransactionType::Chargeback, 1, 1, None);

        engine.process_tx(&dispute_tx).unwrap();
        engine.process_tx(&chargeback_tx).unwrap();

        let account = engine.accounts.get(&1).unwrap();
        assert_eq!(account.available, dec!(0));
        assert_eq!(account.held, dec!(0));
        assert!(account.locked);
    }
}
