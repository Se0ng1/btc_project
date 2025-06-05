use anyhow::Result;
use bitcoin::hashes::Hash;
use bitcoin::secp256k1::{rand::rngs::OsRng, Keypair, Message, Secp256k1};
use bitcoin::sighash::{Prevouts, SighashCache, TapSighashType};
use bitcoin::taproot::{LeafVersion, TapLeafHash, TaprootBuilder};
use bitcoin::{Address, Amount, Network, OutPoint, Sequence, Transaction, TxIn, TxOut, XOnlyPublicKey};
use bitcoincore_rpc::{Auth, Client, RpcApi};
use std::sync::Arc;

mod simple_script;
mod tx_analyzer;

use simple_script::simple_cat_checksig;
use tx_analyzer::TxAnalyzer;

fn main() -> Result<()> {
    env_logger::init();
    println!("OP_CAT + OP_CHECKSIG test");

    let secp = Secp256k1::new();
    let mut rng = OsRng;
    let keypair = Keypair::new(&secp, &mut rng);
    let xonly_pub: XOnlyPublicKey = keypair.x_only_public_key().0;
    println!(" X-only Pubkey: {}", xonly_pub);

    // taproot 주소 생성
    let leaf_script = simple_cat_checksig(&xonly_pub);
    let builder = TaprootBuilder::new()
        .add_leaf(0u8, leaf_script.clone())
        .map_err(|e| anyhow::anyhow!("add leaf err : {:?}", e))?;
    let taproot_info = builder
        .finalize(&secp, xonly_pub)
        .map_err(|e| anyhow::anyhow!("finalize err: {:?}", e))?;
    let tap_addr = Address::p2tr_tweaked(taproot_info.output_key(), Network::Regtest);
    println!("P2TR address: {}", tap_addr);

    let rpc = Arc::new(Client::new(
        "http://127.0.0.1:18443",
        Auth::UserPass("user".into(), "password".into()),
    )?);

    match rpc.list_wallets() {
        Ok(wallets) if wallets.is_empty() => {
            println!("create wallet");
            rpc.create_wallet("testwallet", None, None, None, None)?;
        }
        Ok(_) => println!("wallet exist"),
        Err(_) => {
            match rpc.create_wallet("testwallet", None, None, None, None) {
                Ok(_) => println!("create wallet"),
                Err(e) => println!("{}", e),
            }
        }
    }
    let fresh_unchecked = rpc.get_new_address(None, None)?;
    let fresh_addr = fresh_unchecked.require_network(Network::Regtest)?;
    rpc.generate_to_address(101, &fresh_addr)?;
    let analyzer = TxAnalyzer::new(Arc::clone(&rpc));
    // 지갑 확인
    let balance = rpc.get_balance(None, None)?;
    println!("current amount: {} BTC", balance.to_btc());

    // user1 -> taproot addr 
    let send_amount = Amount::from_sat(10_000); //10_000 sat
    println!("send amount : {} ", send_amount.to_sat());
    let txid1 = rpc.send_to_address(&tap_addr, send_amount, None, None, None, None, None, None)?;
    // finalize
    rpc.generate_to_address(1, &fresh_addr)?;
    let tx_info = rpc.get_transaction(&txid1, None)?;
    let tx = tx_info.transaction()?;
    // taproot output 확인
    let mut utxo_vout = None;
    let mut utxo_amount = None;

    for (vout, output) in tx.output.iter().enumerate() {
        if output.script_pubkey == tap_addr.script_pubkey() {
            utxo_vout = Some(vout as u32);
            utxo_amount = Some(output.value);
            println!("UTXO: vout = {}, amount = {} sat", vout, output.value.to_sat()); // vout = 0, amount = 1 sat
            break;
        }
    }

    let vout = utxo_vout.ok_or_else(|| anyhow::anyhow!("taproot out : None"))?;
    let prev_amount = utxo_amount.unwrap();
    let prevout = OutPoint { txid: txid1, vout };

    // taproot addr -> user2 send tx
    let dest_unchecked = rpc.get_new_address(None, None)?;
    let dest_addr = dest_unchecked.require_network(Network::Regtest)?;
    let fee_sat: u64 = 1_000;
    let spend_value_sat = prev_amount.to_sat().checked_sub(fee_sat)
        .ok_or_else(|| anyhow::anyhow!("UTXO fee prob"))?;

    let mut tx = Transaction {
        version: bitcoin::transaction::Version(2),
        lock_time: bitcoin::absolute::LockTime::ZERO,
        input: vec![TxIn {
            previous_output: prevout,
            sequence: Sequence(0xFFFFFFFF),
            witness: bitcoin::Witness::new(),
            script_sig: Default::default(),
        }],
        output: vec![TxOut {
            script_pubkey: dest_addr.script_pubkey(),
            value: Amount::from_sat(spend_value_sat),
        }],
    };

    // taproot sig 생성
    let prev_txout = TxOut {
        script_pubkey: tap_addr.script_pubkey(),
        value: prev_amount,
    };

    let leaf_hash = TapLeafHash::from_script(&leaf_script, LeafVersion::TapScript);
    let mut cache = SighashCache::new(&tx);
    let sighash_msg = cache.taproot_script_spend_signature_hash(
        0,
        &Prevouts::All(&[prev_txout.clone()]),
        leaf_hash,
        TapSighashType::Default,
    )?;

    let hash_bytes = sighash_msg.to_byte_array();
    let msg = Message::from_digest_slice(&hash_bytes)?;
    let sig = secp.sign_schnorr(&msg, &keypair);
    let sig_bytes = sig.as_ref();

    // Witness 구성 (OP_CAT + OP_CHECKSIG용)
    let r_part = &sig_bytes[0..32];
    let s_part = &sig_bytes[32..64];

    tx.input[0].witness.push(r_part.to_vec());
    tx.input[0].witness.push(s_part.to_vec());
    tx.input[0].witness.push(leaf_script.clone().into_bytes());
    let control_block = taproot_info
        .control_block(&(leaf_script.clone(), LeafVersion::TapScript))
        .expect("control block failed")
        .serialize();
    tx.input[0].witness.push(control_block);

    println!("OP_CAT + OP_CHECKSIG tx send");
    let txid2 = rpc.send_raw_transaction(&tx)?;
    println!("txid: {}", txid2);
    rpc.generate_to_address(1, &fresh_addr)?;

    let all_txids = vec![txid1, txid2];
    analyzer.full_analysis(&all_txids)?;

    Ok(())
}