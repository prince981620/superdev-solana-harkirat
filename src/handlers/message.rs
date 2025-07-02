use axum::{extract::Json, http::StatusCode, response::Json as ResponseJson};
use solana_sdk::signature::{Signature, Signer};

use crate::models::{
    ApiResponse, SignMessageData, SignMessageRequest, VerifyMessageData, VerifyMessageRequest,
};
use crate::utils::{keypair_from_base58, parse_pubkey};

pub async fn sign_message(
    Json(message_request): Json<SignMessageRequest>,
) -> (StatusCode, ResponseJson<ApiResponse<SignMessageData>>) {
    let user_message = match &message_request.message {
        Some(text_content) if !text_content.is_empty() && text_content.len() <= 1000 => text_content,
        Some(text_content) if text_content.is_empty() => return (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::error("Please provide a message to sign".to_string()))),
        Some(_) => return (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::error("Your message is too long - please keep it under 1000 characters".to_string()))),
        None => return (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::error("Please provide a message to sign".to_string()))),
    };
    
    let user_private_key = match &message_request.secret {
        Some(private_key_string) if !private_key_string.is_empty() => private_key_string,
        _ => return (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::error("A valid private key is required for message signing".to_string()))),
    };

    let wallet_keypair = match keypair_from_base58(user_private_key) {
        Ok(valid_keypair) => valid_keypair,
        Err(error_message) => return (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::error(error_message))),
    };

    let message_as_bytes = user_message.as_bytes();
    let digital_signature = wallet_keypair.sign_message(message_as_bytes);

    (StatusCode::OK, ResponseJson(ApiResponse::success(SignMessageData {
        signature: bs58::encode(&digital_signature.as_ref()).into_string(),
        pubkey: wallet_keypair.pubkey().to_string(),
        message: user_message.clone(),
    })))
}

pub async fn verify_message(
    Json(verification_request): Json<VerifyMessageRequest>,
) -> (StatusCode, ResponseJson<ApiResponse<VerifyMessageData>>) {
    let original_message = match &verification_request.message {
        Some(text_content) if !text_content.is_empty() && text_content.len() <= 1000 => text_content,
        Some(text_content) if text_content.is_empty() => return (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::error("Please provide the original message for verification".to_string()))),
        Some(_) => return (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::error("The message is too long to verify - maximum 1000 characters allowed".to_string()))),
        None => return (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::error("Please provide the original message for verification".to_string()))),
    };
    
    let provided_signature = match &verification_request.signature {
        Some(signature_string) if !signature_string.is_empty() => signature_string,
        _ => return (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::error("A digital signature is required for message verification".to_string()))),
    };
    
    let claimed_public_key = match &verification_request.pubkey {
        Some(pubkey_string) if !pubkey_string.is_empty() => pubkey_string,
        _ => return (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::error("The public key of the signer is required for verification".to_string()))),
    };

    let signer_public_key = match parse_pubkey(claimed_public_key) {
        Ok(valid_pubkey) => valid_pubkey,
        Err(parsing_error) => return (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::error(parsing_error))),
    };

    let signature_bytes = match bs58::decode(provided_signature).into_vec() {
        Ok(decoded_bytes) => decoded_bytes,
        Err(_) => return (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::error("The provided signature is not in valid base58 format".to_string()))),
    };

    let digital_signature = match Signature::try_from(signature_bytes.as_slice()) {
        Ok(valid_signature) => valid_signature,
        Err(_) => return (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::error("The signature format is invalid or corrupted".to_string()))),
    };

    let message_as_bytes = original_message.as_bytes();
    let verification_result = digital_signature.verify(&signer_public_key.to_bytes(), message_as_bytes);

    (StatusCode::OK, ResponseJson(ApiResponse::success(VerifyMessageData {
        valid: verification_result,
        message: original_message.clone(),
        pubkey: claimed_public_key.clone(),
    })))
} 