-- Migration 003: Create members table
CREATE TABLE IF NOT EXISTS members (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id),
    circle_id       UUID NOT NULL REFERENCES circles(id),
    payout_position INTEGER,
    deposit_paid    BOOLEAN DEFAULT FALSE,
    is_defaulter    BOOLEAN DEFAULT FALSE,
    UNIQUE(user_id, circle_id)
);

CREATE INDEX IF NOT EXISTS idx_members_circle_id ON members(circle_id);
CREATE INDEX IF NOT EXISTS idx_members_user_id ON members(user_id);
