use base64::{Engine as _, engine::general_purpose};
use solana_program::{instruction::Instruction, pubkey::Pubkey};
use solana_sdk::signature::Keypair;
use std::str::FromStr;

use crate::models::{AccountInfo, InstructionData};

pub fn parse_pubkey(address_string: &str) -> Result<Pubkey, String> {
    // First, let's do some basic checks on the address format
    if address_string.len() < 32 || address_string.len() > 44 {
        return Err("The provided address length is not valid for a Solana public key".to_string());
    }
    
    // Ensure all characters are valid base58 characters (no confusing ones like 0, O, I, l)
    if !address_string.chars().all(|character| character.is_ascii_alphanumeric() && !"0OIl".contains(character)) {
        return Err("The address contains invalid characters for base58 encoding".to_string());
    }
    
    Pubkey::from_str(address_string).map_err(|_| format!("Unable to parse the provided address as a valid Solana public key: {}", address_string))
}

pub fn keypair_from_base58(private_key_string: &str) -> Result<Keypair, String> {
    // Let's validate the secret key format before trying to decode it
    if private_key_string.len() < 80 || private_key_string.len() > 100 {
        return Err("The private key length doesn't match expected base58 encoding standards".to_string());
    }
    
    // Make sure all characters are proper base58 (excluding confusing characters)
    if !private_key_string.chars().all(|character| character.is_ascii_alphanumeric() && !"0OIl".contains(character)) {
        return Err("The private key contains characters that aren't valid in base58 encoding".to_string());
    }
    
    let decoded_key_bytes = bs58::decode(private_key_string)
        .into_vec()
        .map_err(|_| "Failed to decode the private key from base58 format")?;

    if decoded_key_bytes.len() != 64 {
        return Err("The decoded private key must be exactly 64 bytes for Solana keypairs".to_string());
    }

    Keypair::from_bytes(&decoded_key_bytes).map_err(|_| "The provided bytes don't form a valid Solana keypair".to_string())
}

pub fn instruction_to_response(blockchain_instruction: Instruction) -> InstructionData {
    let account_information = blockchain_instruction
        .accounts
        .into_iter()
        .map(|account_metadata| AccountInfo {
            pubkey: account_metadata.pubkey.to_string(),
            is_signer: account_metadata.is_signer,
            is_writable: account_metadata.is_writable,
        })
        .collect();

    InstructionData {
        program_id: blockchain_instruction.program_id.to_string(),
        accounts: account_information,
        instruction_data: general_purpose::STANDARD.encode(&blockchain_instruction.data),
    }
} 