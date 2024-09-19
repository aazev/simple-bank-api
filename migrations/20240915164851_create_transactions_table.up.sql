CREATE TABLE transactions (
    id UUID PRIMARY KEY,
    from_account_id UUID NULL REFERENCES accounts(id),
    to_account_id UUID NOT NULL REFERENCES accounts(id),
    amount_nonce BYTEA NOT NULL,
    amount_ciphertext BYTEA NOT NULL,
    timestamp TIMESTAMP NOT NULL
);
