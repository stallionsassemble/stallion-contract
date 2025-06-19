# Stallion Bounty Platform Smart Contract

[![License](https://img.shields.io/badge/License-Boost_1.0-lightblue.svg)](https://www.boost.org/LICENSE_1_0.txt)

A decentralized bounty platform built on the Stellar network using Soroban smart contracts. This project enables trustless creation, management, and reward distribution for bounties in a secure and transparent manner.

## 🌟 Features

### Bounty Management

- **Create Bounties**: Set up bounties with custom rewards, titles, and deadlines
- **Update Bounties**: Modify bounty details before the submission deadline
- **Delete Bounties**: Remove bounties that have no submissions
- **Time-based Status**: Automatic status tracking (Active, In Review, Completed)

### Bounty Participation

- **Apply to Bounties**: Submit your work for consideration
- **Update Submissions**: Refine your submission before the deadline
- **Multiple Winners Support**: Flexible reward distribution among multiple winners

### Reward System

- **Custom Reward Distribution**: Define percentage-based reward distribution for multiple winners
- **Automatic Payouts**: Secure and transparent reward distribution
- **Platform Fees**: Built-in fee mechanism for platform sustainability

### Administration

- **Admin Controls**: Secure management of platform settings
- **Fee Management**: Configurable fee account for platform revenue
- **Access Control**: Role-based permissions for all operations

### Security & Transparency

- **Immutable Records**: All actions are recorded on the blockchain
- **Transparent Judging**: Clear criteria and process for winner selection
- **Secure Fund Handling**: Funds are held in escrow until bounty completion

## 🏗️ Contract Architecture

The smart contract is organized into several modules:

- `lib.rs`: Core contract implementation and business logic
- `storage.rs`: Data persistence and storage management
- `events.rs`: Event definitions and emission for on-chain transparency
- `types.rs`: Custom data types and enums
- `utils.rs`: Helper functions and utilities
- `test.rs`: Comprehensive test suite

## 🛠️ Prerequisites

- Rust (latest stable version)
- Soroban CLI (latest version)
- Cargo (Rust's package manager)

## 🚀 Getting Started

1. **Clone the repository**
2. **Build the contract** using the provided Makefile
3. **Run tests** to verify everything works as expected

## 📝 Usage

### For Bounty Creators

- Deploy a new bounty with your desired reward and timeline
- Review submissions and select winners based on merit
- Manage bounty details and deadlines

### For Participants

- Browse active bounties
- Submit your work before the deadline
- Update your submission if needed
- Get rewarded for quality contributions

## 🔒 Security

- All critical operations are protected by access control
- Funds are securely escrowed in the contract
- Comprehensive input validation on all functions
- Time-based state transitions are strictly enforced
- Admin-only functions for platform management

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📄 License

This project is licensed under the Boost Software License 1.0 - see the [LICENSE](LICENSE) file for details.

stellar contract invoke \
--id CBEXRYEQIZMQDPVH3L5Q6EENMGTTX5PEFTVMPMLDGTW2E4QJ5GNXTTMX \
--source alice \
--network testnet \
-- \
 create_bounty \
 --owner=$(soroban keys public-key alice) \
 --token="CAQCFVLOBK5GIULPNZRGATJJMIZL5BSP7X5YJVMGCPTUEPFM4AVSRCJU" \
 --reward=100000 \
 --distribution='[(1,50),(2,50)]' \
 --submission_deadline=1723867200 \
 --judging_deadline=1723953600 \
 --title="Test bounty"
