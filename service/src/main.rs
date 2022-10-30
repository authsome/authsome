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
use fuels_signers::fuel_crypto::PublicKey;
use fuels_signers::fuel_crypto::SecretKey;
use fuels_core::parameters::TxParameters;
use fuels_types::bech32::Bech32Address;

const NODE_URL: &str = "node-beta-1.fuel.network";

const PREDICATE_TEMPLATE: &str = "predicate;

use std::{b512::B512, constants::ZERO_B256, ecr::ec_recover_address, inputs::input_predicate_data, prelude::*};

fn get_predicate_input_index() -> u8 {
    asm() {
        gm index i3;
        index: u8
    }
}

fn get_output_index(input_index:u8) -> u64 {
    asm(input_index) {
        gtf output_index input_index i259;
        output_index: u64
    }
}

fn get_tx_id_memory_address(input_index:u8) -> u64 {
    asm(input_index) {
        gtf output_index input_index i258;
        output_index: u64
    }
}

fn get_tx_id_at_address(address: u64) -> b256 {
    let mut i = 0;
    while i < 4 {
        let output = asm(address) -> u64 {
            lw output address i0;
            output
        }
    }

}

fn extract_public_key_and_match(signature: B512, expected_public_key: b256) -> u64 {
    let predicate_input_index = get_predicate_input_index();
    let output_index = get_output_index(predicate_input_index);
    let tx_id_memory_address = get_tx_id_memory_address(predicate_input_index);
    let tx_id_at_address = get_tx_id_at_address(tx_id_memory_address);
    if let Result::Ok(pub_key_sig) = ec_recover_address(signature, ZERO_B256)
    {
        if pub_key_sig.value == expected_public_key {
            return 1;
        }
    }
    return 0;       
}

fn main() -> bool {
    let signatures: [B512; 3] = input_predicate_data(0);

    let public_keys = [
        {{public_key_1}},
        {{public_key_2}},
        {{public_key_2}},
    ];

    let mut matched_keys = 0;

    matched_keys = extract_public_key_and_match(signatures[0], public_keys[0]);
    matched_keys = matched_keys + extract_public_key_and_match(signatures[1], public_keys[1]);
    matched_keys = matched_keys + extract_public_key_and_match(signatures[2], public_keys[2]);

    return matched_keys > 1;
}
";

#[derive(Deserialize)]
struct GenerateWalletRequest {
    public_keys: [PublicKey; 3],
}

#[derive(Serialize)]
struct GenerateWalletResponse {
    public_keys: [PublicKey; 3],
    wallet: Address,
}

/*
Generation of multi-sig wallet.
Receive three public keys to generate one multi-sig wallet address.
*/
#[handler]
fn generate_wallet(req: Json<GenerateWalletRequest>) -> Json<GenerateWalletResponse> {

    let template = 
}

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
