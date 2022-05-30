use fuels::{prelude::*};
use fuels::tx::*;
use fuel_tx::Contract;
use std::fs;
use fuel_crypto::SecretKey;
use fuels::{signers::wallet::*};
use fuels::signers::provider::*;
use fuels::tx::Transaction;
use std::str::FromStr;
use fuel_core::service::{Config, FuelService};
use fuel_gql_client::{
    client::{FuelClient,schema::coin::Coin, types::TransactionResponse, PaginatedResult, PaginationRequest},
    fuel_tx::{Address, AssetId, Input, Output, Receipt, UtxoId, Witness},
};
use std::net::SocketAddr;
use fuel_vm::prelude::Opcode;
use fuel_vm::consts::*;

async fn configure(provider: Provider) -> (Wallet, Wallet) {

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

#[tokio::main]
async fn main() {
    let final_receiver = "0xde97d8624a438121b86a1956544bd72ed68cd69f2c99555b08b1e8c51ffd511c";

    let mut config = Config::local_node();
    config.predicates = true;
    config.utxo_validation = true;

    let srv = FuelService::new_node(config).await.unwrap();
    let client = FuelClient::from(srv.bound_address);
    let provider = Provider::new(client.clone());
    
    let (wallet, wallet2) = configure(provider.clone()).await;
    // Lock coin 

    let amount = wallet.get_asset_balance(&AssetId::default()).await.unwrap();
    let wallet_coins = wallet.get_asset_inputs_for_amount(AssetId::default(), amount, 0).await.unwrap();
    // We load the predicate_code and hash it to get the predicate root hash
    let predicate_code = fs::read("../predicate/out/debug/single-sig.bin").unwrap();
    let predicate_hash = (*Contract::root_from_code(&predicate_code)).into();

    // Output is locked behind predicate_hash
    let output_coin = Output::coin(
        predicate_hash,
        1000,
        AssetId::default(),
    );
    // Add change output
    let mut output_change = Output::change(
        wallet.address(),
        0,
        AssetId::default(),
    );
    // Craft tx
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

    let tx_id = tx.id();
    //println!("Transaction Id {:?}", tx_id);
    let signature = wallet.sign_transaction(&mut tx).await.unwrap();    
    let mut tx_receipts = provider.send_transaction(&tx).await.unwrap();
    println!("{:?}", tx_receipts);
    // Check there are UTXO locked with the predicate hash
    let utxo_predicate_hash = provider.get_spendable_coins(&predicate_hash, AssetId::default(), 1000).await.unwrap();
    let mut inputs = vec![];
    let mut tot_amount = 0;
    for coin in utxo_predicate_hash {
        let input_coin = Input::coin_predicate(
            UtxoId::from(coin.utxo_id),
            coin.owner.into(),
            coin.amount.0,
            AssetId::default(),
            0,
            predicate_code.clone(),
            vec![0],
        );
        inputs.push(input_coin);
        tot_amount += coin.amount.0;
    }
    // Output 
    println!("{}", tot_amount);
    let new_output_coin = Output::coin(
        Address::from_str(final_receiver).unwrap(),
        tot_amount,
        AssetId::default(),
    );

    output_change = Output::change(
        predicate_hash,
        0,
        AssetId::default(),
    );

    let mut new_tx = Transaction::script(
        0,
        1000000,
        0,
        0,
        script.clone(),
        vec![],
        inputs,
        vec![new_output_coin, output_change],
        vec![],
    );
    //println!("{:?}", new_tx);
    let asset_id = AssetId::default();
    let signature = wallet.sign_transaction(&mut new_tx).await.unwrap(); 
    //let sign_s =  wallet_s.sign_transaction(&mut new_tx).await.unwrap(); 
    tx_receipts = provider.send_transaction(&new_tx).await.unwrap();
    //println!("{:?}", tx_receipts);
    /*
    let balance_receiver = client.balance(final_receiver, 
            Some(format!("{:#x}", asset_id).as_str())).await.unwrap();
    let wallet_address = "0xe10f526b192593793b7a1559a391445faba82a1d669e3eb2dcd17f9c121b24b1";
    let balance_sender = client.balance(wallet_address,
        Some(format!("{:#x}", asset_id).as_str())
    ).await.unwrap();
    println!("rec { }, send { }", balance_receiver, balance_sender);
    */
}
