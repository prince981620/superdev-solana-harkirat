use axum::response::Json as ResponseJson;
use solana_sdk::signature::{Keypair, Signer};

use crate::models::{ApiResponse, KeypairData};

pub async fn generate_keypair() -> ResponseJson<ApiResponse<KeypairData>> {
    // Create a brand new cryptographic keypair for Solana blockchain
    let new_wallet_keypair = Keypair::new();
    let wallet_public_address = new_wallet_keypair.pubkey().to_string();
    let wallet_private_key = bs58::encode(&new_wallet_keypair.to_bytes()).into_string();

    ResponseJson(ApiResponse::success(KeypairData { 
        pubkey: wallet_public_address, 
        secret: wallet_private_key 
    }))
} 