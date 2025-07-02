const axios = require("axios");
const { PublicKey, Keypair } = require('@solana/web3.js');
const bs58 = require('bs58').default;
const { getAssociatedTokenAddress } = require("@solana/spl-token");

const HTTP_URL = "http://localhost:8084";

async function testEndpoints() {
    try {
        // Exact test from JavaScript test suite
        console.log("Running exact test from JavaScript test suite...");
        const destinationKeypair = Keypair.generate();
        const mintKeypair = Keypair.generate();
        const ownerKeypair = Keypair.generate();
        const amount = 1000000;

        const res = await axios.post(`${HTTP_URL}/send/token`, {
            destination: bs58.encode(destinationKeypair.publicKey.toBytes()),
            mint: bs58.encode(mintKeypair.publicKey.toBytes()),
            owner: bs58.encode(ownerKeypair.publicKey.toBytes()),
            amount: amount,
        });

        let destinationAta = await getAssociatedTokenAddress(mintKeypair.publicKey, destinationKeypair.publicKey);
        let ownerAta = await getAssociatedTokenAddress(mintKeypair.publicKey, ownerKeypair.publicKey);
        
        console.log("Response:", JSON.stringify(res.data, null, 2));
        console.log("\nExpected values from test:");
        console.log("accounts[0].pubkey should be:", ownerKeypair.publicKey.toString());
        console.log("accounts[1].pubkey should be:", destinationAta.toString());
        console.log("accounts[2].pubkey should be:", ownerKeypair.publicKey.toString());
        
        console.log("\nCalculated ATAs:");
        console.log("Owner ATA:", ownerAta.toString());
        console.log("Destination ATA:", destinationAta.toString());
        
        console.log("\nActual values:");
        if (res.data.success) {
            const accounts = res.data.data.accounts;
            accounts.forEach((acc, i) => {
                console.log(`accounts[${i}].pubkey is:`, acc.pubkey);
                console.log(`accounts[${i}].isSigner is:`, acc.isSigner);
            });
            
            // Test assertions
            console.log("\nTest results:");
            console.log("accounts[0].pubkey === ownerKeypair:", accounts[0].pubkey === ownerKeypair.publicKey.toString());
            console.log("accounts[0].pubkey === ownerAta:", accounts[0].pubkey === ownerAta.toString());
            console.log("accounts[1].pubkey === destinationAta:", accounts[1].pubkey === destinationAta.toString());
            console.log("accounts[2].pubkey === ownerKeypair:", accounts[2].pubkey === ownerKeypair.publicKey.toString());
        }

    } catch (error) {
        console.error("Error:", error.response?.data || error.message);
    }
}

testEndpoints();
