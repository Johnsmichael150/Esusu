"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.INVITE_CODE_LENGTH = exports.MIN_CONTRIBUTION_AMOUNT = exports.MIN_CYCLE_LENGTH_DAYS = exports.MAX_MEMBERS = exports.MIN_MEMBERS = exports.USDC_ISSUER = exports.USDC_ASSET_CODE = exports.STELLAR_NETWORK_PASSPHRASE = exports.STELLAR_RPC_URL = exports.STELLAR_HORIZON_URL = exports.STELLAR_NETWORK = void 0;
// Stellar network configuration
exports.STELLAR_NETWORK = process.env.STELLAR_NETWORK ?? "futurenet";
exports.STELLAR_HORIZON_URL = process.env.STELLAR_HORIZON_URL ??
    "https://horizon-futurenet.stellar.org";
exports.STELLAR_RPC_URL = process.env.STELLAR_RPC_URL ?? "https://rpc-futurenet.stellar.org";
exports.STELLAR_NETWORK_PASSPHRASE = process.env.STELLAR_NETWORK_PASSPHRASE ??
    "Test SDF Future Network ; October 2022";
// USDC asset on Stellar (Futurenet issuer)
exports.USDC_ASSET_CODE = "USDC";
exports.USDC_ISSUER = process.env.USDC_ISSUER ??
    "GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5";
// Circle configuration bounds
exports.MIN_MEMBERS = 2;
exports.MAX_MEMBERS = 50;
exports.MIN_CYCLE_LENGTH_DAYS = 1;
exports.MIN_CONTRIBUTION_AMOUNT = 0; // exclusive — must be > 0
// Invite code length (URL-safe base64 characters)
exports.INVITE_CODE_LENGTH = 16;
//# sourceMappingURL=constants.js.map