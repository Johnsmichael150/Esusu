-- Migration 002: Create circles table
CREATE TABLE IF NOT EXISTS circles (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    contract_address    VARCHAR(64) UNIQUE NOT NULL,
    name                VARCHAR(100) NOT NULL,
    contribution_amount NUMERIC(18,7) NOT NULL,  -- USDC
    deposit_amount      NUMERIC(18,7) NOT NULL DEFAULT 0,
    cycle_length_days   INTEGER NOT NULL,
    max_members         INTEGER NOT NULL,
    payout_order        VARCHAR(10) NOT NULL,     -- 'fixed' | 'randomized'
    status              VARCHAR(10) NOT NULL DEFAULT 'pending',  -- pending | active | completed
    current_cycle       INTEGER NOT NULL DEFAULT 0,
    invite_code         VARCHAR(32) UNIQUE NOT NULL,
    creator_id          UUID REFERENCES users(id),
    created_at          TIMESTAMPTZ DEFAULT NOW()
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_circles_invite_code ON circles(invite_code);
