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

-- DEFAULT ADMIN USER
-- password: bank.administrator

INSERT INTO public.users (id,"name",email,active,"password",encryption_key,created_at,updated_at) VALUES
	 ('019210d1-6a92-7c53-ae18-6f02e8451f2c'::uuid,'admin','admin@localhost',true,'$argon2id$v=19$m=19456,t=2,p=1$6HfhkcBdwg9LZOS3htjb5Q$9Uy99egHhLb88w1HCHLmzvjlDmZ/LSJgyIIfjrx/XyQ',decode('EBB8BA4D31EC099A6E438DCFEAA41A2DC15F0923F247E8BC2A72744B0725942F035689719CE4CB52EE567AC6EBB79BBB758B2C60E3C8C6E3F273E65E','hex'),'2024-09-20 19:03:32.755893',NULL);
