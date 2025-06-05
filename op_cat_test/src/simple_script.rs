use bitcoin::opcodes::all::{OP_CAT, OP_CHECKSIG};
use bitcoin::script::{Builder, ScriptBuf};
use bitcoin::XOnlyPublicKey;

// 스택에 [R(32바이트), S(32바이트)]를 OP_CAT으로 합쳐 schnorr 서명 생성
// OP_CHECKSIG으로 X-only Schnorr 검증 수행
//
// Witness에는 차례대로
// 1) R (32바이트)
// 2) S (32바이트)
// 3) leaf 스크립트 바이트 (simple_cat_checksig이 반환하는 ScriptBuf)
// 4) control block (Taproot Merkle proof)

pub fn simple_cat_checksig(pubkey: &XOnlyPublicKey) -> ScriptBuf {
    Builder::new()
        .push_opcode(OP_CAT)
        .push_slice(&pubkey.serialize())
        .push_opcode(OP_CHECKSIG)
        .into_script()
}