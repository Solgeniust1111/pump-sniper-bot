# 🚀 **PumpFun Sniper Bot using Rust** 

Welcome to the **PumpFun Sniper Bot **! This bot watches for new `pump.fun` token mints on the Solana blockchain in real-time, making it the perfect tool to monitor token launches. 🌟

### 🎯 **Key Features**

- 🛰️ **Real-time WebSocket Streaming**: 
  Connects to Solana's blockchain through Helius RPC WebSocket and listens for new transactions, specifically targeting `pump.fun` mint instructions.
  
- 🔍 **Filter Pump.fun Token Mints**: 
  Filters transactions by program IDs and instruction discriminators related to `pump.fun`.

- 📊 **Formatted Data**: 
  Logs essential transaction details like the transaction signature, creator's wallet, and the minted token address when a new `pump.fun` token is detected.

- ⚡ **Efficient Stream Handling**: 
  Handles WebSocket stream events efficiently, ensuring no loss of data and continuous monitoring.

---

## 🚀 **Getting Started**

Follow these steps to get your **PumpFun Sniper Bot** up and running!

### Prerequisites

- Node.js version 16+ installed on your system
- A Solana wallet with access to the Helius RPC API (and your **API Token**)

### Installation

1. **Clone the Repository**:

    ```bash
    git clone https://github.com/yourusername/pumpfun-sniper-bot
    ```

2. **Install Dependencies**:

    Navigate to the project directory and run the following command:

    ```bash
    cd pumpfun-sniper-bot
    cargo build
    ```

3. **Configure API Token**:

    Replace the API token in the `ENDPOINT` variable:

    ```
    const ENDPOINT = "wss://atlas-mainnet.helius-rpc.com";
    const TOKEN = "YOUR_API_TOKEN";
    ```

4. **Run the Bot**:

    Start the bot by running:

    ```bash
    cargo run
    ```

---

## 🧑‍💻 **Code Overview**

### WebSocket Connection

The bot establishes a WebSocket connection to the Solana network using Helius RPC. It listens for real-time transaction events related to `pump.fun` token mints.

### Transaction Monitoring

By subscribing to the stream, the bot filters for transactions that match specific program IDs and instruction discriminators. It looks for the `InitializeMint2` instruction that signals the creation of a new `pump.fun` token.

### Event Handling

Once a matching transaction is detected, the bot formats the relevant transaction data and logs it, including:

- **Transaction Signature**: Unique identifier for the transaction
- **Creator Wallet**: Address of the token creator
- **Minted Token**: The address of the newly created `pump.fun` token

### Error Handling

The bot gracefully handles errors during the subscription process, ensuring robust performance with retries and log messages.

---

## 📈 **Example Output**

```bash
Geyser connection established - watching new Pump.fun mints.

======================================💊 New Pump.fun Mint Detected!======================================
┌────────────┬──────────────────────────────────────────────────────────────────┐
│  Signature │ '5Gx9A1sZ4jEXAMPLE_SIGNATURE'                                    │
│  Slot      │ '12748279'                                                       │
│  Creator   │ 'H8m57EXAMPLE_CREATOR_ADDRESS'                                   │
│  Token     │ 'SoTd2EXAMPLE_TOKEN_ADDRESS'                                     │
└────────────┴──────────────────────────────────────────────────────────────────┘
```

---

## ⚙️ **Bot Configuration**

You can modify the configuration and filters:

- **Program ID**: The Solana program associated with `pump.fun` token creation.
- **Instruction Discriminators**: Filters transactions based on the discriminator of the mint instruction.
- **Accounts to Include**: Customizes which accounts' data to extract from the transaction.

