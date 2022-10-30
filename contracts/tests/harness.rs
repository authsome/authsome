use fuels::prelude::*;

#[tokio::test]
async fn ec_recover_and_match_predicate_test() -> Result<(), Error> {
    use fuels::contract::predicate::Predicate;
    use fuels::signers::fuel_crypto::SecretKey;

    let secret_key1: SecretKey =
        "0x862512a2363db2b3a375c0d4bbbd27172180d89f23f2e259bac850ab02619301"
            .parse()
            .unwrap();

    let secret_key2: SecretKey =
        "0x37fa81c84ccd547c30c176b118d5cb892bdb113e8e80141f266519422ef9eefd"
            .parse()
            .unwrap();

    let secret_key3: SecretKey =
        "0x976e5c3fa620092c718d852ca703b6da9e3075b9f2ecb8ed42d9f746bf26aafb"
            .parse()
            .unwrap();

    let provider = Provider::connect("node-beta-1.fuel.network").await.unwrap();

    let mut wallet = WalletUnlocked::new_from_private_key(secret_key1, Some(provider.clone()));
    let mut wallet2 = WalletUnlocked::new_from_private_key(secret_key2, Some(provider.clone()));
    let mut wallet3 = WalletUnlocked::new_from_private_key(secret_key3, Some(provider.clone()));
    let receiver = WalletUnlocked::new_random(Some(provider.clone()));

    let predicate = Predicate::load_from(
        "out/debug/contracts.bin",
    )?;

    // dbg!("wallets", wallet.clone(), wallet2.clone(), wallet3.clone());

    let predicate_code = predicate.code();
    let predicate_address = predicate.address();
    let amount_to_predicate = 1;
    let asset_id = AssetId::default();
    dbg!("predicate address", predicate_address);

/* 
    dbg!("predicate ", wallet.clone());

    let mut inputs = vec![];
    let mut outputs = vec![];
    let input = wallet.get_asset_inputs_for_amount(asset_id, amount_to_predicate, 0).await?;
    inputs.extend(input);

    let output = wallet.get_asset_outputs_for_amount(predicate_address, asset_id, amount_to_predicate);
    outputs.extend(output);

    let mut tx = Wallet::build_transfer_tx(&inputs, &outputs, TxParameters::default());
    wallet.sign_transaction(&mut tx).await?;

    let _receipts = provider.send_transaction(&tx).await?;

    dbg!("receipts", _receipts);
    */

    /* 
    wallet
        .transfer(
            predicate_address,
            amount_to_predicate,
            asset_id,
            TxParameters::default(),
        )
        .await?; 
        */
 
    let predicate_balance = provider
        .get_asset_balance(predicate.address(), asset_id)
        .await?;


    let data_to_sign = [0; 32];
    let signature1 = wallet.sign_message(&data_to_sign).await?.to_vec();
    let signature2 = wallet2.sign_message(&data_to_sign).await?.to_vec();
    let signature3 = wallet3.sign_message(&data_to_sign).await?.to_vec();

    let signatures = vec![signature1, signature2, signature3];

    let predicate_data = signatures.into_iter().flatten().collect();
    wallet
        .spend_predicate(
            predicate_address,
            predicate_code,
            amount_to_predicate,
            asset_id,
            receiver.address(),
            Some(predicate_data),
            TxParameters::new(Some(1), Some(100), Some(100)),
        )
        .await?;

    let receiver_balance_after = provider
        .get_asset_balance(receiver.address(), asset_id)
        .await?;
    //assert_eq!(amount_to_predicate, receiver_balance_after);

    let predicate_balance = provider
        .get_asset_balance(predicate.address(), asset_id)
        .await?;
    //assert_eq!(predicate_balance, 0);

    Ok(())
}