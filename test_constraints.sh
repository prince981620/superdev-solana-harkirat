#!/bin/bash

# Comprehensive test for all validation constraints

echo "🧪 Testing Enhanced Validation Constraints"
echo "=========================================="

# Start the server in background
echo "Starting server..."
cargo run --bin superdev-rust-assignment &
SERVER_PID=$!

# Wait for server to start
sleep 3

BASE_URL="http://localhost:8084"

echo -e "\n1. Testing Token Creation Constraints:"
echo "----------------------------------------"

# Valid decimals
curl -s -X POST "$BASE_URL/token/create" \
  -H "Content-Type: application/json" \
  -d '{"mintAuthority": "11111111111111111111111111111112", "mint": "11111111111111111111111111111113", "decimals": 6}' \
  | jq '.success, .data.program_id // .error'

# Invalid decimals (too high)
echo "Testing invalid decimals (> 9):"
curl -s -X POST "$BASE_URL/token/create" \
  -H "Content-Type: application/json" \
  -d '{"mintAuthority": "11111111111111111111111111111112", "mint": "11111111111111111111111111111113", "decimals": 15}' \
  | jq '.success, .error'

# System program as mint authority
echo "Testing system program as mint authority:"
curl -s -X POST "$BASE_URL/token/create" \
  -H "Content-Type: application/json" \
  -d '{"mintAuthority": "11111111111111111111111111111111", "mint": "11111111111111111111111111111113", "decimals": 6}' \
  | jq '.success, .error'

echo -e "\n2. Testing Message Signing Constraints:"
echo "----------------------------------------"

# Test long message (over 1000 chars)
LONG_MESSAGE=$(printf 'a%.0s' {1..1001})
echo "Testing message length > 1000 chars:"
curl -s -X POST "$BASE_URL/message/sign" \
  -H "Content-Type: application/json" \
  -d "{\"message\": \"$LONG_MESSAGE\", \"secret\": \"5J7XqTxBdKKkvkUWn4rEBv2Qwm7JYCKzXrVH4nKvWP8A5aB3j6nV7xS9Z1KMC2D4G5E8F9Q4R7T1Y2U3I6O8P\"}" \
  | jq '.success, .error'

echo -e "\n3. Testing SOL Transfer Constraints:"
echo "-----------------------------------"

# Valid transfer
echo "Valid SOL transfer:"
curl -s -X POST "$BASE_URL/send/sol" \
  -H "Content-Type: application/json" \
  -d '{"from": "11111111111111111111111111111112", "to": "11111111111111111111111111111113", "lamports": 100000}' \
  | jq '.success, .data.program_id // .error'

# Self-transfer
echo "Testing self-transfer prevention:"
curl -s -X POST "$BASE_URL/send/sol" \
  -H "Content-Type: application/json" \
  -d '{"from": "11111111111111111111111111111112", "to": "11111111111111111111111111111112", "lamports": 100000}' \
  | jq '.success, .error'

# Amount too large
echo "Testing amount too large:"
curl -s -X POST "$BASE_URL/send/sol" \
  -H "Content-Type: application/json" \
  -d '{"from": "11111111111111111111111111111112", "to": "11111111111111111111111111111113", "lamports": 999999999999999}' \
  | jq '.success, .error'

# System program transfer
echo "Testing system program transfer:"
curl -s -X POST "$BASE_URL/send/sol" \
  -H "Content-Type: application/json" \
  -d '{"from": "11111111111111111111111111111111", "to": "11111111111111111111111111111113", "lamports": 100000}' \
  | jq '.success, .error'

echo -e "\n4. Testing Token Transfer Constraints:"
echo "-------------------------------------"

# Valid token transfer
echo "Valid token transfer:"
curl -s -X POST "$BASE_URL/send/token" \
  -H "Content-Type: application/json" \
  -d '{"destination": "11111111111111111111111111111113", "mint": "11111111111111111111111111111114", "owner": "11111111111111111111111111111112", "amount": 100000}' \
  | jq '.success, .data.program_id // .error'

# Self-transfer prevention
echo "Testing token self-transfer prevention:"
curl -s -X POST "$BASE_URL/send/token" \
  -H "Content-Type: application/json" \
  -d '{"destination": "11111111111111111111111111111112", "mint": "11111111111111111111111111111114", "owner": "11111111111111111111111111111112", "amount": 100000}' \
  | jq '.success, .error'

# Amount too large
echo "Testing token amount too large:"
curl -s -X POST "$BASE_URL/send/token" \
  -H "Content-Type: application/json" \
  -d '{"destination": "11111111111111111111111111111113", "mint": "11111111111111111111111111111114", "owner": "11111111111111111111111111111112", "amount": 18446744073709551615}' \
  | jq '.success, .error'

# System program as owner
echo "Testing system program as owner:"
curl -s -X POST "$BASE_URL/send/token" \
  -H "Content-Type: application/json" \
  -d '{"destination": "11111111111111111111111111111113", "mint": "11111111111111111111111111111114", "owner": "11111111111111111111111111111111", "amount": 100000}' \
  | jq '.success, .error'

echo -e "\n5. Testing Public Key Validation:"
echo "---------------------------------"

# Invalid public key format
echo "Testing invalid public key:"
curl -s -X POST "$BASE_URL/send/sol" \
  -H "Content-Type: application/json" \
  -d '{"from": "invalid_key", "to": "11111111111111111111111111111113", "lamports": 100000}' \
  | jq '.success, .error'

# Public key too short
echo "Testing public key too short:"
curl -s -X POST "$BASE_URL/send/sol" \
  -H "Content-Type: application/json" \
  -d '{"from": "123", "to": "11111111111111111111111111111113", "lamports": 100000}' \
  | jq '.success, .error'

echo -e "\n6. Testing Mint Token Constraints:"
echo "----------------------------------"

# Valid mint
echo "Valid mint:"
curl -s -X POST "$BASE_URL/token/mint" \
  -H "Content-Type: application/json" \
  -d '{"mint": "11111111111111111111111111111114", "destination": "11111111111111111111111111111113", "authority": "11111111111111111111111111111112", "amount": 1000000}' \
  | jq '.success, .data.program_id // .error'

# Zero amount
echo "Testing zero amount:"
curl -s -X POST "$BASE_URL/token/mint" \
  -H "Content-Type: application/json" \
  -d '{"mint": "11111111111111111111111111111114", "destination": "11111111111111111111111111111113", "authority": "11111111111111111111111111111112", "amount": 0}' \
  | jq '.success, .error'

# System program as destination
echo "Testing system program as destination:"
curl -s -X POST "$BASE_URL/token/mint" \
  -H "Content-Type: application/json" \
  -d '{"mint": "11111111111111111111111111111114", "destination": "11111111111111111111111111111111", "authority": "11111111111111111111111111111112", "amount": 1000000}' \
  | jq '.success, .error'

# Kill the server
kill $SERVER_PID

echo -e "\n✅ Testing completed!"
echo "All validation constraints have been tested."
