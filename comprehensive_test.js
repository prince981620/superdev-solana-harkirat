const axios = require("axios");
const nacl = require('tweetnacl');
const { PublicKey, Keypair } = require('@solana/web3.js');
const bs58 = require('bs58').default;
const { getAssociatedTokenAddress } = require("@solana/spl-token");

const HTTP_URL = "http://localhost:8084";

async function runTests() {
    let generatedKeypair = null;
    let testsPassed = 0;
    let totalTests = 0;

    function test(name, testFn) {
        totalTests++;
        try {
            testFn();
            console.log(`✅ ${name}`);
            testsPassed++;
        } catch (error) {
            console.log(`❌ ${name}: ${error.message}`);
        }
    }

    try {
        // Test 1: Generate keypair
        console.log("1. Testing keypair generation...");
        const keypairRes = await axios.post(`${HTTP_URL}/keypair`);
        test("Keypair should be successful", () => {
            if (keypairRes.status !== 200) throw new Error(`Expected 200, got ${keypairRes.status}`);
            if (!keypairRes.data.success) throw new Error("Expected success: true");
            if (!keypairRes.data.data.pubkey) throw new Error("Missing pubkey");
            if (!keypairRes.data.data.secret) throw new Error("Missing secret");
        });
        generatedKeypair = keypairRes.data.data;

        // Test 2: Message signing
        console.log("\n2. Testing message signing...");
        const message = "Hello, Solana!";
        const signRes = await axios.post(`${HTTP_URL}/message/sign`, {
            message: message,
            secret: generatedKeypair.secret
        });
        
        test("Message sign should be successful", () => {
            if (signRes.status !== 200) throw new Error(`Expected 200, got ${signRes.status}`);
            if (!signRes.data.success) throw new Error("Expected success: true");
            if (!signRes.data.data.signature) throw new Error("Missing signature");
            if (signRes.data.data.message !== message) throw new Error("Message mismatch");
            if (signRes.data.data.pubkey !== generatedKeypair.pubkey) throw new Error("Pubkey mismatch");
        });

        test("Signature should be base58 encoded", () => {
            const signatureBytes = bs58.decode(signRes.data.data.signature);
            if (signatureBytes.length !== 64) throw new Error(`Expected 64 bytes, got ${signatureBytes.length}`);
        });

        // Test 3: SOL transfer
        console.log("\n3. Testing SOL transfer...");
        const senderKeypair = Keypair.generate();
        const recipientKeypair = Keypair.generate();
        const lamports = 1000000;

        const solRes = await axios.post(`${HTTP_URL}/send/sol`, {
            from: senderKeypair.publicKey.toString(),
            to: recipientKeypair.publicKey.toString(),
            lamports: lamports,
        });

        test("SOL transfer should be successful", () => {
            if (solRes.status !== 200) throw new Error(`Expected 200, got ${solRes.status}`);
            if (!solRes.data.success) throw new Error("Expected success: true");
            if (solRes.data.data.program_id !== "11111111111111111111111111111111") throw new Error("Wrong program_id");
            if (solRes.data.data.accounts.length !== 2) throw new Error("Expected 2 accounts");
            if (solRes.data.data.accounts[0] !== senderKeypair.publicKey.toString()) throw new Error("Wrong sender account");
            if (solRes.data.data.accounts[1] !== recipientKeypair.publicKey.toString()) throw new Error("Wrong recipient account");
        });

        test("SOL transfer instruction data should be base58", () => {
            const instructionData = bs58.decode(solRes.data.data.instruction_data);
            if (instructionData.length === 0) throw new Error("Empty instruction data");
        });

        // Test 4: Zero lamports should fail
        try {
            await axios.post(`${HTTP_URL}/send/sol`, {
                from: senderKeypair.publicKey.toString(),
                to: recipientKeypair.publicKey.toString(),
                lamports: 0
            });
        } catch (error) {
            test("Zero lamports should be rejected", () => {
                if (error.response.status !== 400) throw new Error(`Expected 400, got ${error.response.status}`);
                if (error.response.data.error !== "Amount must be greater than 0") throw new Error(`Wrong error message: ${error.response.data.error}`);
            });
        }

        // Test 5: Invalid sender should fail
        try {
            await axios.post(`${HTTP_URL}/send/sol`, {
                from: "invalid",
                to: recipientKeypair.publicKey.toString(),
                lamports: 1000000
            });
        } catch (error) {
            test("Invalid sender should be rejected", () => {
                if (error.response.status !== 400) throw new Error(`Expected 400, got ${error.response.status}`);
                if (error.response.data.error !== "Invalid sender public key") throw new Error(`Wrong error message: ${error.response.data.error}`);
            });
        }

        // Test 6: Token transfer
        console.log("\n4. Testing token transfer...");
        const destinationKeypair = Keypair.generate();
        const mintKeypair = Keypair.generate();
        const ownerKeypair = Keypair.generate();
        const amount = 1000000;

        const tokenRes = await axios.post(`${HTTP_URL}/send/token`, {
            destination: bs58.encode(destinationKeypair.publicKey.toBytes()),
            mint: bs58.encode(mintKeypair.publicKey.toBytes()),
            owner: bs58.encode(ownerKeypair.publicKey.toBytes()),
            amount: amount,
        });

        let ata = await getAssociatedTokenAddress(mintKeypair.publicKey, destinationKeypair.publicKey);

        test("Token transfer should be successful", () => {
            if (tokenRes.status !== 200) throw new Error(`Expected 200, got ${tokenRes.status}`);
            if (!tokenRes.data.success) throw new Error("Expected success: true");
            if (tokenRes.data.data.program_id !== "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") throw new Error("Wrong program_id");
            if (tokenRes.data.data.accounts.length !== 3) throw new Error("Expected 3 accounts");
        });

        test("Token transfer account structure should match test expectations", () => {
            const accounts = tokenRes.data.data.accounts;
            if (accounts[0].pubkey !== ownerKeypair.publicKey.toString()) throw new Error("accounts[0] should be owner");
            if (accounts[1].pubkey !== ata.toString()) throw new Error("accounts[1] should be destination ATA");
            if (accounts[2].pubkey !== ownerKeypair.publicKey.toString()) throw new Error("accounts[2] should be owner");
            
            if (accounts[0].isSigner !== false) throw new Error("accounts[0].isSigner should be false");
            if (accounts[1].isSigner !== false) throw new Error("accounts[1].isSigner should be false");
            if (accounts[2].isSigner !== true) throw new Error("accounts[2].isSigner should be true");
        });

        // Test 7: Token creation
        console.log("\n5. Testing token creation...");
        let mintKeypairForCreate = Keypair.generate();
        const tokenCreateRes = await axios.post(`${HTTP_URL}/token/create`, {
            mintAuthority: generatedKeypair.pubkey,
            mint: mintKeypairForCreate.publicKey.toString(),
            decimals: 6
        });

        test("Token creation should be successful", () => {
            if (tokenCreateRes.status !== 200) throw new Error(`Expected 200, got ${tokenCreateRes.status}`);
            if (!tokenCreateRes.data.success) throw new Error("Expected success: true");
            if (tokenCreateRes.data.data.program_id !== "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") throw new Error("Wrong program_id");
            if (tokenCreateRes.data.data.accounts.length !== 2) throw new Error("Expected 2 accounts");
            
            const accounts = tokenCreateRes.data.data.accounts;
            if (accounts[0].is_signer !== false) throw new Error("accounts[0].is_signer should be false");
            if (accounts[0].is_writable !== true) throw new Error("accounts[0].is_writable should be true");
            if (accounts[1].is_signer !== false) throw new Error("accounts[1].is_signer should be false");
            if (accounts[1].is_writable !== false) throw new Error("accounts[1].is_writable should be false");
        });

        console.log(`\n📊 Tests Summary: ${testsPassed}/${totalTests} passed`);
        if (testsPassed === totalTests) {
            console.log("🎉 All tests passed!");
        } else {
            console.log(`❌ ${totalTests - testsPassed} tests failed`);
        }

    } catch (error) {
        console.error("Test suite error:", error.response?.data || error.message);
    }
}

runTests();
