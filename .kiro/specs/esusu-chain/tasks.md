# Implementation Plan: EsusuChain

## Overview

Incremental implementation of the EsusuChain savings circles platform. Tasks are ordered so each step builds on the previous: project scaffolding → smart contract → backend API → frontend → notifications → property-based tests. No code is left unintegrated.

---

## Tasks

- [x] 1. Project scaffolding and shared configuration
  - [x] 1.1 Initialize Next.js frontend with TypeScript and Tailwind CSS (mobile-first config)
    - `npx create-next-app@latest frontend --typescript --tailwind`
    - Configure `next.config.js` for mobile-first viewport defaults
    - _Requirements: 7.1_
  - [x] 1.2 Initialize Node.js/Express backend with TypeScript
    - Set up `tsconfig.json`, `src/` structure, `nodemon` dev script
    - Install `express`, `pg`, `dotenv`, `zod` for validation
    - _Requirements: 1.2, 2.1_
  - [x] 1.3 Initialize Soroban smart contract project
    - `stellar contract init esusu-contract --name esusu`
    - Add `proptest` and `soroban-sdk` to `Cargo.toml`
    - _Requirements: 2.2_
  - [x] 1.4 Set up PostgreSQL schema migrations
    - Create migration files for `users`, `circles`, `members`, `contributions`, `payouts` tables
    - Add `invite_code` unique index and `(user_id, circle_id)` unique constraint on members
    - _Requirements: 1.2, 2.1, 2.4_
  - [x] 1.5 Configure environment variables and shared constants
    - `.env.example` for both frontend and backend (Stellar network, Twilio, DB URL)
    - Shared USDC asset code and issuer constants

- [x] 2. Soroban smart contract — core state and initialization
  - [x] 2.1 Define contract data types and storage keys
    - Implement `CircleState`, `CircleStatus`, `MemberStatus`, `PayoutEntry`, `PayoutOrder`, `ContractError` enums and structs
    - _Requirements: 2.2, 6.1_
  - [x] 2.2 Implement `initialize` function
    - Validate `contribution_amount > 0`, `max_members` in [2, 50], `cycle_length_days >= 1`, `deposit_amount >= 0`
    - Store initial `CircleState` with status `Pending`
    - _Requirements: 2.2, 2.5, 2.6, 10.1_
  - [x] 2.3 Implement `join` function
    - Reject if circle is full (`CircleFull`) or caller already a member (`AlreadyMember`)
    - Require deposit transfer via USDC SAC before confirming membership
    - Assign next sequential payout position (fixed mode) or defer to shuffle (randomized mode)
    - When final member joins, finalize payout order and set status to `Active`, open first `Cycle_Window`
    - _Requirements: 3.2, 3.3, 3.4, 3.5, 3.6, 10.2_
  - [x] 2.4 Implement payout order finalization
    - Fixed mode: positions already assigned in join order — no-op at activation
    - Randomized mode: shuffle `members` vec using on-chain entropy at activation, write to `payout_order`
    - Emit event with finalized `payout_order`
    - _Requirements: 6.2, 6.3, 6.4_
  - [x] 2.5 Implement `contribute` function
    - Reject if `CircleNotActive`, outside `Cycle_Window` (`WindowClosed`), wrong amount (`WrongAmount`), or already paid (`AlreadyPaid`)
    - Record `contributions[(member, current_cycle)] = true`
    - _Requirements: 4.1, 4.3, 4.4, 4.5_
  - [x] 2.6 Implement `try_payout` function
    - Return `false` (no-op) if any member has not contributed (`NotAllPaid`)
    - Transfer `contribution_amount × member_count` USDC to the current cycle's payout recipient
    - Advance `current_cycle`, reset `Cycle_Window`, emit payout event
    - If final cycle completed, set status to `Completed`
    - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5_
  - [x] 2.7 Implement default flagging logic
    - When `Cycle_Window` expires with unpaid members, mark them as `Defaulter` in contract state
    - Hold their staked deposit; block payout until organizer resolves via off-chain flow
    - _Requirements: 5.5, 10.3, 10.4_
  - [x] 2.8 Implement `claim_deposit` function
    - Allow member to call after circle `Completed` and member has no defaults
    - Transfer `deposit_amount` USDC back to member's wallet
    - _Requirements: 10.5_
  - [x] 2.9 Implement read-only query functions
    - `get_state`, `get_member_status`, `get_payout_schedule`
    - _Requirements: 7.3, 9.1_
  - [ ]* 2.10 Write proptest property tests for contract — P1, P2, P4, P5, P7
    - **Property 1: Contribution amount invariant** — for any valid circle config and member, `contribute` only succeeds when `amount == contribution_amount`
      - `Feature: esusu-chain, Property 1: contribution amount invariant`
      - _Validates: Requirements 4.1, 4.5_
    - **Property 2: Payout equals pooled contributions** — for any cycle where all members paid, payout amount == `contribution_amount × member_count`
      - `Feature: esusu-chain, Property 2: payout equals pooled contributions`
      - _Validates: Requirements 5.1_
    - **Property 4: No payout before all contributions** — for any cycle with at least one unpaid member, `try_payout` returns false and no USDC is transferred
      - `Feature: esusu-chain, Property 4: no payout before all contributions`
      - _Validates: Requirements 5.1, 5.5_
    - **Property 5: Deposit round trip** — for any member who completes all cycles without defaulting, `claim_deposit` returns exactly `deposit_amount` to their wallet
      - `Feature: esusu-chain, Property 5: deposit round trip`
      - _Validates: Requirements 10.5_
    - **Property 7: Cycle advancement monotonicity** — for any sequence of valid operations, `current_cycle` never decreases
      - `Feature: esusu-chain, Property 7: cycle advancement monotonicity`
      - _Validates: Requirements 5.3_

- [x] 3. Checkpoint — smart contract tests pass
  - Build contract with `cargo build` and run `cargo test`. Ensure all unit and property tests pass before proceeding.

- [ ] 4. Backend API — user and authentication
  - [ ] 4.1 Implement `POST /api/users` — register or login by wallet address
    - Upsert user row by `wallet_address`; return existing user if already registered
    - _Requirements: 1.1, 1.2, 1.4_
  - [ ] 4.2 Implement `PUT /api/users/:id/phone` — update phone number
    - Validate E.164 format with regex before saving
    - _Requirements: 1.3, 1.5_
  - [ ]* 4.3 Write fast-check property test — phone number validation
    - For any string that is not valid E.164, the endpoint must reject it with 400
    - `Feature: esusu-chain, Property 8 (partial): member count bounds / input validation`
    - _Requirements: 1.5_

- [ ] 5. Backend API — circle lifecycle
  - [ ] 5.1 Implement `POST /api/circles` — create circle
    - Validate: `contribution_amount > 0`, `max_members` in [2, 50], `cycle_length_days >= 1`
    - Generate cryptographically random `invite_code` (URL-safe, 16 chars)
    - Insert circle row; trigger Soroban contract deploy via Stellar SDK (server-side keypair signs deploy tx)
    - Assign creator as first member with `payout_position = 1`
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6_
  - [ ] 5.2 Implement `GET /api/circles/:id` — get circle details
    - Return circle row joined with member count
    - _Requirements: 3.1, 7.1_
  - [ ] 5.3 Implement `POST /api/circles/:id/join` — join circle
    - Check circle not full (409) and user not already a member (409)
    - Insert member row; call contract `join` via Stellar SDK
    - If final member joined, update circle status to `active` in DB
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6_
  - [ ] 5.4 Implement `GET /api/circles/:id/members` — list members with contribution status
    - Join members + contributions for current cycle
    - _Requirements: 7.2_
  - [ ]* 5.5 Write fast-check property test — P8: member count bounds
    - For any circle config, the API must reject `max_members < 2` or `max_members > 50` with 400
    - For any circle at capacity, `POST /api/circles/:id/join` must return 409
    - `Feature: esusu-chain, Property 8: member count bounds`
    - _Validates: Requirements 2.5, 3.4_
  - [ ]* 5.6 Write fast-check property test — P9: invite code uniqueness
    - For any N circle creation requests, all generated invite codes must be distinct
    - `Feature: esusu-chain, Property 9: invite code uniqueness`
    - _Validates: Requirements 2.4_

- [ ] 6. Backend API — contributions and payouts
  - [ ] 6.1 Implement `POST /api/contributions` — record confirmed contribution
    - Called after frontend confirms on-chain tx; validate `tx_hash` not already recorded
    - Upsert contribution row with status `paid` and `tx_hash`
    - _Requirements: 4.2, 9.2_
  - [ ] 6.2 Implement `GET /api/circles/:id/transactions` — transaction history
    - Return contributions and payouts for circle with `tx_hash` and Stellar explorer URL
    - _Requirements: 9.3, 9.4_
  - [ ] 6.3 Implement `POST /api/webhooks/stellar` — Horizon event handler
    - Parse incoming Horizon event; identify contribution or payout transactions by contract address
    - Sync contribution status and payout records to DB
    - Trigger notification dispatch after payout events
    - _Requirements: 4.2, 5.2, 8.2_
  - [ ]* 6.4 Write fast-check property test — P6: contribution status consistency
    - For any set of simulated Horizon events, every member marked "paid" in DB must have a corresponding confirmed tx hash
    - `Feature: esusu-chain, Property 6: contribution status consistency`
    - _Validates: Requirements 4.2, 9.2_

- [ ] 7. Checkpoint — backend API tests pass
  - Run the full API test suite. All unit and property tests must pass before proceeding to frontend.

- [ ] 8. Frontend — wallet connection and onboarding
  - [ ] 8.1 Implement wallet connection flow using `@stellar/freighter-api`
    - `ConnectWallet` component: detect Freighter, request public key, call `POST /api/users`
    - Store wallet address in React context / Zustand store
    - _Requirements: 1.1, 1.2_
  - [ ] 8.2 Implement phone number registration form
    - After wallet connect, prompt for phone number with E.164 hint
    - Call `PUT /api/users/:id/phone`; show validation error inline
    - _Requirements: 1.3, 1.5_
  - [ ] 8.3 Implement auth guard — redirect unauthenticated users to `/`
    - _Requirements: 1.1_

- [ ] 9. Frontend — create and join circle
  - [ ] 9.1 Implement `/circles/new` — create circle form
    - Fields: name, contribution amount, deposit amount, cycle length, max members, payout order (Fixed / Randomized)
    - On submit: call `POST /api/circles`, then redirect to `/circles/[id]`
    - _Requirements: 2.1, 2.5, 2.6, 6.1, 10.1_
  - [ ] 9.2 Implement invite link display and copy-to-clipboard
    - Show shareable URL `https://<host>/circles/join/<inviteCode>` after circle creation
    - _Requirements: 2.4_
  - [ ] 9.3 Implement `/circles/join/[inviteCode]` — join circle flow
    - Fetch circle details via `GET /api/circles/:id`; display name, amount, cycle length, member count
    - "Join Circle" button calls `POST /api/circles/:id/join`
    - Show error states: circle full, already a member
    - _Requirements: 3.1, 3.4, 3.5_

- [ ] 10. Frontend — group dashboard
  - [ ] 10.1 Implement `/circles/[id]` — group dashboard page
    - Display: circle name, current cycle / total cycles, days remaining in Cycle_Window
    - _Requirements: 7.1_
  - [ ] 10.2 Implement contribution status table
    - Fetch `GET /api/circles/:id/members`; show each member's paid/unpaid status for current cycle
    - _Requirements: 7.2_
  - [ ] 10.3 Implement payout schedule display
    - Show ordered list of members with their payout position and cycle number
    - _Requirements: 7.3_
  - [ ] 10.4 Implement transaction history with explorer links
    - Fetch `GET /api/circles/:id/transactions`; render each tx hash as a link to `stellar.expert` or Horizon explorer
    - _Requirements: 7.4, 9.3_
  - [ ] 10.5 Implement real-time dashboard polling
    - Poll `GET /api/circles/:id/members` every 15 seconds; update contribution status without full page reload
    - _Requirements: 7.5_

- [ ] 11. Frontend — contribute USDC flow
  - [ ] 11.1 Implement "Contribute" button and Soroban transaction signing
    - Build and sign `contribute` contract invocation via Stellar SDK + Freighter
    - On confirmation, call `POST /api/contributions` with `tx_hash`
    - _Requirements: 4.1, 4.2_
  - [ ] 11.2 Implement contribution error handling
    - Map contract errors to user-friendly messages: `WrongAmount`, `WindowClosed`, `AlreadyPaid`
    - _Requirements: 4.4, 4.5_
  - [ ] 11.3 Implement deposit staking flow on join
    - Before confirming join, prompt member to sign deposit transfer transaction
    - _Requirements: 10.2_

- [ ] 12. Notifications — SMS and in-app
  - [ ] 12.1 Implement Twilio SMS service module
    - Wrap Twilio REST client; expose `sendSms(to: string, body: string)`
    - Use mock client in test environment
    - _Requirements: 8.4_
  - [ ] 12.2 Implement cycle-open notification dispatch
    - After circle becomes active or new cycle opens, send "It's your turn to pay — [Circle Name], Cycle [N]" to all unpaid members with a phone number
    - _Requirements: 8.1_
  - [ ] 12.3 Implement payout notification dispatch
    - After Horizon webhook confirms payout, send "You received your payout — [Amount] USDC from [Circle Name]" to recipient
    - _Requirements: 8.2_
  - [ ] 12.4 Implement 24-hour reminder notification
    - Scheduled job (cron): query circles with Cycle_Window closing in < 24 hours; send reminder to unpaid members
    - _Requirements: 8.3_
  - [ ] 12.5 Implement in-app notification store
    - Backend endpoint `GET /api/notifications` returns unread alerts for logged-in user
    - Frontend polls and renders notification badge / toast
    - _Requirements: 8.5_
  - [ ]* 12.6 Write fast-check property test — P10: notification delivery coverage
    - For any circle with N members and M unpaid members at cycle open, exactly M notifications are dispatched — no member skipped, no duplicates
    - `Feature: esusu-chain, Property 10: notification delivery coverage`
    - _Validates: Requirements 8.1_

- [ ] 13. Frontend — payout position and P3 property test
  - [ ] 13.1 Wire payout order finalization to dashboard
    - After circle goes active, fetch and display finalized payout schedule from `get_payout_schedule`
    - _Requirements: 6.4_
  - [ ]* 13.2 Write fast-check property test — P3: payout position uniqueness
    - For any circle with N members (2 ≤ N ≤ 50), the finalized payout order must be a permutation of [1..N] with no duplicates
    - `Feature: esusu-chain, Property 3: payout position uniqueness`
    - _Validates: Requirements 3.2, 6.2, 6.3_

- [ ] 14. Checkpoint — full integration test pass
  - Run all tests (contract, API, frontend). Verify the full contribution cycle integration test: create circle → all members join and stake deposit → all contribute → payout executes → deposit returned. Ask the user if any questions arise  - [ ] 1.4 Set up PostgreSQL schema migrations
.

- [ ] 15. Wiring and final integration
  - [ ] 15.1 Connect Horizon webhook to notification dispatch pipeline
    - Ensure `POST /api/webhooks/stellar` triggers SMS and in-app notifications end-to-end
    - _Requirements: 5.2, 8.2_
  - [ ] 15.2 Wire default handling end-to-end
    - Horizon webhook detects expired Cycle_Window with unpaid members → update DB defaulter flag → notify organizer via SMS
    - _Requirements: 5.5, 10.3, 10.4_
  - [ ] 15.3 Wire deposit return flow end-to-end
    - After circle completes, frontend shows "Claim Deposit" button for eligible members; signs `claim_deposit` contract call
    - _Requirements: 10.5_
  - [ ] 15.4 Add Stellar explorer links throughout UI
    - Ensure every `tx_hash` in dashboard and transaction history renders as a clickable `stellar.expert` link
    - _Requirements: 9.3_

- [ ] 16. Final checkpoint — all tests pass
  - Run the complete test suite across contract, API, and frontend. Ensure all property tests run ≥ 100 iterations. Ask the user if any questions arise before considering the implementation complete.

---

## Notes

- Sub-tasks marked with `*` are optional and can be skipped for a faster MVP
- Each property test must run a minimum of 100 iterations
- Property tests use **proptest** (Rust) for contract properties (P1, P2, P4, P5, P7) and **fast-check** (TypeScript) for API/frontend properties (P3, P6, P8, P9, P10)
- All on-chain interactions use Stellar Futurenet for development and testing
- The smart contract is the source of truth for funds; the backend only mirrors state for dashboard and notifications
