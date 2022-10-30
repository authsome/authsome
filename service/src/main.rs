use std::io::Error;
use std::io::ErrorKind;

use poem::error::BadGateway;
use poem::error::InternalServerError;
use poem::handler;
use poem::listener::TcpListener;
use poem::post;
use poem::web::Json;
use poem::Result;
use poem::Route;
use poem::Server;

use serde::Serialize;
use serde::Deserialize;

use fuel_gql_client::fuel_tx::Address;
use fuel_gql_client::fuel_tx::AssetId;
use fuel_gql_client::fuel_tx::ContractId;
use fuels_signers::WalletUnlocked;
use fuels_signers::Payload;
use fuels_signers::provider::Provider;
use fuels_signers::fuel_crypto::PublicKey;
use fuels_signers::fuel_crypto::SecretKey;
use fuels_core::parameters::TxParameters;
use fuels_core::tx::Receipt;
use fuels_core::tx::UtxoId;
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
    Json(GenerateWalletResponse{
        public_keys: req.public_keys,
        wallet: Address::new([0;32]),
    })
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
    tx_id: ContractId,
}

#[derive(Serialize)]
struct InputNoSig {
    utxo_id: UtxoId,
}


#[handler]
async fn spend_funds(req: Json<SpendFundsRequest>) -> Result<Json<SpendFundsResponse>> {

    // initialize the provider and the wallet with given private key
    let provider = match Provider::connect(NODE_URL).await {
        Ok(provider) => provider,
        Err(err) => return Err(BadGateway(err)),
    };
    let secret = unsafe { SecretKey::from_bytes_unchecked([0; 32]) };
    let unlocked = WalletUnlocked::new_from_private_key(secret, Some(provider));

    // convert address strings to Bech32 addresses (which can't be deserialized directly)
    let wallet = Bech32Address::from(req.wallet);
    let recipient = Bech32Address::from(req.recipient);

    // TODO: retrieve code for this wallet address, probably from a mapping that has wallet address as key
    let code = Vec::<u8>::new();

    // convert the inputs we want to spend into payloads with concatenated signatures
    let payloads = req.inputs
        .iter()
        .map(|input| Payload{
            utxo_id: input.utxo_id,
            data: input.signatures.concat(),
        })
        .collect();

    let receipts = match unlocked.multi_spend_predicate(
        &wallet,
        code,
        req.asset_id,
        req.amount,
        &recipient,
        payloads,
        TxParameters::default(),
    ).await {
        Ok(receipts) => receipts,
        Err(err) => return Err(InternalServerError(err)),
    };

    if receipts.len() != 1 {
        return Err(InternalServerError(Error::new(ErrorKind::Unsupported, "invalid receipts number")))
    }

    let tx_id = match receipts.first().unwrap() {
        receipt @ Receipt::Transfer {..} => receipt.id().unwrap(),
        _ => return Err(InternalServerError(Error::new(ErrorKind::Unsupported, "invalid receipt type"))),
    };

    let inputs = req.inputs
        .iter()
        .map(|input| InputNoSig{utxo_id: input.utxo_id})
        .collect();

    Ok(Json(SpendFundsResponse {
        wallet: req.wallet,
        asset_id: req.asset_id,
        amount: req.amount,
        recipient: req.recipient,
        inputs: inputs,
        tx_id: *tx_id,
    }))
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
