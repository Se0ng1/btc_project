use anyhow::Result;
use bitcoin::{Transaction, Txid, Amount, ScriptBuf};
use bitcoincore_rpc::{Client, RpcApi};
use std::sync::Arc;

pub struct TxAnalyzer {
    rpc: Arc<Client>,
}

impl TxAnalyzer {
    pub fn new(rpc: Arc<Client>) -> Self {
        Self { rpc }
    }
    fn analyze_script_type(&self, script: &ScriptBuf) -> String {
        if script.is_p2pkh() {
            "P2PKH".to_string()
        } else if script.is_p2sh() {
            "P2SH".to_string()
        } else if script.is_witness_program() {
            if script.is_p2wpkh() {
                "P2WPKH".to_string()
            } else if script.is_p2wsh() {
                "P2WSH".to_string()
            } else if script.is_p2tr() {
                "P2TR (Taproot)".to_string()
            } else {
                "Witness Program".to_string()
            }
        } else {
            "Other".to_string()
        }
    }

    fn calculate_fee(&self, txid: &Txid) -> Result<Option<Amount>> {
        let tx_info = self.rpc.get_transaction(txid, None)?;
        let tx = tx_info.transaction()?;
        let mut input_total = 0u64;
        let mut output_total = 0u64;

        // input 계산
        for input in &tx.input {
            if let Ok(prev_tx_info) = self.rpc.get_transaction(&input.previous_output.txid, None) {
                if let Ok(prev_tx) = prev_tx_info.transaction() {
                    if let Some(prev_output) = prev_tx.output.get(input.previous_output.vout as usize) {
                        input_total += prev_output.value.to_sat();
                    }
                }
            }
        }

        // output 계산
        for output in &tx.output {
            output_total += output.value.to_sat();
        }

        if input_total > output_total {
            Ok(Some(Amount::from_sat(input_total - output_total)))
        } else {
            Ok(None)
        }
    }
    pub fn analyze_transaction(&self, txid: &Txid, name: &str) -> Result<()> {

        let tx_info = self.rpc.get_transaction(txid, None)?;
        let tx = tx_info.transaction()?;

        println!("txid: {}", txid);
        println!("confirmations: {}", tx_info.info.confirmations);
        if let Some(height) = tx_info.info.blockheight {
            println!("block: {}", height);
        }
        if let Ok(Some(fee)) = self.calculate_fee(txid) {
            println!("fee: {} sat", fee.to_sat());
        }
        println!();

        // 입력 분석
        println!("input ({}):", tx.input.len());
        for (i, input) in tx.input.iter().enumerate() {
            println!("prev txid : {}, prev vout : {}",input.previous_output.txid, input.previous_output.vout);

            // 이전 출력 값 조회
            if let Ok(prev_tx_info) = self.rpc.get_transaction(&input.previous_output.txid, None) {
                if let Ok(prev_tx) = prev_tx_info.transaction() {
                    if let Some(prev_output) = prev_tx.output.get(input.previous_output.vout as usize) {
                        println!("amount: {} sat", prev_output.value.to_sat());
                    }
                }
            }

            // Witness 정보
            if !input.witness.is_empty() {
                println!("witness: {}", input.witness.len());
                for (w_idx, witness_item) in input.witness.iter().enumerate() {
                    let description = match w_idx {
                        0 => "r part",
                        1 => "s part",
                        2 => "script",
                        3 => "control block",
                        _ => "Data",
                    };
                    println!("[{}] {}: {} bytes", w_idx, description, witness_item.len());
                }
            }
        }
        println!();

        // 출력 분석
        println!("output ({}):", tx.output.len());
        for (i, output) in tx.output.iter().enumerate() {
            println!("[{}] {} sat", i, output.value.to_sat());
            println!("Type: {}", self.analyze_script_type(&output.script_pubkey));
        }
        println!();

        Ok(())
    }

    /// 트랜잭션 관계 분석
    pub fn analyze_relationship(&self, funding_txid: &Txid, spending_txid: &Txid) -> Result<()> {
        println!("tx analyze");

        let funding_tx_info = self.rpc.get_transaction(funding_txid, None)?;
        let funding_tx = funding_tx_info.transaction()?;

        let spending_tx_info = self.rpc.get_transaction(spending_txid, None)?;
        let spending_tx = spending_tx_info.transaction()?;

        // 연결 관계 찾기
        for (input_idx, input) in spending_tx.input.iter().enumerate() {
            if input.previous_output.txid == *funding_txid {
                let vout = input.previous_output.vout as usize;
                if let Some(prev_output) = funding_tx.output.get(vout) {
                    let input_amount = prev_output.value.to_sat();
                    let total_output: u64 = spending_tx.output.iter().map(|o| o.value.to_sat()).sum();
                    let fee = input_amount - total_output;

                    println!("user1 -> taproot addr tx ({})", &funding_txid.to_string());
                    println!("- output[{}] : {} sat", vout, input_amount);
                    println!("        │         ");
                    println!("        ▼         ");
                    println!("taproot addr -> user2 tx ({})", &spending_txid.to_string());
                    println!("- input [{}] : {} sat", input_idx, input_amount);
                    println!("- output 총합 : {} sat", total_output);
                    println!("- fee : {} sat", fee);
                    println!();

                    // OP_CAT + OP_CHECKSIG
                    if input.witness.len() == 4 {
                        println!("OP_CAT + OP_CHECKSIG 실행:");
                        println!("r ({} bytes) + s ({} bytes)",
                            input.witness[0].len(), input.witness[1].len());
                    }
                }
            }
        }

        Ok(())
    }

    pub fn full_analysis(&self, txids: &[Txid]) -> Result<()> {
        let names = ["송금 트랜잭션", "소비 트랜잭션"];
        for (i, txid) in txids.iter().enumerate() {
            let name = names.get(i).unwrap_or(&"트랜잭션");
            self.analyze_transaction(txid, name)?;
            println!("{}", "=".repeat(60));
            println!();
        }
        if txids.len() == 2 {
            self.analyze_relationship(&txids[0], &txids[1])?;
        }

        Ok(())
    }
}