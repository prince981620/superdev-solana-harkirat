#!/bin/bash

# Start the server in background
cargo run &
SERVER_PID=$!

# Wait for server to start
sleep 3

echo "Testing endpoints..."

# Test 1: Generate Keypair
echo "1. Testing /keypair endpoint:"
curl -X POST http://localhost:8084/keypair -H "Content-Type: application/json" | jq '.'

# Test 2: Sign Message
echo -e "\n2. Testing /message/sign endpoint:"
curl -X POST http://localhost:8084/message/sign \
  -H "Content-Type: application/json" \
  -d '{"message": "Hello, Solana!", "secret": "5J7XqTxBdKKkvkUWn4rEBv2Qwm7JYCKzXrVH4nKvWP8A5aB3j6nV7xS9Z1KMC2D4G5E8F9Q4R7T1Y2U3I6O8P"}' | jq '.'

# Test 3: Create Token
echo -e "\n3. Testing /token/create endpoint:"
curl -X POST http://localhost:8084/token/create \
  -H "Content-Type: application/json" \
  -d '{"mintAuthority": "11111111111111111111111111111112", "mint": "11111111111111111111111111111113", "decimals": 6}' | jq '.'

# Test 4: Send SOL
echo -e "\n4. Testing /send/sol endpoint:"
curl -X POST http://localhost:8084/send/sol \
  -H "Content-Type: application/json" \
  -d '{"from": "11111111111111111111111111111112", "to": "11111111111111111111111111111113", "lamports": 100000}' | jq '.'

# Kill the server
kill $SERVER_PID

echo -e "\nTesting completed!"
