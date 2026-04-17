# Requirements Document

## Introduction

EsusuChain digitizes traditional African savings circles (Ajo/Esusu) using USDC on the Stellar blockchain. The platform enables groups of members to pool contributions each cycle and rotate payouts to one member at a time — enforced by Soroban smart contracts rather than social trust. The MVP targets cross-border groups (Nigeria, UK, US diaspora) with a mobile-first web interface and SMS notification support.

## Glossary

- **Circle**: A savings group with a fixed set of members, contribution amount, and cycle schedule
- **Cycle**: One round of contributions where all members pay and one member receives the payout
- **Contribution**: A USDC payment made by a member to the Circle's smart contract for a given cycle
- **Payout**: The total pooled USDC released to the designated recipient member at the end of a cycle
- **Payout_Position**: The order in which a member receives the payout (fixed or randomized)
- **Defaulter**: A member who has not contributed by the end of a contribution window
- **Smart_Contract**: The Soroban contract on Stellar that holds funds, enforces rules, and executes payouts
- **Wallet**: A Stellar-compatible wallet holding USDC used to interact with the Smart_Contract
- **System**: The EsusuChain platform including frontend, backend API, and Smart_Contract layer
- **Member**: A user who has joined a Circle and has an assigned Payout_Position
- **Creator**: The user who creates and configures a Circle
- **Organizer**: The Creator acting in an administrative capacity for their Circle
- **Cycle_Window**: The time period during which contributions for a given cycle are accepted

---

## Requirements

### Requirement 1: User Onboarding and Wallet Connection

**User Story:** As a new user, I want to connect my Stellar wallet and register my phone number, so that I can participate in savings circles and receive notifications.

#### Acceptance Criteria

1. WHEN a user visits the platform for the first time, THE System SHALL present a wallet connection flow supporting Stellar-compatible wallets
2. WHEN a user connects a wallet, THE System SHALL associate the wallet address with a user account and store it in the off-chain database
3. WHEN a user account is created, THE System SHALL prompt the user to provide a phone number for SMS notifications
4. IF a wallet address is already registered, THEN THE System SHALL log the user in to the existing account without creating a duplicate
5. WHEN a user provides a phone number, THE System SHALL validate that the phone number is in E.164 international format before saving

---

### Requirement 2: Create Savings Circle

**User Story:** As a Creator, I want to configure and deploy a new savings circle, so that I can organize a group of members to save together.

#### Acceptance Criteria

1. WHEN a Creator submits a new Circle configuration, THE System SHALL record the Circle with: name, contribution amount in USDC, cycle length in days, and maximum member count
2. WHEN a Circle is created, THE System SHALL deploy a Smart_Contract on Stellar/Soroban that encodes the contribution amount, cycle length, and member count
3. WHEN a Circle is created, THE System SHALL assign the Creator as the first Member with Payout_Position 1
4. WHEN a Circle is created, THE System SHALL generate a unique invite link that allows other users to join the Circle
5. THE System SHALL enforce that the contribution amount is greater than 0 USDC and the member count is between 2 and 50 inclusive
6. THE System SHALL enforce that the cycle length is at least 1 day

---

### Requirement 3: Join Savings Circle

**User Story:** As a user, I want to join an existing savings circle via an invite link, so that I can participate in the group savings rotation.

#### Acceptance Criteria

1. WHEN a user follows a valid invite link, THE System SHALL display the Circle details including name, contribution amount, cycle length, and current member count
2. WHEN a user accepts the Circle rules and joins, THE System SHALL add the user as a Member and assign the next available Payout_Position
3. WHEN a user joins a Circle, THE System SHALL register the Member's wallet address with the Smart_Contract
4. IF a Circle has reached its maximum member count, THEN THE System SHALL prevent additional users from joining and display an informative message
5. IF a user is already a Member of the Circle, THEN THE System SHALL prevent duplicate membership and display an informative message
6. WHEN the final available Payout_Position is filled, THE System SHALL mark the Circle as active and begin the first Cycle_Window

---

### Requirement 4: Contribute USDC

**User Story:** As a Member, I want to send my USDC contribution for the current cycle, so that I fulfill my obligation and the payout can be released.

#### Acceptance Criteria

1. WHEN a Member initiates a contribution, THE System SHALL invoke a transaction sending the exact contribution amount in USDC from the Member's Wallet to the Smart_Contract
2. WHEN a contribution transaction is confirmed on Stellar, THE System SHALL record the contribution with status "paid" for that Member, cycle number, and group
3. WHILE a Cycle_Window is open, THE System SHALL allow any unpaid Member to submit their contribution
4. IF a Member attempts to contribute outside an active Cycle_Window, THEN THE System SHALL reject the transaction and display an informative message
5. IF a Member attempts to contribute an amount other than the exact contribution amount, THEN THE Smart_Contract SHALL reject the transaction
6. WHEN a Member's contribution is confirmed, THE System SHALL update the group dashboard in real time to reflect the payment status

---

### Requirement 5: Automatic Payout Execution

**User Story:** As a Member, I want the payout to be released automatically once all members have contributed, so that I can trust the process without relying on a central administrator.

#### Acceptance Criteria

1. WHEN all Members in a Circle have submitted confirmed contributions for the current cycle, THE Smart_Contract SHALL automatically release the total pooled USDC to the Member with the current cycle's Payout_Position
2. WHEN a payout is executed, THE System SHALL record the payout transaction hash and recipient in the off-chain database
3. WHEN a payout is executed, THE System SHALL advance the Circle to the next cycle and open a new Cycle_Window
4. WHEN the final cycle's payout is executed, THE System SHALL mark the Circle as completed
5. IF one or more Members have not contributed by the end of the Cycle_Window, THEN THE Smart_Contract SHALL flag those Members as Defaulters and delay the payout until the Organizer resolves the default

---

### Requirement 6: Payout Order

**User Story:** As a Creator, I want to choose how the payout order is determined, so that the rotation matches my group's preferences.

#### Acceptance Criteria

1. WHEN creating a Circle, THE Creator SHALL select a payout order mode: Fixed or Randomized
2. WHERE the payout order mode is Fixed, THE System SHALL assign Payout_Positions in the order members join the Circle
3. WHERE the payout order mode is Randomized, THE System SHALL assign Payout_Positions by randomly shuffling the member list once the Circle becomes active
4. WHEN Payout_Positions are finalized, THE System SHALL record them immutably in the Smart_Contract and display the full payout schedule to all Members

---

### Requirement 7: Group Dashboard

**User Story:** As a Member, I want to view my circle's status at a glance, so that I can track contributions, see who has paid, and know when my payout is scheduled.

#### Acceptance Criteria

1. WHEN a Member views the group dashboard, THE System SHALL display the current cycle number, total cycles, and days remaining in the Cycle_Window
2. WHEN a Member views the group dashboard, THE System SHALL display each Member's contribution status (paid / unpaid) for the current cycle
3. WHEN a Member views the group dashboard, THE System SHALL display the full payout schedule showing each Member's Payout_Position and the cycle in which they receive the payout
4. WHEN a Member views the group dashboard, THE System SHALL display a link to the Stellar blockchain explorer for each completed transaction
5. WHEN contribution statuses change, THE System SHALL update the dashboard without requiring a full page reload

---

### Requirement 8: Notifications

**User Story:** As a Member, I want to receive timely notifications about contribution deadlines and payouts, so that I never miss a payment or a payout.

#### Acceptance Criteria

1. WHEN a new Cycle_Window opens, THE System SHALL send a notification to all unpaid Members with the message "It's your turn to pay — [Circle Name], Cycle [N]"
2. WHEN a payout is executed, THE System SHALL send a notification to the recipient Member with the message "You received your payout — [Amount] USDC from [Circle Name]"
3. WHEN a Member has not contributed and the Cycle_Window has less than 24 hours remaining, THE System SHALL send a reminder notification to that Member
4. WHERE a Member has a registered phone number, THE System SHALL deliver notifications via SMS
5. THE System SHALL also deliver notifications via in-app alerts for Members who are logged in

---

### Requirement 9: Transparency and Immutable Records

**User Story:** As a Member, I want all transactions to be publicly verifiable on the blockchain, so that I can trust the platform is not manipulating funds.

#### Acceptance Criteria

1. THE Smart_Contract SHALL hold all pooled USDC funds without any admin withdrawal capability
2. WHEN any contribution or payout transaction occurs, THE System SHALL record the Stellar transaction hash and make it visible on the group dashboard
3. THE System SHALL provide a link to the Stellar blockchain explorer for every on-chain transaction
4. WHEN a Member queries their contribution history, THE System SHALL return all past contributions and payouts with cycle numbers and transaction hashes

---

### Requirement 10: Default Handling

**User Story:** As a Member, I want the system to protect the group from defaulting members, so that honest members are not penalized.

#### Acceptance Criteria

1. WHEN a Circle is created, THE Creator SHALL specify an upfront deposit amount in USDC that each Member must stake before the first cycle begins
2. WHEN a user joins a Circle, THE System SHALL require the user to transfer the upfront deposit to the Smart_Contract before the membership is confirmed
3. IF a Member defaults on a contribution, THEN THE Smart_Contract SHALL flag the Member as a Defaulter and record the default in the off-chain reputation log
4. IF a Member defaults, THEN THE Smart_Contract SHALL hold the Member's staked deposit and the Organizer SHALL be notified to resolve the default
5. WHEN a Member completes all contributions without defaulting, THE Smart_Contract SHALL return the staked deposit to the Member's Wallet at the end of the Circle
