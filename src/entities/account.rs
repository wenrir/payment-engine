//! Account sepcific data structs and implementations
use serde::{Serialize, Serializer};

use crate::errors::AccountError;
#[derive(Serialize, Default, Debug)]
/// Account data.
pub(crate) struct Account {
    /// The owner of the account.
    pub(crate) client: u16,
    /// Total funds available for the account.
    #[serde(serialize_with = "serialize_round_float")]
    pub(crate) available: f64,
    /// The total funds that are held for dispute.
    #[serde(serialize_with = "serialize_round_float")]
    pub(crate) held: f64,
    /// Total funds that are available or held.
    #[serde(serialize_with = "serialize_round_float")]
    pub(crate) total: f64,
    /// Whether the account is locked.
    pub(crate) locked: bool,
}
/// Creates a member function to the Account struct.
/// The function created with the macro contains code for operating on the account balances.
/// Inputs:
/// $name = name of function to create, e.g. deposit
/// (target, operation) = target on self to do operation on, e.g. (total, +=) becomes `self.total +=`
/// [self, amount] = These are here to make it possible to use inside of the assert function, please use [self, amount]
/// cond = condition to be true in order for operations to go through.
/// asserts = expands to a function body, that has access to self and amount. Anything can be placed here but the intention is for assertions.
/// For example
/// ``` rust
/// modify_account_balance_fn!(dispute, ((held, +=), (available, -=)),[self, amount], self.available - amount >= 0_f64, {});
/// ```
/// expands to
/// ``` rust
/// pub(crate) fn dispute(&mut self, amount: &f64) -> Result<(), AccountError> {
/// {}
/// if !self.locked && self.available - amount >= 0_f64 {
///    self.held += amount;
///    self.available -= amount;
/// }
/// Ok(())
/// }
/// ```
macro_rules! modify_account_balance_fn {
            ($name:ident, ($( ($target:ident, $operation:tt) ),*), [$self:ident, $amount:ident], $cond:expr, $asserts:block) => {
                pub(crate) fn $name(
                    &mut $self,
                    $amount: &f64
                ) -> Result<(), AccountError> {
                    // Perform assertions or checks before continuing with the operation
                    $asserts
                    // Check the condition and whether the account is locked
                    if !$self.locked && $cond {
                        // Loop through each target and perform the corresponding operation
                        $(
                            $self.$target $operation $amount;
                        )*

                        // Optionally, include a return statement with Ok
                    }

                    Ok(())
                }
            };
        }
impl Account {
    modify_account_balance_fn!(deposit, ((total, +=), (available, +=)), [self, amount], true, {
        assert!(self.total + amount <= f64::MAX, "Unable to add {:?} to account, as the total will overflow.", &amount);
        assert!(self.total + self.available <= f64::MAX, "Unable to add {:?} to account, as the total will overflow.", &amount);
    });
    modify_account_balance_fn!(withdrawl, ((total, -=), (available, -=)),[self, amount], self.available - amount >= 0_f64, {});
    modify_account_balance_fn!(dispute, ((held, +=), (available, -=)),[self, amount], self.available - amount >= 0_f64, {});
    modify_account_balance_fn!(resolve, ((held, -=), (available, +=)),[self, amount], self.held - amount >= 0_f64, {});
    pub(crate) fn chargeback(
        &mut self,
        amount: &f64,
    ) -> Result<(), AccountError> {
        if self.held - amount >= 0_f64 {
            self.locked = true;
            self.total -= amount;
            self.held -= amount;
        }
        Ok(())
    }

    pub(crate) fn new(client_id: u16) -> Self {
        Self {
            client: client_id,
            ..Default::default()
        }
    }
}
/// Serializes a float by rounding it to 4 decimals.
/// Trims away 0s and `.`, e.g 1.0000 => 1 while 1.5000 => 1.5
/// Percision requirement.
fn serialize_round_float<S>(f: &f64, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(
        format!("{:.4}", f)
            .as_str()
            .trim_end_matches('0')
            .trim_end_matches('.'),
    )
}
#[cfg(test)]
mod tests {
    use super::Account;

    fn create_account() -> Account {
        Account::new(1)
    }
    #[test]
    fn test_deposit() {
        let mut account = create_account();

        let res = account.deposit(&1_f64);
        assert!(res.is_ok());
        assert_eq!(account.available, 1_f64);
        assert_eq!(account.held, 0_f64);
        assert_eq!(account.total, 1_f64);
    }
    #[test]
    fn test_withdrawl() {
        let mut account = create_account();
        {
            let res = account.deposit(&1_f64);
            assert!(res.is_ok());
            assert_eq!(account.available, 1_f64);
            assert_eq!(account.held, 0_f64);
            assert_eq!(account.total, 1_f64);
        }
        {
            let res = account.withdrawl(&1_f64);
            assert!(res.is_ok());
            assert_eq!(account.available, 0_f64);
            assert_eq!(account.held, 0_f64);
            assert_eq!(account.total, 0_f64);
        }
    }
    #[test]
    fn test_dispute() {
        let mut account = create_account();
        {
            let res = account.deposit(&1_f64);
            assert!(res.is_ok());
            assert_eq!(account.available, 1_f64);
            assert_eq!(account.held, 0_f64);
            assert_eq!(account.total, 1_f64);
        }
        {
            let res = account.dispute(&1_f64);
            assert!(res.is_ok());
            assert_eq!(account.available, 0_f64);
            assert_eq!(account.held, 1_f64);
            assert_eq!(account.total, 1_f64);
        }
    }
    #[test]
    fn test_resolve() {
        let mut account = create_account();
        {
            let res = account.deposit(&1_f64);
            assert!(res.is_ok());
            assert_eq!(account.available, 1_f64);
            assert_eq!(account.held, 0_f64);
            assert_eq!(account.total, 1_f64);
        }
        {
            let res = account.dispute(&1_f64);
            assert!(res.is_ok());
            assert_eq!(account.available, 0_f64);
            assert_eq!(account.held, 1_f64);
            assert_eq!(account.total, 1_f64);
        }
        {
            let res = account.resolve(&1_f64);
            assert!(res.is_ok());
            assert_eq!(account.available, 1_f64);
            assert_eq!(account.held, 0_f64);
            assert_eq!(account.total, 1_f64);
        }
    }
    #[test]
    fn test_chargeback() {
        let mut account = create_account();
        {
            let res = account.deposit(&1_f64);
            assert!(res.is_ok());
            assert_eq!(account.available, 1_f64);
            assert_eq!(account.held, 0_f64);
            assert_eq!(account.total, 1_f64);
        }
        {
            let res = account.dispute(&1_f64);
            assert!(res.is_ok());
            assert_eq!(account.available, 0_f64);
            assert_eq!(account.held, 1_f64);
            assert_eq!(account.total, 1_f64);
        }
        {
            let res = account.chargeback(&1_f64);
            assert!(res.is_ok());
            assert_eq!(account.available, 0_f64);
            assert_eq!(account.held, 0_f64);
            assert_eq!(account.total, 0_f64);
            assert!(account.locked);
        }
    }
}
