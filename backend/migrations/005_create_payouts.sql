-- Migration 005: Create payouts table
CREATE TABLE IF NOT EXISTS payouts (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    circle_id       UUID NOT NULL REFERENCES circles(id),
    cycle_number    INTEGER NOT NULL,
    recipient_id    UUID NOT NULL REFERENCES users(id),
    amount          NUMERIC(18,7) NOT NULL,
    tx_hash         VARCHAR(128),
    executed_at     TIMESTAMPTZ,
    UNIQUE(circle_id, cycle_number)
);

CREATE INDEX IF NOT EXISTS idx_payouts_circle_id ON payouts(circle_id);
