CREATE TABLE users (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT NOT NULL UNIQUE,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    password TEXT NOT NULL,
    encryption_key BYTEA NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP
);

create index idx_user_name on users (name);
create index idx_user_email on users (email);
create index idx_user_active on users (active);
create index idx_user_created_at on users (created_at);

CREATE TRIGGER update_users_updated_at
BEFORE UPDATE ON users
FOR EACH ROW
    EXECUTE FUNCTION update_updated_at();
