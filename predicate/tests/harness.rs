use fuels::{prelude::*};
use fuel_tx::Contract;
use std::fs;
use fuel_crypto::SecretKey;
use fuels::{signers::wallet::*};
use fuels::tx::Transaction;
use std::str::FromStr;
use fuel_core::service::{Config, FuelService};
use fuel_gql_client::{
    client::{FuelClient,schema::coin::Coin},
    fuel_tx::{Address, AssetId, Input, Output, Receipt, UtxoId, Witness},
};
use fuel_vm::prelude::Opcode;
use fuel_vm::consts::*;


async fn configure_wallets(provider: Provider) -> (Wallet, Wallet) {

    let secret_key1 = 
        SecretKey::from_str("0x862512a2363db2b3a375c0d4bbbd27172180d89f23f2e259bac850ab02619301")
            .unwrap();

    let secret_key2 =
        SecretKey::from_str("0x37fa81c84ccd547c30c176b118d5cb892bdb113e8e80141f266519422ef9eefd")
            .unwrap();

    
    let mut wallet = Wallet::new_from_private_key(secret_key1, None);
    let mut wallet2 = Wallet::new_from_private_key(secret_key2, None);

    wallet.set_provider(provider.clone());
    wallet2.set_provider(provider.clone());
    (wallet, wallet2)
    
}

async fn send_coins_to_predicate_hash(asset_id: AssetId, wallet: Wallet, provider: Provider, amount_to_predicate: u64) {
    let amount = wallet.get_asset_balance(&asset_id).await.unwrap();
    let wallet_coins = wallet.get_asset_inputs_for_amount(asset_id, amount, 0).await.unwrap();
    // We load the predicate_code and hash it to get the predicate root hash
    let predicate_code = fs::read("./out/debug/multisignature_predicate.bin").unwrap();
    let predicate_hash = (*Contract::root_from_code(&predicate_code)).into();

    // Output is locked behind predicate_hash
    let output_coin = Output::coin(
        predicate_hash,
        amount_to_predicate,
        asset_id,
    );
    // Add change output
    let mut output_change = Output::change(
        wallet.address(),
        0,
        asset_id,
    );
    // Build 
    let script = Opcode::RET(REG_ONE).to_bytes().to_vec();
    //script.push(1);
    let mut tx = Transaction::script(
        1,
        1000000,
        1,
        0,
        script.clone(),
        vec![],
        wallet_coins,
        vec![output_coin, output_change],
        vec![],
    );
    let signature = wallet.sign_transaction(&mut tx).await.unwrap();    
    let mut tx_receipts = provider.send_transaction(&tx).await.unwrap();
}


async fn craft_predicate_spending_tx(receiver_address: Address, asset_id: AssetId, amount_to_spend: u64, provider: Provider, predicate_data: &mut [u8]) -> Transaction {
    let predicate_code = fs::read("./out/debug/multisignature_predicate.bin").unwrap();
    let predicate_hash = (*Contract::root_from_code(&predicate_code)).into();
    let utxo_predicate_hash = provider.get_spendable_coins(&predicate_hash, asset_id, amount_to_spend).await.unwrap();
    let mut inputs = vec![];
    let mut tot_amount = 0;

    for coin in utxo_predicate_hash {
        let input_coin = Input::coin_predicate(
            UtxoId::from(coin.utxo_id),
            coin.owner.into(),
            coin.amount.0,
            asset_id,
            0,
            predicate_code.clone(),
            predicate_data.to_vec(),
        );
        inputs.push(input_coin);
        tot_amount += coin.amount.0;
    }


    let output_coin = Output::coin(
        receiver_address,
        tot_amount,
        asset_id,
    );

    let output_change = Output::change(
        predicate_hash,
        0,
        asset_id,
    );

    let mut new_tx = Transaction::script(
        0,
        1000000,
        0,
        0,
        vec![],
        vec![],
        inputs,
        vec![output_coin, output_change],
        vec![],
    );
    new_tx
}
/*
#[tokio::test]
async fn single_signer() {
    let receiver_address = Address::from_str("0xde97d8624a438121b86a1956544bd72ed68cd69f2c99555b08b1e8c51ffd511c").unwrap();

    let mut config = Config::local_node();
    config.predicates = true;
    config.utxo_validation = true;

    let srv = FuelService::new_node(config).await.unwrap();
    let client = FuelClient::from(srv.bound_address);
    let provider = Provider::new(client.clone());

    let asset_id = AssetId::default();
    let (initial_wallet, second_signer) = configure_wallets(provider.clone()).await;
    let amount_to_predicate = 1000;
    
    send_coins_to_predicate_hash(asset_id, initial_wallet.clone(), provider.clone(), amount_to_predicate).await;

    // Check there are UTXO locked with the predicate hash
    let predicate_code = fs::read("./out/debug/multisignature_predicate.bin").unwrap();
    let predicate_hash = (*Contract::root_from_code(&predicate_code)).into();
    let mut predicate_balance = provider.get_asset_balance(&predicate_hash, asset_id, ).await.unwrap();

    assert_eq!(predicate_balance, amount_to_predicate);
    let mut predicate_data = vec![0];
    let mut predicate_spending_tx = craft_predicate_spending_tx(receiver_address, asset_id, predicate_balance, provider.clone(), &mut predicate_data).await;
    let receiver_balance_before = provider.get_asset_balance(&receiver_address, asset_id).await.unwrap();

    // Execute tx
    let _signature = initial_wallet.sign_transaction(&mut predicate_spending_tx).await.unwrap(); 
    //let sign_s =  wallet_s.sign_transaction(&mut new_tx).await.unwrap(); 
    let _tx_receipts = provider.send_transaction(&predicate_spending_tx).await.unwrap();

    // Check the balance of the receiver 
    let receiver_balance_after = provider.get_asset_balance(&receiver_address, asset_id, ).await.unwrap();
    assert_eq!(receiver_balance_before + predicate_balance, receiver_balance_after);

    // Check we spent the entire predicate hash input
    predicate_balance = provider.get_asset_balance(&predicate_hash, asset_id).await.unwrap();
    assert_eq!(predicate_balance, 0);
}
*/

#[tokio::test]
async fn dual_signer() {
    let receiver_address = Address::from_str("0xde97d8624a438121b86a1956544bd72ed68cd69f2c99555b08b1e8c51ffd511c").unwrap();

    let mut config = Config::local_node();
    config.predicates = true;
    config.utxo_validation = true;

    let srv = FuelService::new_node(config).await.unwrap();
    let client = FuelClient::from(srv.bound_address);
    let provider = Provider::new(client.clone());

    let asset_id = AssetId::default();
    let (initial_wallet, second_signer) = configure_wallets(provider.clone()).await;
    let amount_to_predicate = 1000;
    
    send_coins_to_predicate_hash(asset_id, initial_wallet.clone(), provider.clone(), amount_to_predicate).await;

    // Check there are UTXO locked with the predicate hash
    let predicate_code = fs::read("./out/debug/multisignature_predicate.bin").unwrap();
    let predicate_hash = (*Contract::root_from_code(&predicate_code)).into();
    let mut predicate_balance = provider.get_asset_balance(&predicate_hash, asset_id, ).await.unwrap();

    assert_eq!(predicate_balance, amount_to_predicate);
    let mut predicate_data = vec![0, 1];
    let mut predicate_spending_tx = craft_predicate_spending_tx(receiver_address, asset_id, predicate_balance, provider.clone(), &mut predicate_data).await;
    let receiver_balance_before = provider.get_asset_balance(&receiver_address, asset_id).await.unwrap();

    // Execute tx
    let _signature = initial_wallet.sign_transaction(&mut predicate_spending_tx).await.unwrap(); 
    let _second_signature =  second_signer.sign_transaction(&mut predicate_spending_tx).await.unwrap(); 
    let _tx_receipts = provider.send_transaction(&predicate_spending_tx).await.unwrap();

    // Check the balance of the receiver 
    let receiver_balance_after = provider.get_asset_balance(&receiver_address, asset_id, ).await.unwrap();
    assert_eq!(receiver_balance_before + predicate_balance, receiver_balance_after);

    // Check we spent the entire predicate hash input
    predicate_balance = provider.get_asset_balance(&predicate_hash, asset_id).await.unwrap();
    assert_eq!(predicate_balance, 0);
}

#[tokio::test]
async fn dual_signer_other_order() {
    let receiver_address = Address::from_str("0xde97d8624a438121b86a1956544bd72ed68cd69f2c99555b08b1e8c51ffd511c").unwrap();

    let mut config = Config::local_node();
    config.predicates = true;
    config.utxo_validation = true;

    let srv = FuelService::new_node(config).await.unwrap();
    let client = FuelClient::from(srv.bound_address);
    let provider = Provider::new(client.clone());

    let asset_id = AssetId::default();
    let (initial_wallet, second_signer) = configure_wallets(provider.clone()).await;
    let amount_to_predicate = 1000;
    
    send_coins_to_predicate_hash(asset_id, initial_wallet.clone(), provider.clone(), amount_to_predicate).await;

    // Check there are UTXO locked with the predicate hash
    let predicate_code = fs::read("./out/debug/multisignature_predicate.bin").unwrap();
    let predicate_hash = (*Contract::root_from_code(&predicate_code)).into();
    let mut predicate_balance = provider.get_asset_balance(&predicate_hash, asset_id, ).await.unwrap();

    assert_eq!(predicate_balance, amount_to_predicate);
    let mut predicate_data = vec![0, 1];
    let mut predicate_spending_tx = craft_predicate_spending_tx(receiver_address, asset_id, predicate_balance, provider.clone(), &mut predicate_data).await;
    let receiver_balance_before = provider.get_asset_balance(&receiver_address, asset_id).await.unwrap();

    // Execute tx
    let _second_signature =  second_signer.sign_transaction(&mut predicate_spending_tx).await.unwrap(); 
    let _signature = initial_wallet.sign_transaction(&mut predicate_spending_tx).await.unwrap(); 
    let _tx_receipts = provider.send_transaction(&predicate_spending_tx).await.unwrap();

    // Check the balance of the receiver 
    let receiver_balance_after = provider.get_asset_balance(&receiver_address, asset_id, ).await.unwrap();
    assert_eq!(receiver_balance_before + predicate_balance, receiver_balance_after);

    // Check we spent the entire predicate hash input
    predicate_balance = provider.get_asset_balance(&predicate_hash, asset_id).await.unwrap();
    assert_eq!(predicate_balance, 0);
}


#[tokio::test]
#[should_panic(expected = "InvalidPredicate")]
async fn single_signer_wrong_wallet() {
    let receiver_address = Address::from_str("0x6b63804cfbf9856e68e5b6e7aef238dc8311ec55bec04df774003a2c96e0418e").unwrap();

    let wrong_secret_key = 
        SecretKey::from_str("0xde97d8624a438121b86a1956544bd72ed68cd69f2c99555b08b1e8c51ffd511c")
            .unwrap();

    let mut wrong_wallet = Wallet::new_from_private_key(wrong_secret_key, None);


    let mut config = Config::local_node();
    config.predicates = true;
    config.utxo_validation = true;

    let srv = FuelService::new_node(config).await.unwrap();
    let client = FuelClient::from(srv.bound_address);
    let provider = Provider::new(client.clone());
    
    let asset_id = AssetId::default();
    let (initial_wallet, second_signer) = configure_wallets(provider.clone()).await;
    wrong_wallet.set_provider(provider.clone());
    let amount_to_predicate = 1000;
    
    send_coins_to_predicate_hash(asset_id, initial_wallet.clone(), provider.clone(), amount_to_predicate).await;

    // Check there are UTXO locked with the predicate hash
    let predicate_code = fs::read("./out/debug/multisignature_predicate.bin").unwrap();
    let predicate_hash = (*Contract::root_from_code(&predicate_code)).into();
    let mut predicate_balance = provider.get_asset_balance(&predicate_hash, asset_id, ).await.unwrap();

    assert_eq!(predicate_balance, amount_to_predicate);
    let mut predicate_data = vec![0, 1];
    let mut predicate_spending_tx = craft_predicate_spending_tx(receiver_address, asset_id, predicate_balance, provider.clone(), &mut predicate_data).await;
    let receiver_balance_before = provider.get_asset_balance(&receiver_address, asset_id).await.unwrap();

    // Execute tx
    let __signature =  wrong_wallet.sign_transaction(&mut predicate_spending_tx).await.unwrap(); 
    //let _second_signature = initial_wallet.sign_transaction(&mut predicate_spending_tx).await.unwrap(); 
    let _tx_receipts = provider.send_transaction(&predicate_spending_tx).await.unwrap();

    // Check the balance of the receiver 
    let receiver_balance_after = provider.get_asset_balance(&receiver_address, asset_id, ).await.unwrap();
    assert_eq!(receiver_balance_before + predicate_balance, receiver_balance_after);

    // Check we spent the entire predicate hash input
    predicate_balance = provider.get_asset_balance(&predicate_hash, asset_id).await.unwrap();
    assert_eq!(predicate_balance, 0);
}

#[tokio::test]
#[should_panic(expected = "InvalidPredicate")]
async fn single_signer_twice() {
    let receiver_address = Address::from_str("0x6b63804cfbf9856e68e5b6e7aef238dc8311ec55bec04df774003a2c96e0418e").unwrap();

    let wrong_secret_key = 
        SecretKey::from_str("0xde97d8624a438121b86a1956544bd72ed68cd69f2c99555b08b1e8c51ffd511c")
            .unwrap();

    let mut wrong_wallet = Wallet::new_from_private_key(wrong_secret_key, None);


    let mut config = Config::local_node();
    config.predicates = true;
    config.utxo_validation = true;

    let srv = FuelService::new_node(config).await.unwrap();
    let client = FuelClient::from(srv.bound_address);
    let provider = Provider::new(client.clone());
    
    let asset_id = AssetId::default();
    let (initial_wallet, second_signer) = configure_wallets(provider.clone()).await;
    wrong_wallet.set_provider(provider.clone());
    let amount_to_predicate = 1000;
    
    send_coins_to_predicate_hash(asset_id, initial_wallet.clone(), provider.clone(), amount_to_predicate).await;

    // Check there are UTXO locked with the predicate hash
    let predicate_code = fs::read("./out/debug/multisignature_predicate.bin").unwrap();
    let predicate_hash = (*Contract::root_from_code(&predicate_code)).into();
    let mut predicate_balance = provider.get_asset_balance(&predicate_hash, asset_id, ).await.unwrap();

    assert_eq!(predicate_balance, amount_to_predicate);
    let mut predicate_data = vec![0, 1];
    let mut predicate_spending_tx = craft_predicate_spending_tx(receiver_address, asset_id, predicate_balance, provider.clone(), &mut predicate_data).await;
    let receiver_balance_before = provider.get_asset_balance(&receiver_address, asset_id).await.unwrap();

    // Execute tx
    let __signature =  initial_wallet.sign_transaction(&mut predicate_spending_tx).await.unwrap(); 
    let _second_signature = initial_wallet.sign_transaction(&mut predicate_spending_tx).await.unwrap(); 
    let _tx_receipts = provider.send_transaction(&predicate_spending_tx).await.unwrap();

    // Check the balance of the receiver 
    let receiver_balance_after = provider.get_asset_balance(&receiver_address, asset_id, ).await.unwrap();
    assert_eq!(receiver_balance_before + predicate_balance, receiver_balance_after);

    // Check we spent the entire predicate hash input
    predicate_balance = provider.get_asset_balance(&predicate_hash, asset_id).await.unwrap();
    assert_eq!(predicate_balance, 0);
}
