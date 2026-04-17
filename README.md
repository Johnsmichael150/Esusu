# EsusuChain

Trustless savings circles on Stellar. EsusuChain digitizes the traditional African Ajo/Esusu practice — groups of members pool USDC contributions each cycle and rotate payouts, enforced by a Soroban smart contract with no admin custody of funds.

## How it works

1. A creator deploys a circle with a fixed contribution amount, cycle length, and member cap
2. Members join via invite link and stake a deposit to the smart contract
3. Each cycle, all members contribute USDC; the contract automatically pays out to the designated member
4. Payout order is either fixed (join order) or randomized at circle activation
5. Deposits are returned to members who complete all cycles without defaulting

## Stack

| Layer | Tech |
|-------|------|
| Frontend | Next.js 14, Tailwind CSS, Freighter wallet |
| Backend | Node.js, Express, PostgreSQL |
| Smart Contract | Rust, Soroban SDK 20 (Stellar) |
| Notifications | Twilio SMS |
| Testing | Jest + fast-check (API), proptest (contract) |

## Project structure

```
esusu-chain/
├── frontend/          # Next.js web app
├── backend/           # Express REST API
│   └── migrations/    # PostgreSQL schema migrations
└── esusu-contract/    # Soroban smart contract (Rust)
```

## Prerequisites

- Node.js 20+
- Rust + `soroban-cli`
- PostgreSQL
- A [Freighter](https://freighter.app) wallet (for testing)

## Getting started

### 1. Smart contract

```bash
cd esusu-contract
cargo build --target wasm32-unknown-unknown --release
```

To run contract tests:

```bash
cargo test
```

### 2. Backend

```bash
cd backend
cp .env.example .env   # fill in your values
npm install
npm run dev
```

Run migrations:

```bash
npm run migrate        # applies SQL files in backend/migrations/
```

Run tests:

```bash
npm test
```

### 3. Frontend

```bash
cd frontend
cp .env.example .env.local
npm install
npm run dev            # http://localhost:3000
```

## Environment variables

See `backend/.env.example` and `frontend/.env.example` for all required variables. Key ones:

- `DATABASE_URL` — PostgreSQL connection string
- `STELLAR_SERVER_SECRET_KEY` — keypair used to deploy contracts
- `USDC_ISSUER` — USDC issuer address on Stellar (Futurenet default is pre-filled)
- `TWILIO_*` — Twilio credentials for SMS notifications

## API overview

| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/users` | Register / login with wallet address |
| PUT | `/api/users/:id/phone` | Set phone number for SMS |
| POST | `/api/circles` | Create a new circle |
| GET | `/api/circles/:id` | Get circle details |
| POST | `/api/circles/:id/join` | Join a circle |
| GET | `/api/circles/:id/members` | Member list + contribution status |
| POST | `/api/contributions` | Record a confirmed on-chain contribution |
| GET | `/api/circles/:id/transactions` | Transaction history |
| POST | `/api/webhooks/stellar` | Stellar Horizon event webhook |

## Smart contract interface

```rust
fn initialize(env, creator, contribution_amount, cycle_length_days, max_members, deposit_amount, payout_order)
fn join(env, member) -> payout_position
fn contribute(env, member, amount)
fn try_payout(env) -> bool
fn claim_deposit(env, member)
fn get_state(env) -> CircleState
fn get_member_status(env, member) -> MemberStatus
fn get_payout_schedule(env) -> Vec<PayoutEntry>
```

## Network

The project targets Stellar Futurenet by default. To switch to testnet or mainnet, update `STELLAR_NETWORK`, `STELLAR_HORIZON_URL`, `STELLAR_RPC_URL`, and `STELLAR_NETWORK_PASSPHRASE` in your `.env`.
