/*
We implement this backend service, instead of using the TypeScript SDK directly in the frontend, because:

- Wallet generation requires compilation of Sway code to obtain the predicate byte code, which in turn gives us the multi-sig wallet address (as a hash of the byte code). This would be hard to do in the front-end.
- The TypeScript SDK has no support for predicates, which would make it impossible to spend funds from the multi-sig wallet. There is rudimentary support for predicates in the Rust SDK which we can buid on.

API spec:
(possible extension is to implement a OpenAPI spec for it)
TBD
 */

use poem::{error::NotFoundError, get, handler, listener::TcpListener, Result, Route, Server};

#[handler]
fn index() -> String {
    "Authsome!".to_string()
}

/*
Generation of multi-sig wallet.
Receive three public keys to generate one multi-sig wallet address.
*/
#[handler]
fn generate_wallet(req: Json<Request>) -> Json<Response> {
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
#[handler]
fn spend_fund() -> String {
    "spend fund TODO!".to_string()
}

#[handler]
fn return_err() -> Result<&'static str, NotFoundError> {
    Err(NotFoundError)
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let app = Route::new()
        .at("/", get(index))
        .at("/generate_wallet/", post(generate_wallet))
        .at("/spend_fund/", post(spend_fund));

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
