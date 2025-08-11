use rust_decimal::Decimal;
use serde::Deserialize;

use crate::error::{Error, Result};

#[derive(Debug, Deserialize)]
pub struct Account {
    pub id: u16,
    pub available: Decimal,
    pub held: Decimal,
    pub total: Decimal,
    pub locked: bool,
}

impl Account {
    pub fn new(id: u16) -> Self {
        Self {
            id,
            available: Decimal::ZERO,
            held: Decimal::ZERO,
            total: Decimal::ZERO,
            locked: false,
        }
    }

    pub fn deposit(&mut self, amount: Decimal) -> Result<()> {
        self.check_lock()?;
        self.validate_deposit_amount(amount)?;

        let new_available = self
            .available
            .checked_add(amount)
            .ok_or_else(|| Error::TransactionError("Overflow Error: invalid deposit tx amount."))?;
        let new_total = self
            .total
            .checked_add(amount)
            .ok_or_else(|| Error::TransactionError("Overflow Error: invalid deposit tx amount."))?;

        self.available = new_available;
        self.total = new_total;

        Ok(())
    }

    pub fn withdrawal(&mut self, amount: Decimal) -> Result<()> {
        self.check_lock()?;
        self.validate_withdrawal_amount(amount)?;

        // theoretically all underflows should NEVER happen bc we always check for sufficient funds
        let new_available = self.available.checked_sub(amount).ok_or_else(|| {
            Error::TransactionError("Underflow Error: invalid withdrawal tx amount.")
        })?;
        let new_total = self.total.checked_sub(amount).ok_or_else(|| {
            Error::TransactionError("Underflow Error: invalid withdrawal tx amount.")
        })?;

        self.available = new_available;
        self.total = new_total;

        Ok(())
    }

    pub fn dispute(&mut self, amount: Decimal) -> Result<()> {
        self.check_lock()?;
        self.validate_dispute_amount(amount)?;

        let new_available = self.available.checked_sub(amount).ok_or_else(|| {
            Error::TransactionError("Underflow Error: invalid dispute tx amount.")
        })?;
        let new_held = self
            .held
            .checked_add(amount)
            .ok_or_else(|| Error::TransactionError("Overflow Error: invalid dispute tx amount."))?;

        self.available = new_available;
        self.held = new_held;

        Ok(())
    }

    pub fn resolve(&mut self, amount: Decimal) -> Result<()> {
        self.check_lock()?;
        self.validate_resolve_amount(amount)?;

        let new_held = self.held.checked_sub(amount).ok_or_else(|| {
            Error::TransactionError("Underflow Error: invalid resolve tx amount.")
        })?;
        let new_available = self
            .available
            .checked_add(amount)
            .ok_or_else(|| Error::TransactionError("Overflow Error: invalid resolve tx amount."))?;

        self.held = new_held;
        self.available = new_available;

        Ok(())
    }

    pub fn chargeback(&mut self, amount: Decimal) -> Result<()> {
        self.check_lock()?;
        self.validate_chargeback_amount(amount)?;

        let new_held = self.held.checked_sub(amount).ok_or_else(|| {
            Error::TransactionError("Underflow Error: invalid chargeback tx amount.")
        })?;
        let new_total = self.total.checked_sub(amount).ok_or_else(|| {
            Error::TransactionError("Underflow Error: invalid chargeback tx amount.")
        })?;

        self.held = new_held;
        self.total = new_total;
        self.locked = true; // lock account after successful chargeback

        Ok(())
    }

    fn check_lock(&self) -> Result<()> {
        if self.locked {
            return Err(Error::AccountError(
                "Account is locked. All transactions are currently unavailable.",
            ));
        }

        Ok(())
    }

    fn validate_deposit_amount(&self, amount: Decimal) -> Result<()> {
        Self::check_negative_amount(amount)?;

        Ok(())
    }

    fn validate_withdrawal_amount(&self, amount: Decimal) -> Result<()> {
        Self::check_negative_amount(amount)?;

        // ensure the account has enough available/total funds
        if self.available < amount || self.total < amount {
            return Err(Error::AccountError(
                "Insufficient funds to complete withdrawal transaction.",
            ));
        }

        Ok(())
    }

    fn validate_dispute_amount(&self, amount: Decimal) -> Result<()> {
        // ensure the account has enough available funds
        if self.available < amount {
            return Err(Error::AccountError(
                "Insufficient funds to complete dispute transaction.",
            ));
        }

        Ok(())
    }

    fn validate_resolve_amount(&self, amount: Decimal) -> Result<()> {
        // ensure the account has enough held funds
        if self.held < amount {
            return Err(Error::AccountError(
                "Insufficient funds to complete resolve transaction.",
            ));
        }

        Ok(())
    }

    fn validate_chargeback_amount(&self, amount: Decimal) -> Result<()> {
        // ensure the account has enough held/total funds
        if self.held < amount || self.total < amount {
            return Err(Error::AccountError(
                "Insufficient funds to complete chargeback transaction.",
            ));
        }

        Ok(())
    }

    // ensure the transaction account ID matches the account ID for disputes, resolves, and
    // chargebacks
    pub fn validate_tx_account_id(&self, tx_account_id: u16) -> Result<()> {
        if self.id != tx_account_id {
            return Err(Error::TransactionError(
                "Transaction account ID does not match account.",
            ));
        }

        Ok(())
    }

    // ensure that deposit/withdrawal amounts are not negative
    fn check_negative_amount(amount: Decimal) -> Result<()> {
        if amount.is_sign_negative() {
            return Err(Error::TransactionError(
                "Deposit/withdrawal amounts must be greater than zero.",
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::{Decimal, dec};

    #[test]
    fn test_deposit_success() {
        let mut account = Account::new(1);
        account.deposit(dec!(100)).unwrap();

        assert_eq!(account.available, dec!(100));
        assert_eq!(account.total, dec!(100));
        assert_eq!(account.held, dec!(0));
        assert!(!account.locked);
    }

    #[test]
    fn test_deposit_failure_overflow() {
        let mut account = Account::new(1);
        account.deposit(Decimal::ONE).unwrap();
        let result = account.deposit(Decimal::MAX);

        assert!(result.is_err());
    }

    #[test]
    fn test_deposit_failure_locked_account() {
        let mut account = Account::new(1);
        account.locked = true;
        let result = account.deposit(dec!(100));

        assert!(result.is_err());
    }

    #[test]
    fn test_withdrawal_success() {
        let mut account = Account::new(1);
        account.deposit(dec!(100)).unwrap();
        account.withdrawal(dec!(40)).unwrap();

        assert_eq!(account.available, dec!(60));
        assert_eq!(account.total, dec!(60));
    }

    #[test]
    fn test_withdrawal_failure_insufficient_funds() {
        let mut account = Account::new(1);
        account.deposit(dec!(50)).unwrap();
        let result = account.withdrawal(dec!(60));

        assert!(result.is_err());
        assert_eq!(account.available, dec!(50));
    }

    #[test]
    fn test_withdrawal_failure_locked_account() {
        let mut account = Account::new(1);
        account.deposit(dec!(100)).unwrap();
        account.locked = true;
        let result = account.withdrawal(dec!(10));

        assert!(result.is_err());
    }

    #[test]
    fn test_dispute_success() {
        let mut account = Account::new(1);
        account.deposit(dec!(100)).unwrap();
        account.dispute(dec!(60)).unwrap();

        assert_eq!(account.available, dec!(40));
        assert_eq!(account.held, dec!(60));
        assert_eq!(account.total, dec!(100));
    }

    #[test]
    fn test_dispute_failure_insufficient_available_funds() {
        let mut account = Account::new(1);
        account.deposit(dec!(60)).unwrap();

        let result = account.dispute(dec!(80));

        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_success() {
        let mut account = Account::new(1);
        account.deposit(dec!(100)).unwrap();
        account.dispute(dec!(60)).unwrap();

        assert_eq!(account.available, dec!(40));

        account.resolve(dec!(60)).unwrap();

        assert_eq!(account.available, dec!(100));
        assert_eq!(account.held, dec!(0));
        assert_eq!(account.total, dec!(100));
    }

    #[test]
    fn test_resolve_failure_insufficient_held_funds() {
        let mut account = Account::new(1);
        account.deposit(dec!(100)).unwrap();
        account.dispute(dec!(60)).unwrap();

        let result = account.resolve(dec!(80));

        assert!(result.is_err());
    }

    #[test]
    fn test_chargeback_success() {
        let mut account = Account::new(1);
        account.deposit(dec!(100)).unwrap();
        account.dispute(dec!(60)).unwrap();

        assert_eq!(account.total, dec!(100));
        assert!(!account.locked);

        account.chargeback(dec!(60)).unwrap();

        assert_eq!(account.total, dec!(40));
        assert_eq!(account.held, dec!(0));
        assert!(account.locked);
    }

    #[test]
    fn test_chargeback_failure_insufficient_held_funds() {
        let mut account = Account::new(1);
        account.deposit(dec!(100)).unwrap();
        account.dispute(dec!(60)).unwrap();

        let result = account.chargeback(dec!(80));

        assert!(result.is_err());
    }

    #[test]
    fn test_check_lock() {
        let mut account = Account::new(1);
        assert!(account.check_lock().is_ok());

        account.locked = true;
        assert!(account.check_lock().is_err());
    }

    #[test]
    fn test_check_negative_amount_success() {
        let result = Account::check_negative_amount(dec!(1));

        assert!(result.is_ok());
    }

    #[test]
    fn test_check_negative_amount_failure() {
        let result = Account::check_negative_amount(dec!(-1));

        assert!(result.is_err());
    }
}
