/*
Assumes
- `forc` is installed and in path
- internet connection ...
*/
#[macro_use]
extern crate lazy_static;

use std::collections::BTreeMap;
use std::fs;
use std::io::Error;
use std::io::ErrorKind;
use std::process::Command;
use std::collections::HashMap;
use std::sync::Mutex;

use sha2::{Digest, Sha256};


use poem::error::{BadGateway, InternalServerError};
use poem::{handler, listener::TcpListener, post, web::Json, Result, Route, Server};

use handlebars::Handlebars;
use serde::Deserialize;
use serde::Serialize;

use fuel_gql_client::fuel_tx::Address;
use fuel_gql_client::fuel_tx::AssetId;
use fuel_gql_client::fuel_tx::ContractId;
use fuels_contract::predicate::Predicate;
use fuels_core::parameters::TxParameters;
use fuels_core::tx::Receipt;
use fuels_signers::fuel_crypto::PublicKey;
use fuels_signers::fuel_crypto::SecretKey;
use fuels_signers::provider::Provider;
use fuels_signers::Payload;
use fuels_signers::WalletUnlocked;
use fuels_types::bech32::Bech32Address;
use fuels_core::tx::UtxoId;

const N_PUBLIC_KEYS: usize = 3;

const NODE_URL: &str = "node-beta-1.fuel.network";

const FORCBUILD_DIR: &str = "/tmp/forcbuild/";
const PREDICATE_DIR_PATH: &str = "/tmp/predicates/";
const PREDICATE_OUTPUT_DIR: &str = "/tmp/predicates_bytecode_output/";

const FORCTOML: &str = "[project]
entry = \"main.sw\"
license = \"Apache-2.0\"
name = \"contracts\"
";

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
    let output = ZERO_B256;
    while i < 4 {
        let output = asm(address) {
            lw output address i0;
            output
        };
    }
    output
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

    matched_keys > 1
}
";

lazy_static! {
    static ref BYTE_CODE_LOOKUP: Mutex<HashMap<String, Vec<u8>>> =  {
        let lookup = HashMap::new();
        Mutex::new(lookup)
    };
}

#[derive(Deserialize)]
struct GenerateWalletRequest {
    public_keys: [PublicKey; 3],
}

#[derive(Serialize)]
struct GenerateWalletResponse {
    public_keys: [PublicKey; 3],
    wallet: Address,
}

fn hash_publickeys(public_keys: [PublicKey; N_PUBLIC_KEYS]) -> String {
    let mut public_keys_concatenated = String::new();
    let mut public_keys_sorted = public_keys;
    public_keys_sorted.sort(); // for determinism
    for public_key in public_keys_sorted {
        public_keys_concatenated.push_str(&public_key.to_string());
    }

    let mut hasher = Sha256::new();
    hasher.update(public_keys_concatenated);
    let result = hasher.finalize();
    let result_string = format!("{:x}", result);
    result_string
}

/*
Generation of multi-sig wallet.
Receive three public keys to generate one multi-sig wallet address.

fill PREDICATE_TEMPLATE  -> P
compile P -> bytecode (exec)
hash bytecode (obtain address) -> H
    .address from rust-sdk -- how to use predicate
memory map of server -- random file -- so that in other function, we can retrieve bytecode of file wiht unieque name from hash key
*/
#[handler]
fn generate_wallet(req: Json<GenerateWalletRequest>) -> Result<Json<GenerateWalletResponse>> {
    // TODO ensure concurrency safety

    let mut handlebars = Handlebars::new();
    match handlebars.register_template_string("t1", PREDICATE_TEMPLATE) {
        Err(err) => return Err(InternalServerError(err)),
        _ => (),
    };

    let mut data = BTreeMap::new();
    for (i, public_key) in req.public_keys.iter().enumerate() {
        data.insert(format!("public_key_{}", i + 1), public_key.to_string());
    }
    let rendered_template = handlebars.render("t1", &data).unwrap();
    let h = hash_publickeys(req.public_keys);

    /*
    every time we generate a wallet, we create new folder named with hash of aggregated public keys
    in /tmp/predicates/
    each having src/main.sw and Forc.toml
     */

    let project_dir = format!("{}{}", PREDICATE_DIR_PATH, h);
    match fs::write(&format!("{}{}", project_dir, "Forc.toml"), FORCTOML) {
        Err(err) => return Err(InternalServerError(err)),
        _ => (),
    };

    match fs::create_dir_all(&format!("{}{}", project_dir, "src/")) {
        Err(err) => return Err(InternalServerError(err)),
        _ => (),
    }

    match fs::write(
        &format!("{}{}", project_dir, "src/main.sw"),
        rendered_template,
    ) {
        Err(err) => return Err(InternalServerError(err)),
        _ => (),
    }

    let bytecode_file_path = format!("{}{}/predicate.bin", PREDICATE_OUTPUT_DIR, h);
    let _output = execute_command(
        format!(
            "forc build --path {}{} --output {}",
            PREDICATE_DIR_PATH, h, bytecode_file_path,
        )
        .as_str(),
    );

    let predicate = Predicate::load_from(&bytecode_file_path);
    match predicate {
        Ok(predicate) => {
            let wallet = predicate.address();
            let code = predicate.code().clone();
            let mut lookup = BYTE_CODE_LOOKUP.lock().unwrap();
            lookup.insert(wallet.to_string(), code);
            Ok(Json(GenerateWalletResponse {
                public_keys: req.public_keys,
                wallet: wallet.into(),
            }))
        },
        Err(err) => Err(InternalServerError(err)),
    }
}

fn execute_command(command: &str) -> String {
    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .expect("failed to execute process");
    let output_string = String::from_utf8_lossy(&output.stdout);
    output_string.to_string()
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

    // convert the inputs we want to spend into payloads with concatenated signatures
    let payloads = req
        .inputs
        .iter()
        .map(|input| Payload {
            utxo_id: input.utxo_id,
            data: input.signatures.concat(),
        })
        .collect();

    let code = {
        let lookup = BYTE_CODE_LOOKUP.lock().unwrap();
        match lookup.get(&wallet.to_string()) {
            Some(code) => code.clone(),
            None => return Err(InternalServerError(Error::new(
                ErrorKind::Unsupported,
                "could not find byte code",
            ))),
        }
    };

    let receipts = match unlocked
        .multi_spend_predicate(
            &wallet,
            code,
            req.asset_id,
            req.amount,
            &recipient,
            payloads,
            TxParameters::default(),
        )
        .await
    {
        Ok(receipts) => receipts,
        Err(err) => return Err(InternalServerError(err)),
    };

    if receipts.len() != 1 {
        return Err(InternalServerError(Error::new(
            ErrorKind::Unsupported,
            "invalid receipts number",
        )));
    }

    let tx_id = match receipts.first().unwrap() {
        receipt @ Receipt::Transfer { .. } => receipt.id().unwrap(),
        _ => {
            return Err(InternalServerError(Error::new(
                ErrorKind::Unsupported,
                "invalid receipt type",
            )))
        }
    };

    let inputs = req
        .inputs
        .iter()
        .map(|input| InputNoSig {
            utxo_id: input.utxo_id,
        })
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

fn init() -> Result<(), std::io::Error> {
    fs::create_dir_all(FORCBUILD_DIR).unwrap();
    fs::write(&format!("{}", FORCBUILD_DIR), &FORCTOML)
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {


    match init() {
        Err(err) => return Err(err),
        _ => (),
    };

    let app = Route::new()
        .at("/generate_wallet/", post(generate_wallet))
        .at("/spend_funds/", post(spend_funds));

    Server::new(TcpListener::bind("127.0.0.1:3000"))
        .run(app)
        .await
}

#[cfg(test)]
mod tests {
    // use crate::index;
    // use poem::test::TestClient;

    // #[tokio::test]
    // async fn test_index() {
    //     let resp = TestClient::new(index).get("/").send().await;
    //     resp.assert_status_is_ok();
    //     resp.assert_text("Authsome!").await;
    // }

    // send
    // #[tokio::test]
    // async fn test_generate_wallet() {
    //     let resp = TestClient::new(index).get("/").send().await;
    //     resp.assert_status_is_ok();
    //     resp.assert_text("Authsome!").await;
    // }

    // send invalid public keys, endpoint should error
    // #[tokio::test]
    // async fn test_generate_wallet_erroneous_public_keys() {
    //     let resp = TestClient::new(index).get("/").send().await;
    //     resp.assert_status_is_ok();
    //     resp.assert_text("Authsome!").await;
    // }
}
