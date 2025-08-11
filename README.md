# Payments Engine

A simple payments engine written in Rust.

## Overview
This project contains a CLI (bin) and three core abstractions that make up the core engine logic: `PaymentsEngine`, `Account`, and `Transaction`. These three types handle all operations surrounding account management, while the CLI handles all IO operations for transaction ingestion. Separating out the core engine logic from the CLI creates a separation of concerns, allowing for easier testing and maintainability (the core engine logic could become a library or the project itself could be turned into a workspace for greater modularity/reusability).

### PaymentsEngine
The `PaymentsEngine` is the orchestrator that routes transactions and maintains account/transaction state. The orchestrator is agnostic to account internals, keeping a separation of concerns.

### Account
An `Account` represents a single user's account in the system and is responsible for enforcing payment rules and updating its own account state by applying transactions. 

### Transaction
A `Transaction` is a single operation that can be applied to an account. It contains the transaction type, account ID, transaction ID, and amount.

## Design Assumptions
- A failed transaction does not fail the system--errors are logged to stderr and transaction processing continues.
- If an account is locked, no transactions can be applied to it.
- `Decimal` from [rust_decimal](https://docs.rs/rust_decimal/latest/rust_decimal/) is used for monetary values to avoid floating point precision issues, therefore amounts are assumed to be smaller than 96-bit integers.

## Testing
Unit tests were used to test the core engine logic (e.g. `engine.rs`/`account.rs` modules) to ensure correctness as well as to test against edge cases/errors. The CLI was tested with two CSVs (clean and dirty) to simulate system inputs and verify resulting outputs. The test CSVs used are located in `tests/fixtures/`.
