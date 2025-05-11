# Stallion Bounty Platform Smart Contract

[![License](https://img.shields.io/badge/License-Boost_1.0-lightblue.svg)](https://www.boost.org/LICENSE_1_0.txt)

A decentralized bounty platform built on the Stellar network using Soroban smart contracts. This project allows users to create bounties, apply to them, and distribute rewards to winners in a trustless manner.

## Features

- Create bounties with custom rewards and deadlines
- Apply to bounties with submission links
- Select winners and distribute rewards automatically
- Fee distribution to platform administrators
- Time-based bounty status management (Open, Judging, Completed)
- Secure access control with admin functions

## Prerequisites

- Rust (latest stable version)
- Soroban CLI (latest version)
- Cargo (Rust's package manager)

## Getting Started

1. **Clone the repository**

   ```bash
   git clone https://github.com/your-username/stallion-contract.git
   cd stallion-contract
   ```

2. **Build the contract**

   ```bash
   make build
   ```

3. **Run tests**
   ```bash
   make test
   ```

## Contract Architecture

The smart contract is structured into several modules:

- `lib.rs`: Main contract implementation
- `storage.rs`: Storage management and data structures
- `events.rs`: Event definitions and emission
- `types.rs`: Custom types and enums
- `utils.rs`: Utility functions
- `test.rs`: Test suite

## Usage

### Initialization

Deploy the contract and initialize it with the required parameters:

```rust
    let contract_id = env.register(
        StallionContract {},
        (token.address.clone(), admin.clone(), fee_account.clone()),
    );
    let client = StallionContractClient::new(&env, &contract_id);
```

### Creating a Bounty

```rust
let distribution = vec![(7000, 3000)]; // 70% to 1st place, 30% to 2nd
let result = client.create_bounty(
    &bounty_owner,
    1000, // reward amount
    distribution,
    env.ledger().timestamp() + 7 * 24 * 60 * 60, // 1 week from now
    env.ledger().timestamp() + 14 * 24 * 60 * 60, // 2 weeks from now
    String::from_slice(&env, "Build a DEX interface")
);
```

### Applying to a Bounty

```rust
client.apply_to_bounty(&applicant, bounty_id, Symbol::new(&env, "github.com/example/submission"));
```

### Selecting Winners

```rust
let winners = vec![winner1, winner2];
client.select_winners(&bounty_owner, bounty_id, &winners);
```

## Security

- Admin-only functions are protected
- All operations include access control checks
- Time-based state transitions are enforced
- Input validation is performed for all user inputs

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the Boost Software License 1.0 - see the [LICENSE](LICENSE) file for details.
