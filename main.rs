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

    let mut file = match File::open("./src/target.txt") {
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
                "accountInclude": ["675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8", "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P"],
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
                        if key["pubkey"] == "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8" {
                            tx_ray(json.clone(), target.clone()).await;
                        }
                        if key["pubkey"] == "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P" {
                            tx_pump(json.clone(), target.clone()).await;
                        }
                    }
                }
            }
        }
    }
}

pub async fn tx_ray(json: Value, target: String) {
    if let Some(inner_instructions) =
        json["params"]["result"]["transaction"]["meta"]["innerInstructions"].as_array()
    {
        for inner_instruction in inner_instructions.iter() {
            // Try to extract the string representation of the log
            if let Some(instructions) = inner_instruction["instructions"].as_array() {
                for instruction in instructions {
                    if instruction["parsed"]["type"] == "transfer".to_string()
                        && instruction["program"] == "spl-token".to_string()
                        && instruction["parsed"]["info"]["authority"] == target
                    {
                        let start_time = Instant::now();

                        let amount_str = instruction["parsed"]["info"]["amount"]
                            .as_str()
                            .unwrap()
                            .to_string();
                        let amount_in = amount_str.parse::<u64>().unwrap();
                        let mut mint = "".to_string();
                        let mut mint_post_amount = 0_f64;
                        let mut mint_pre_amount = 0_f64;
                        let mut dirs = "".to_string();

                        if let Some(postTokenBalances) = json["params"]["result"]["transaction"]
                            ["meta"]["postTokenBalances"]
                            .as_array()
                        {
                            for postTokenBalance in postTokenBalances.iter() {
                                if postTokenBalance["owner"] == target.to_string() {
                                    mint =
                                        postTokenBalance["mint"].as_str().unwrap_or("").to_string();
                                    mint_post_amount = postTokenBalance["uiTokenAmount"]
                                        ["uiAmount"]
                                        .as_f64()
                                        .unwrap();
                                }
                            }
                        }
                        if let Some(preTokenBalances) = json["params"]["result"]["transaction"]
                            ["meta"]["preTokenBalances"]
                            .as_array()
                        {
                            for preTokenBalance in preTokenBalances.iter() {
                                if preTokenBalance["owner"] == target.to_string() {
                                    mint_pre_amount = preTokenBalance["uiTokenAmount"]["uiAmount"]
                                        .as_f64()
                                        .unwrap();
                                }
                            }
                        }

                        if mint_pre_amount < mint_post_amount {
                            dirs = "buy".to_string();
                            swap_on_jup(mint, dirs, amount_in).await;
                        } else {
                            dirs = "sell".to_string();
                            swap_on_jup(mint, dirs, amount_in).await;
                        }

                        break;
                    }
                }
            }
        }
    }
}

pub async fn tx_pump(json: Value, target: String) {
    // Iterate over logs and check for unwanted_key

    let mut amount_in = 0_u64;
    let mut mint = "".to_string();
    let mut mint_post_amount = 0_f64;
    let mut mint_pre_amount = 0_f64;
    let mut dirs = "".to_string();
    let mut pool_vault = "".to_string();

    if let Some(postTokenBalances) =
        json["params"]["result"]["transaction"]["meta"]["postTokenBalances"].as_array()
    {
        for postTokenBalance in postTokenBalances.iter() {
            if postTokenBalance["owner"] == target.to_string() {
                mint = postTokenBalance["mint"].as_str().unwrap_or("").to_string();
                mint_post_amount = postTokenBalance["uiTokenAmount"]["uiAmount"]
                    .as_f64()
                    .unwrap();
            } else {
                pool_vault = postTokenBalance["owner"].as_str().unwrap().to_string();
            }
        }
    }
    if let Some(preTokenBalances) =
        json["params"]["result"]["transaction"]["meta"]["preTokenBalances"].as_array()
    {
        for preTokenBalance in preTokenBalances.iter() {
            if preTokenBalance["owner"] == target.to_string() {
                mint_pre_amount = preTokenBalance["uiTokenAmount"]["uiAmount"]
                    .as_f64()
                    .unwrap();
            }
        }
    }

    if mint_pre_amount < mint_post_amount {
        if let Some(inner_instructions) =
            json["params"]["result"]["transaction"]["meta"]["innerInstructions"].as_array()
        {
            for inner_instruction in inner_instructions.iter() {
                // Try to extract the string representation of the log
                if let Some(instructions) = inner_instruction["instructions"].as_array() {
                    for instruction in instructions {
                        if instruction["parsed"]["type"] == "transfer".to_string()
                            && instruction["program"] == "system".to_string()
                            && instruction["parsed"]["info"]["destination"] == pool_vault
                        {
                            amount_in = instruction["parsed"]["info"]["lamports"].as_u64().unwrap();
                        }
                    }
                }
            }
        }
        dirs = "buy".to_string();

        swap_on_jup(mint, dirs, amount_in).await;
    } else {
        dirs = "sell".to_string();
        amount_in = ((mint_pre_amount - mint_post_amount) * 1000000.0) as u64;

        swap_on_jup(mint, dirs, amount_in).await;
    }
}

pub async fn swap_on_jup(mint: String, dir: String, amount: u64) {
    let rpc_client = create_rpc_client().unwrap();
    let mut url = "".to_string();
    let input_mint = "So11111111111111111111111111111111111111112";
    let base_mint = &mint; // Replace with your actual base mint
    let wallet = import_wallet().unwrap();
    // Construct the request URL
    if dir == "buy" {
        url = format!(
            "https://quote-api.jup.ag/v6/quote?inputMint={}&outputMint={}&amount={}&slippageBps=10000",
            input_mint,
            base_mint, // You might need to convert this to base58 representation if it's not already
            amount
        );
    } else {
        url = format!(
            "https://quote-api.jup.ag/v6/quote?inputMint={}&outputMint={}&amount={}&slippageBps=10000",
            base_mint, // You might need to convert this to base58 representation if it's not already
            input_mint,
            amount
        );
    }
    let client = reqwest::Client::new();

    // Send the GET request
    let time_stamp = Instant::now();
    if let Ok(response) = reqwest::get(&url).await {
        if response.status().is_success() {
            // Parse the response JSON if needed
            let json: serde_json::Value = response.json().await.unwrap();
            let swap_request = SwapRequest {
                quoteResponse: json,
                userPublicKey: wallet.pubkey().to_string(),
                wrapAndUnwrapSol: true,
                dynamicComputeUnitLimit: true,
                prioritizationFeeLamports: 52000,
            };
            if let Ok(response) = client
                .post("https://quote-api.jup.ag/v6/swap")
                .header("Content-Type", "application/json")
                .json(&swap_request)
                .send()
                .await
            {
                let res: serde_json::Value = response.json().await.unwrap();

                let tx = base64::decode(
                    res["swapTransaction"]
                        .as_str()
                        .unwrap_or("default")
                        .to_string(),
                )
                .unwrap();
                if let Ok(transaction) = bincode::options()
                    .with_fixint_encoding()
                    .reject_trailing_bytes()
                    .deserialize::<VersionedTransaction>(&tx)
                {
                    let signed_tx =
                        VersionedTransaction::try_new(transaction.message.clone(), &[&wallet])
                            .unwrap();
                    let recent_blockhash = VersionedMessage::recent_blockhash(&transaction.message);
                    // println!("get VersionedTx {:#?}", signed_tx);
                    jito_confirm(
                        &wallet,
                        &rpc_client,
                        signed_tx,
                        &time_stamp,
                        recent_blockhash,
                    )
                    .await;
                };
            };
        } else {
            println!("Failed to fetch: {}", response.status());
        }
    }
}
