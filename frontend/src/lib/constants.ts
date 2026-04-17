// Stellar network configuration
export const STELLAR_NETWORK =
  process.env.NEXT_PUBLIC_STELLAR_NETWORK ?? "futurenet";
export const STELLAR_HORIZON_URL =
  process.env.NEXT_PUBLIC_STELLAR_HORIZON_URL ??
  "https://horizon-futurenet.stellar.org";
export const STELLAR_RPC_URL =
  process.env.NEXT_PUBLIC_STELLAR_RPC_URL ??
  "https://rpc-futurenet.stellar.org";
export const STELLAR_NETWORK_PASSPHRASE =
  process.env.NEXT_PUBLIC_STELLAR_NETWORK_PASSPHRASE ??
  "Test SDF Future Network ; October 2022";

// USDC asset on Stellar
export const USDC_ASSET_CODE = "USDC";
export const USDC_ISSUER =
  process.env.NEXT_PUBLIC_USDC_ISSUER ??
  "GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5";

// API base URL
export const API_URL =
  process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:3001";

// Circle configuration bounds
export const MIN_MEMBERS = 2;
export const MAX_MEMBERS = 50;
export const MIN_CYCLE_LENGTH_DAYS = 1;
