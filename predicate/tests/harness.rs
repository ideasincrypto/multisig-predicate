use fuels::{prelude::*};
use fuels::tx::*;
use fuel_crypto::{Message, SecretKey};
use fuels_abigen_macro::abigen;
use fuels::{signers::wallet::*};
use fuels::signers::provider::*;
use fuels::tx::Transaction;
use std::str::FromStr;
use fuel_gql_client::client::FuelClient;
use std::net::SocketAddr;


#[tokio::test]
async fn test_single_sig_predicate() {
    let client_details = "127.0.0.1:4000";
    let addr: SocketAddr = client_details.parse().expect("Error");
    let client = FuelClient::from(addr);
    let provider = Provider::new(client);


    let secret_key = 
        SecretKey::from_str("0x976e5c3fa620092c718d852ca703b6da9e3075b9f2ecb8ed42d9f746bf26aafb")
            .unwrap();

    // Use the test helper to setup a test provider.
    //let (provider, _address) = setup_test_provider(vec![]).await;

    // Create first account from local private key
    let mut wallet = Wallet::new_from_private_key(secret_key, None);
    wallet.set_provider(provider.clone());
    
    let amount = wallet.get_asset_balance(&AssetId::default()).await.unwrap();
    assert_eq!(10000000, amount);

    let wallet_coins = wallet.get_asset_inputs_for_amount(AssetId::default(), 10000000, 0).await.unwrap();

    let input_coin = Input::coin_signed(
        UtxoId::new(Bytes32::zeroed(), 0),
        Address::from_str("0xf1e92c42b90934aa6372e30bc568a326f6e66a1a0288595e6e3fbd392a4f3e6e")
            .unwrap(),
        10000000,
        AssetId::from([0u8; 32]),
        0,
        0,
    );

    let output_coin = Output::coin(
        Address::from_str("0xde97d8624a438121b86a1956544bd72ed68cd69f2c99555b08b1e8c51ffd511c")
            .unwrap(),
        10000000,
        AssetId::default(),
    );

    let mut tx = Transaction::script(
        0,
        1000000,
        0,
        0,
        vec![],
        vec![],
        wallet_coins,
        vec![output_coin],
        vec![],
    );

    let signature = wallet.sign_transaction(&mut tx).await.unwrap();    
    // Send tx locked with single sig predicate (hash of predicate) 
    // Try to spend the utxo in another tx by sending the predicate bytecode/data and the signature
    let receipt = provider.send_transaction(&tx).await.unwrap();
    let new_amount = wallet.get_asset_balance(&AssetId::default()).await.unwrap();
    assert_eq!(0, new_amount);

}
