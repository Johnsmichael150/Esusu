-- Migration 004: Create contributions table
CREATE TABLE IF NOT EXISTS contributions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id),
    circle_id       UUID NOT NULL REFERENCES circles(id),
    cycle_number    INTEGER NOT NULL,
    status          VARCHAR(10) NOT NULL DEFAULT 'pending',  -- pending | paid | defaulted
    tx_hash         VARCHAR(128),
    created_at      TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(user_id, circle_id, cycle_number)
);

CREATE INDEX IF NOT EXISTS idx_contributions_circle_cycle ON contributions(circle_id, cycle_number);
