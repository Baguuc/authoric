CREATE TABLE permissions (
    name VARCHAR(255) PRIMARY KEY,
    description VARCHAR(3000)
);

CREATE TABLE groups (
    name VARCHAR(255) PRIMARY KEY,
    description VARCHAR(3000)
);

CREATE TABLE groups_permissions (
    group_name VARCHAR(255) REFERENCES groups(name),
    permission_name VARCHAR(255) REFERENCES permissions(name)
);

CREATE TABLE users (
    login VARCHAR(255) PRIMARY KEY,
    -- Argon2 produces 32-byte long hashes
    password_hash VARCHAR(32),
    details JSON
);

CREATE TABLE users_groups (
    user_login VARCHAR(255) REFERENCES users(login),
    group_name VARCHAR(255) REFERENCES groups(name)
);

CREATE TABLE event (
    id INTEGER PRIMARY KEY,
    -- Types of event:
    -- PermissionGet
    -- PermissionCreate
    -- PermissionDelete
    -- GroupGet
    -- GroupCreate
    -- GroupDelete
    -- UserRegister
    -- UserLogin
    -- UserDelete
    -- the longest here is "PermissionCreate" and "PermissionDelete"
    -- and there are 16 chars long so we allocate 16 bytes for them
    type VARCHAR(16),
    -- Statues:
    -- Commited
    -- OnHold
    -- the longest is 8 chars long
    status VARCHAR(8),
    data JSON
);