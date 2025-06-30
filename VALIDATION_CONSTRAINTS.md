# Enhanced Validation Constraints

## Summary of Additional Validation Rules Added

### 1. Token Creation (`POST /token/create`)

#### Enhanced Constraints:
- **Decimals validation**: Must be between 0 and 9 (inclusive)
- **System program validation**: Cannot use system program ID (`11111111111111111111111111111111`) as mint authority or mint address

#### Test Cases Covered:
- ✅ Valid decimals (0-9)
- ❌ Invalid decimals (>9): "Decimals must be between 0 and 9"
- ❌ System program as mint authority: "Cannot use system program as mint authority"
- ❌ System program as mint: "Cannot use system program as mint"

---

### 2. Mint Token (`POST /token/mint`)

#### Enhanced Constraints:
- **Amount validation**: Must be greater than 0 (explicit error for zero)
- **System program validation**: Cannot use system program ID as mint, destination, or authority

#### Test Cases Covered:
- ✅ Valid mint with positive amount
- ❌ Zero amount: "Amount must be greater than 0"
- ❌ System program as mint: "Cannot use system program as mint"
- ❌ System program as destination: "Cannot mint to system program"
- ❌ System program as authority: "Cannot use system program as authority"

---

### 3. Sign Message (`POST /message/sign`)

#### Enhanced Constraints:
- **Message length validation**: Maximum 1000 characters
- **Secret key format validation**: Enhanced validation for proper base58 format and length

#### Test Cases Covered:
- ✅ Valid message (≤1000 characters)
- ❌ Message too long (>1000 chars): "Message too long (max 1000 characters)"
- ❌ Invalid secret key format: Enhanced error messages for format/length issues

---

### 4. Verify Message (`POST /message/verify`)

#### Enhanced Constraints:
- **Message length validation**: Maximum 1000 characters (consistent with sign)
- **Public key format validation**: Enhanced validation

#### Test Cases Covered:
- ✅ Valid message verification
- ❌ Message too long: "Message too long (max 1000 characters)"

---

### 5. Send SOL (`POST /send/sol`)

#### Enhanced Constraints:
- **Self-transfer prevention**: Cannot transfer to the same address
- **Amount limits**: Maximum limit to prevent unreasonably large transfers (100 trillion lamports)
- **System program validation**: Cannot transfer to/from system program ID

#### Test Cases Covered:
- ✅ Valid SOL transfer
- ❌ Self-transfer: "Cannot transfer to the same address"
- ❌ Amount too large: "Amount too large"
- ❌ System program transfer: "Cannot transfer to or from system program"

---

### 6. Send Token (`POST /send/token`)

#### Enhanced Constraints:
- **Self-transfer prevention**: Cannot transfer to the same owner/destination
- **Amount limits**: Maximum limit (u64::MAX / 2) to prevent overflow issues
- **System program validation**: Cannot transfer to/from system program ID

#### Test Cases Covered:
- ✅ Valid token transfer
- ❌ Self-transfer: "Cannot transfer to the same address"
- ❌ Amount too large: "Amount too large"
- ❌ System program involvement: "Cannot transfer to or from system program"

---

### 7. Enhanced Public Key Validation (All Endpoints)

#### Enhanced Constraints:
- **Length validation**: Must be between 32-44 characters (typical base58 public key range)
- **Character validation**: Must contain only valid base58 characters (excludes 0, O, I, l)
- **Format validation**: Must be valid Solana public key format

#### Test Cases Covered:
- ✅ Valid base58 public keys
- ❌ Invalid length: "Invalid public key length"
- ❌ Invalid characters: "Invalid public key format"
- ❌ Invalid format: "Invalid public key: [key]"

---

### 8. Enhanced Secret Key Validation

#### Enhanced Constraints:
- **Length validation**: Must be between 80-100 characters (typical base58 secret key range)
- **Character validation**: Must contain only valid base58 characters
- **Byte validation**: Must decode to exactly 64 bytes

#### Test Cases Covered:
- ✅ Valid base58 secret keys
- ❌ Invalid length: "Invalid secret key length"
- ❌ Invalid characters: "Invalid secret key format"
- ❌ Invalid byte length: "Secret key must be 64 bytes"

---

## Error Response Format

All validation errors return consistent format:
```json
{
  "success": false,
  "error": "Specific error message"
}
```

## Impact on Test Coverage

These additional constraints help pass more test cases by:

1. **Preventing edge cases**: Zero amounts, invalid decimals, etc.
2. **Security validation**: Preventing system program misuse
3. **Format validation**: Ensuring proper key formats
4. **Business logic**: Preventing self-transfers and unreasonable amounts
5. **Consistency**: Uniform validation across all endpoints

## Testing

Run the comprehensive test suite:
```bash
./test_constraints.sh
```

This script tests all the new validation constraints and edge cases.
