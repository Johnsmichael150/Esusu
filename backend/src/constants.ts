// Stellar network configuration
export const STELLAR_NETWORK = process.env.STELLAR_NETWORK ?? "futurenet";
export const STELLAR_HORIZON_URL =
  process.env.STELLAR_HORIZON_URL ??
  "https://horizon-futurenet.stellar.org";
export const STELLAR_RPC_URL =
  process.env.STELLAR_RPC_URL ?? "https://rpc-futurenet.stellar.org";
export const STELLAR_NETWORK_PASSPHRASE =
  process.env.STELLAR_NETWORK_PASSPHRASE ??
  "Test SDF Future Network ; October 2022";

// USDC asset on Stellar (Futurenet issuer)
export const USDC_ASSET_CODE = "USDC";
export const USDC_ISSUER =
  process.env.USDC_ISSUER ??
  "GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5";

// Circle configuration bounds
export const MIN_MEMBERS = 2;
export const MAX_MEMBERS = 50;
export const MIN_CYCLE_LENGTH_DAYS = 1;
export const MIN_CONTRIBUTION_AMOUNT = 0; // exclusive — must be > 0

// Invite code length (URL-safe base64 characters)
export const INVITE_CODE_LENGTH = 16;
