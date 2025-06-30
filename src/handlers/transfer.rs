use axum::{extract::Json, http::StatusCode, response::Json as ResponseJson};
use base64::{Engine as _, engine::general_purpose};
use solana_program::system_instruction;
use spl_token::instruction as token_instruction;

use crate::models::{ApiResponse, SendSolRequest, SendTokenRequest, SolTransferData, TokenTransferData, TokenAccountInfo};
use crate::utils::{parse_pubkey};

pub async fn handle_solana_transfer_request(
    Json(transfer_request): Json<SendSolRequest>,
) -> (StatusCode, ResponseJson<ApiResponse<SolTransferData>>) {
    let sender_wallet = match &transfer_request.from {
        Some(wallet_address) if !wallet_address.is_empty() => wallet_address,
        _ => return (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::error("Please provide a valid sender wallet address".to_string()))),
    };
    
    let recipient_wallet = match &transfer_request.to {
        Some(wallet_address) if !wallet_address.is_empty() => wallet_address,
        _ => return (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::error("Please provide a valid recipient wallet address".to_string()))),
    };
    
    let transfer_amount_in_lamports = match transfer_request.lamports {
        Some(0) => return (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::error("Transfer amount needs to be greater than zero".to_string()))),
        Some(amount) if amount > 100_000_000_000_000 => return (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::error("The transfer amount exceeds the maximum allowed limit".to_string()))),
        Some(amount) => amount,
        None => return (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::error("Please specify the amount you want to transfer".to_string()))),
    };
    let sender_public_key = match parse_pubkey(sender_wallet) {
        Ok(parsed_key) => parsed_key,
        Err(validation_error) => return (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::error(validation_error))),
    };
        
    let recipient_public_key = match parse_pubkey(recipient_wallet) {
        Ok(parsed_key) => parsed_key,
        Err(validation_error) => return (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::error(validation_error))),
    };

    if sender_public_key == recipient_public_key {
        return (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::error("You cannot send SOL to your own wallet address".to_string())));
    }

    if sender_public_key == solana_program::system_program::id() || recipient_public_key == solana_program::system_program::id() {
        return (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::error("Transfers involving the system program are not permitted".to_string())));
    }

    let blockchain_instruction = system_instruction::transfer(&sender_public_key, &recipient_public_key, transfer_amount_in_lamports);

    let transfer_response = SolTransferData {
        program_id: blockchain_instruction.program_id.to_string(),
        accounts: blockchain_instruction.accounts.iter().map(|account| account.pubkey.to_string()).collect(),
        instruction_data: general_purpose::STANDARD.encode(&blockchain_instruction.data),
    };

    (StatusCode::OK, ResponseJson(ApiResponse::success(transfer_response)))
}

pub async fn handle_token_transfer_between_users(
    Json(token_request): Json<SendTokenRequest>,
) -> (StatusCode, ResponseJson<ApiResponse<TokenTransferData>>) {
    let receiving_user_address = match &token_request.destination {
        Some(address_string) if !address_string.is_empty() => address_string,
        _ => return (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::error("Destination wallet address is required for this operation".to_string()))),
    };
    
    let token_mint_address = match &token_request.mint {
        Some(mint_string) if !mint_string.is_empty() => mint_string,
        _ => return (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::error("Token mint address must be specified".to_string()))),
    };
    
    let current_token_owner = match &token_request.owner {
        Some(owner_string) if !owner_string.is_empty() => owner_string,
        _ => return (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::error("Current token owner address is needed".to_string()))),
    };
    
    let token_transfer_amount = match token_request.amount {
        Some(0) => return (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::error("Token transfer amount should be more than zero".to_string()))),
        Some(requested_amount) if requested_amount > u64::MAX / 2 => return (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::error("The requested transfer amount is unreasonably large".to_string()))),
        Some(valid_amount) => valid_amount,
        None => return (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::error("Please specify how many tokens to transfer".to_string()))),
    };

    let token_mint_public_key = match parse_pubkey(token_mint_address) {
        Ok(parsed_mint_key) => parsed_mint_key,
        Err(parsing_error) => return (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::error(parsing_error))),
    };
    
    let owner_public_key = match parse_pubkey(current_token_owner) {
        Ok(parsed_owner_key) => parsed_owner_key,
        Err(parsing_error) => return (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::error(parsing_error))),
    };
    
    let destination_public_key = match parse_pubkey(receiving_user_address) {
        Ok(parsed_destination_key) => parsed_destination_key,
        Err(parsing_error) => return (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::error(parsing_error))),
    };

    if owner_public_key == destination_public_key {
        return (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::error("You cannot send tokens to your own wallet address".to_string())));
    }

    if owner_public_key == solana_program::system_program::id() || destination_public_key == solana_program::system_program::id() {
        return (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::error("Token transfers involving the system program are not allowed".to_string())));
    }

    let sender_token_account = spl_associated_token_account::get_associated_token_address(&owner_public_key, &token_mint_public_key);
    let receiver_token_account = spl_associated_token_account::get_associated_token_address(&destination_public_key, &token_mint_public_key);

    let token_transfer_instruction = match token_instruction::transfer(
        &spl_token::id(),
        &sender_token_account,
        &receiver_token_account,
        &owner_public_key,
        &[],
        token_transfer_amount,
    ) {
        Ok(created_instruction) => created_instruction,
        Err(_) => return (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::error("Unable to create the token transfer instruction".to_string()))),
    };

    let account_details = token_transfer_instruction
        .accounts
        .into_iter()
        .map(|account_meta| TokenAccountInfo {
            pubkey: account_meta.pubkey.to_string(),
            is_signer: account_meta.is_signer,
        })
        .collect();

    let token_transfer_response = TokenTransferData {
        program_id: token_transfer_instruction.program_id.to_string(),
        accounts: account_details,
        instruction_data: general_purpose::STANDARD.encode(&token_transfer_instruction.data),
    };

    (StatusCode::OK, ResponseJson(ApiResponse::success(token_transfer_response)))
}