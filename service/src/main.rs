/*
We implement this backend service, instead of using the TypeScript SDK directly in the frontend, because:

- Wallet generation requires compilation of Sway code to obtain the predicate byte code, which in turn gives us the multi-sig wallet address (as a hash of the byte code). This would be hard to do in the front-end.
- The TypeScript SDK has no support for predicates, which would make it impossible to spend funds from the multi-sig wallet. There is rudimentary support for predicates in the Rust SDK which we can buid on.

API spec:
(possible extension is to implement a OpenAPI spec for it)
TBD
 */

use poem::{
    error::NotFoundError, handler, listener::TcpListener, post, web::Json, Result, Route,
    Server,
};
use serde::Serialize;
use serde::Deserialize;

use fuel_gql_client::fuel_tx::Address;
use fuel_gql_client::fuel_tx::AssetId;
use fuel_gql_client::fuel_tx::TxId;
use fuel_gql_client::fuel_tx::UtxoId;
use fuels_signers::WalletUnlocked;
use fuels_signers::Payload;
use fuels_signers::provider::Provider;
use fuels_signers::fuel_crypto::SecretKey;
use fuels_core::parameters::TxParameters;
use fuels_types::bech32::Bech32Address;

#[derive(Debug, Deserialize)]
struct PublicKeys {
    pk1: String,
    pk2: String,
    pk3: String,
}

const NODE_URL: &str = "node-beta-1.fuel.network";

/*
Generation of multi-sig wallet.
Receive three public keys to generate one multi-sig wallet address.
*/
#[handler]
fn generate_wallet() -> String {
    "gen wallet TODO!".to_string()
}

/*
Spending of funds from multi-sig wallet.

1. Specificy wallet address to list wallet assets. To do that, retrieve all UTXOs for the wallet, and add up the amounts of coins for each asset ID.
2. Specify asset ID, amount of coins and destination address for transfer. Collect sufficient number of UTXOs from the retrieved list to spend given amount for given asset ID (if possible, otherwise error).
3. For each selected UTXO, concatenate and hash the transaction ID and the output index to obtain the message hash to be signed.
4. For each message hash, sign the message hash with at least two of the three private keys associated with the given wallet address.
5. Send the request for the transaction creation to the backend service, which will properly assemble the transaction and submit it to the network.
6. Store the transaction ID for the generated transaction.

For signature verification:
- msg_hash = sha256(txid + output_index)
- signature = sign(private_key, msg_hash)
*/

#[derive(Deserialize)]
struct SpendFundsRequest {
    wallet: Address,
    asset_id: AssetId,
    amount: u64,
    recipient: Address,
    inputs: Vec<InputWithSig>,
}

#[derive(Deserialize)]
struct InputWithSig {
    utxo_id: UtxoId,
    signatures: Vec<Vec<u8>>,
}

#[derive(Serialize)]
struct SpendFundsResponse {
    wallet: Address,
    asset_id: AssetId,
    amount: u64,
    recipient: Address,
    inputs: Vec<InputNoSig>,
    utxo_id: UtxoId,
}

#[derive(Serialize)]
struct InputNoSig {
    utxo_id: UtxoId,
}


#[handler]
async fn spend_funds(req: Json<SpendFundsRequest>) -> Json<SpendFundsResponse> {

    // initialize the provider and the wallet with given private key
    let provider = Provider::connect(NODE_URL).await.unwrap();
    let secret = unsafe { SecretKey::from_bytes_unchecked([0; 32]) };
    let unlocked = WalletUnlocked::new_from_private_key(secret, Some(provider));

    // convert address strings to Bech32 addresses (which can't be deserialized directly)
    let wallet = Bech32Address::from(req.wallet);
    let recipient = Bech32Address::from(req.recipient);

    // TODO: retrieve code for this wallet address, probably from a mapping that has wallet address as key
    let code = Vec::<u8>::new();

    let payloads = Vec::<Payload>::new();

    let result = unlocked.multi_spend_predicate(
        &wallet,
        code,
        req.asset_id,
        req.amount,
        &recipient,
        payloads,
        TxParameters::default(),
    );

    let inputs = req.inputs
        .iter()
        .map(|input| InputNoSig{utxo_id: input.utxo_id})
        .collect();

    Json(SpendFundsResponse {
        wallet: req.wallet,
        asset_id: req.asset_id,
        amount: req.amount,
        recipient: req.recipient,
        inputs: inputs,
        utxo_id: UtxoId::new(
            TxId::new([0; 32]),
            0,
        ),
    })
}

#[handler]
fn return_err() -> Result<&'static str, NotFoundError> {
    Err(NotFoundError)
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let app = Route::new()
        .at("/generate_wallet/", post(generate_wallet))
        .at("/spend_funds/", post(spend_funds));

    Server::new(TcpListener::bind("127.0.0.1:3000"))
        .run(app)
        .await
}

#[cfg(test)]
mod tests {
    use crate::index;
    use poem::test::TestClient;

    #[tokio::test]
    async fn test_index() {
        let resp = TestClient::new(index).get("/").send().await;
        resp.assert_status_is_ok();
        resp.assert_text("Authsome!").await;
    }

    // send
    #[tokio::test]
    async fn test_generate_wallet() {
        let resp = TestClient::new(index).get("/").send().await;
        resp.assert_status_is_ok();
        resp.assert_text("Authsome!").await;
    }

    // send invalid public keys, endpoint should error
    #[tokio::test]
    async fn test_generate_wallet_erroneous_public_keys() {
        let resp = TestClient::new(index).get("/").send().await;
        resp.assert_status_is_ok();
        resp.assert_text("Authsome!").await;
    }
}
