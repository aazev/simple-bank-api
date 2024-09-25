CREATE TYPE transaction_type AS ENUM (
    'deposit',
    'withdrawal',
    'transfer',
    'payment',
    'fee',
    'interest'
);

CREATE TABLE transactions (
    id UUID PRIMARY KEY,
    type transaction_type NOT NULL,
    from_account_id UUID NULL REFERENCES accounts(id),
    to_account_id UUID NOT NULL REFERENCES accounts(id),
    amount BYTEA NOT NULL,
    timestamp TIMESTAMP NOT NULL
);

CREATE INDEX transactions_id_idx ON transactions(id);
CREATE INDEX transactions_type_idx ON transactions(type);
CREATE INDEX transactions_from_account_id_idx ON transactions(from_account_id);
CREATE INDEX transactions_to_account_id_idx ON transactions(to_account_id);
CREATE INDEX transactions_timestamp_idx ON transactions(timestamp);
