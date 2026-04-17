-- Migration 001: Create users table
CREATE TABLE IF NOT EXISTS users (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wallet_address  VARCHAR(64) UNIQUE NOT NULL,
    phone           VARCHAR(20),           -- E.164 format
    created_at      TIMESTAMPTZ DEFAULT NOW()
);
