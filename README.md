# CLMM Swap Contract on Solana

This is a **Concentrated Liquidity Market Maker (CLMM)** implementation built on **Solana** using the **Anchor framework**. It allows users to perform token swaps with fine-grained liquidity ranges, similar to Uniswap V3.

---

## Overview

The CLMM Swap contract is designed to provide a flexible and efficient platform for token swaps on Solana. It allows users to swap tokens with a range of liquidity, similar to Uniswap V3. The contract uses a **tick-based** approach, where each tick represents a price range and liquidity is provided in terms of ticks.

---

## Architecture

The CLMM Swap contract is built using the Anchor framework, which provides a high-level interface for building Solana programs. The contract is composed of several modules:

- **State**: Defines the data structures and state variables for the contract.
- **Math**: Contains mathematical functions for computing prices, liquidity, and swap amounts.
- **Errors**: Defines custom error types for handling errors in the contract.
- **Instructions**: Defines the instructions for interacting with the contract, including initialize_pool, provide_liquidity, remove_liquidity, and swap.

---

## Usage

To use the CLMM Swap contract, you can follow these steps:

1. Clone the repository:

```bash
git clone https://github.com/darrendc26/clmm-dex.git
```

2. Navigate to the `clmm-dex` directory:    

```bash
cd clmm-dex
```

3. Install the dependencies:    

```bash
cargo install --path .
```

4. Build the contract:    

```bash
anchor build
```

5. Deploy the contract:    

```bash
anchor deploy
```

6. Initialize the pool:    

```bash
anchor idl init --filepath target/idl/clmm_dex.json --provider.cluster devnet --provider.wallet ~/.config/solana/id.json
```

7. Provide liquidity:    

```bash
anchor idl invoke --filepath target/idl/clmm_dex.json --provider.cluster devnet --provider.wallet ~/.config/solana/id.json --program-id <program-id> provide_liquidity --tick_lower <tick_lower> --tick_upper <tick_upper> --liquidity <liquidity>
```

8. Remove liquidity:    

```bash
anchor idl invoke --filepath target/idl/clmm_dex.json --provider.cluster devnet --provider.wallet ~/.config/solana/id.json --program-id <program-id> remove_liquidity
```

9. Swap:    

```bash
anchor idl invoke --filepath target/idl/clmm_dex.json --provider.cluster devnet --provider.wallet ~/.config/solana/id.json --program-id <program-id> swap --amount_in <amount_in> --a_to_b <a_to_b>
```

10. Check the contract state:    

```bash
anchor idl inspect --filepath target/idl/clmm_dex.json --provider.cluster devnet --program-id <program-id>
```

---

## Contributing

Contributions are welcome! If you find any issues or have suggestions for improvements, please open an issue or submit a pull request on the GitHub repository.

