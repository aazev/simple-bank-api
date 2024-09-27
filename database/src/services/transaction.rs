use crate::repositories::{
    accounts::AccountRepository, transactions::TransactionRepository, users::UserRepository,
};

#[derive(Debug)]
pub struct Service {
    account_repository: AccountRepository,
    transaction_repository: TransactionRepository,
    user_repository: UserRepository,
}

impl Service {
    pub fn new() -> Self {
        Self {
            account_repository: AccountRepository::new(),
            transaction_repository: TransactionRepository::new(),
            user_repository: UserRepository::new(),
        }
    }
}
