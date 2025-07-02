use axum::{extract::Json, http::StatusCode, response::Json as ResponseJson};
use spl_token::instruction as token_instruction;

use crate::models::{ApiResponse, CreateTokenRequest, InstructionData, MintTokenRequest};
use crate::utils::{instruction_to_response, parse_pubkey};

pub async fn create_token(
    Json(token_creation_request): Json<CreateTokenRequest>,
) -> (StatusCode, ResponseJson<ApiResponse<InstructionData>>) {
    let authority_for_new_mint = match &token_creation_request.mint_authority {
        Some(authority_address) if !authority_address.is_empty() => authority_address,
        _ => return (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::error("A mint authority address is required to create a new token".to_string()))),
    };
    
    let new_token_mint_address = match &token_creation_request.mint {
        Some(mint_address) if !mint_address.is_empty() => mint_address,
        _ => return (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::error("Please provide the address for the new token mint".to_string()))),
    };
    
    let token_decimal_places = match token_creation_request.decimals {
        Some(decimal_count) if decimal_count <= 9 => decimal_count,
        Some(_) => return (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::error("Decimals must be between 0 and 9".to_string()))),
        None => return (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::error("Please specify the number of decimal places for this token".to_string()))),
    };

    let mint_authority_public_key = match parse_pubkey(authority_for_new_mint) {
        Ok(valid_pubkey) => valid_pubkey,
        Err(parsing_error) => return (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::error(parsing_error))),
    };
    
    let token_mint_public_key = match parse_pubkey(new_token_mint_address) {
        Ok(valid_pubkey) => valid_pubkey,
        Err(parsing_error) => return (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::error(parsing_error))),
    };

    if mint_authority_public_key == solana_program::system_program::id() {
        return (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::error("The system program cannot be used as a mint authority".to_string())));
    }
    
    if token_mint_public_key == solana_program::system_program::id() {
        return (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::error("The system program cannot be used as a token mint address".to_string())));
    }

    let mint_initialization_instruction = match token_instruction::initialize_mint(
        &spl_token::id(),
        &token_mint_public_key,
        &mint_authority_public_key,
        Some(&mint_authority_public_key),
        token_decimal_places,
    ) {
        Ok(created_instruction) => created_instruction,
        Err(_) => return (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::error("Unable to create the token initialization instruction".to_string()))),
    };

    (StatusCode::OK, ResponseJson(ApiResponse::success(instruction_to_response(mint_initialization_instruction))))
}

pub async fn mint_token(
    Json(token_minting_request): Json<MintTokenRequest>,
) -> (StatusCode, ResponseJson<ApiResponse<InstructionData>>) {
    let target_token_mint = match &token_minting_request.mint {
        Some(mint_address) if !mint_address.is_empty() => mint_address,
        _ => return (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::error("Please provide the mint address of the token you want to mint".to_string()))),
    };
    
    let token_recipient_address = match &token_minting_request.destination {
        Some(recipient_address) if !recipient_address.is_empty() => recipient_address,
        _ => return (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::error("A destination address is required to receive the minted tokens".to_string()))),
    };
    
    let minting_authority_address = match &token_minting_request.authority {
        Some(authority_address) if !authority_address.is_empty() => authority_address,
        _ => return (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::error("The minting authority address is required to authorize this operation".to_string()))),
    };
    
    let tokens_to_mint = match token_minting_request.amount {
        Some(0) => return (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::error("Amount must be greater than 0".to_string()))),
        Some(mint_amount) if mint_amount > 0 => mint_amount,
        _ => return (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::error("Please specify how many tokens you want to mint".to_string()))),
    };

    let token_mint_public_key = match parse_pubkey(target_token_mint) {
        Ok(valid_pubkey) => valid_pubkey,
        Err(parsing_error) => return (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::error(parsing_error))),
    };
    
    let recipient_public_key = match parse_pubkey(token_recipient_address) {
        Ok(valid_pubkey) => valid_pubkey,
        Err(parsing_error) => return (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::error(parsing_error))),
    };
    
    let minting_authority_public_key = match parse_pubkey(minting_authority_address) {
        Ok(valid_pubkey) => valid_pubkey,
        Err(parsing_error) => return (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::error(parsing_error))),
    };

    if token_mint_public_key == solana_program::system_program::id() {
        return (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::error("The system program cannot be used as a token mint".to_string())));
    }
    
    if recipient_public_key == solana_program::system_program::id() {
        return (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::error("Tokens cannot be minted directly to the system program".to_string())));
    }
    
    if minting_authority_public_key == solana_program::system_program::id() {
        return (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::error("The system program cannot serve as a minting authority".to_string())));
    }

    let token_minting_instruction = match token_instruction::mint_to(
        &spl_token::id(),
        &token_mint_public_key,
        &recipient_public_key,
        &minting_authority_public_key,
        &[],
        tokens_to_mint,
    ) {
        Ok(created_instruction) => created_instruction,
        Err(_) => return (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::error("Unable to create the token minting instruction".to_string()))),
    };

    (StatusCode::OK, ResponseJson(ApiResponse::success(instruction_to_response(token_minting_instruction))))
}