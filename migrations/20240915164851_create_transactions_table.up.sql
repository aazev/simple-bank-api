CREATE TYPE transaction_operation AS ENUM (
    'deposit',
    'withdrawal',
    'transfer',
    'payment',
    'fee',
    'interest'
);

CREATE TABLE transactions (
    id UUID PRIMARY KEY,
    operation transaction_operation NOT NULL,
    from_account_id UUID NULL REFERENCES accounts(id),
    to_account_id UUID NOT NULL REFERENCES accounts(id),
    amount BYTEA NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX transactions_id_idx ON transactions(id);
CREATE INDEX transactions_operation_idx ON transactions(operation);
CREATE INDEX transactions_from_account_id_idx ON transactions(from_account_id);
CREATE INDEX transactions_to_account_id_idx ON transactions(to_account_id);
CREATE INDEX transactions_timestamp_idx ON transactions(created_at);
