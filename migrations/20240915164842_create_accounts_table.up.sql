CREATE TABLE accounts (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    balance BYTEA NOT NULL,
    bank_id INTEGER NULL DEFAULT NULL,
    bank_account_number integer NULL DEFAULT NULL,
    bank_account_digit integer NULL DEFAULT NULL,
    bank_agency_number integer NULL DEFAULT NULL,
    bank_agency_digit integer NULL DEFAULT NULL,
    bank_account_type integer NULL DEFAULT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP
);

CREATE TRIGGER update_accounts_updated_at
BEFORE UPDATE ON accounts
FOR EACH ROW
    EXECUTE FUNCTION update_updated_at();
