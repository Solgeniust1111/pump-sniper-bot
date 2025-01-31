use bincode::Options;
use copy_trading_bot::common::utils::{
    create_arc_rpc_client, create_nonblocking_rpc_client, create_rpc_client, import_arc_wallet,
    import_wallet, AppState,
};
use copy_trading_bot::core::tx::jito_confirm;
use copy_trading_bot::engine::swap::pump_swap;
use dotenv::dotenv;
use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
use serde_json::Value;
use solana_sdk::bs58::encode::EncodeTarget;
use solana_sdk::message::VersionedMessage;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::VersionedTransaction;
use spl_associated_token_account::instruction;
use std::env;
use std::fs::File;
use std::io::{self, Read};
use tokio::time::Instant;
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};

#[derive(Serialize)]
struct SwapRequest {
    quoteResponse: serde_json::Value, // You may deserialize it into a specific struct if known
    userPublicKey: String,
    wrapAndUnwrapSol: bool,
    dynamicComputeUnitLimit: bool,
    prioritizationFeeLamports: u64,
}
#[tokio::main]

async fn main() {
    dotenv().ok();

    let mut file = match File::open("./target.txt") {
        Ok(data) => data,
        Err(e) => {
            println!("target.txt file not");
            return;
        }
    };

    // Create a String to store the contents of the file
    let mut contents = String::new();

    // Read the file contents into the string
    let data = file.read_to_string(&mut contents).unwrap();
    if data != 44 {
        println!("target pubkey is not correct");
        return;
    }

    let unwanted_key = env::var("JUP_PUBKEY").expect("JUP_PUBKEY not set");
    let target = contents;
    let ws_url = env::var("RPC_WEBSOCKET_ENDPOINT").expect("RPC_WEBSOCKET_ENDPOINT not set");

    let (ws_stream, _) = connect_async(ws_url)
        .await
        .expect("Failed to connect to WebSocket server");
    let (mut write, mut read) = ws_stream.split();
    // Subscribe to logs
    let subscription_message = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "transactionSubscribe",
        "params": [

            {
                "failed": false,
                "accountInclude": [ "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P"],
                "accountExclude": [unwanted_key],
                // Optionally specify accounts of interest
            },
            {
                "commitment": "processed",
                "encoding": "jsonParsed",
                "transactionDetails": "full",
                "maxSupportedTransactionVersion": 0
            }
        ]
    });

    write
        .send(subscription_message.to_string().into())
        .await
        .expect("Failed to send subscription message");

    // Listen for messages
    while let Some(Ok(msg)) = read.next().await {
        if let WsMessage::Text(text) = msg {
            let json: Value = serde_json::from_str(&text).unwrap();

            let sig = json["params"]["result"]["signature"].to_string();

            if let Some(accountKeys) = json["params"]["result"]["transaction"]["transaction"]
                ["message"]["accountKeys"]
                .as_array()
            {
                let mut flag = false;
                for accountKey in accountKeys.iter() {
                    if accountKey["signer"] == true && accountKey["pubkey"] == target {
                        flag = true;
                        break;
                    } else {
                        flag = false;
                    }
                }
                if flag == true {
                    for key in accountKeys.iter() {

                        }
                    }
                }
            }
        }
    }
}




