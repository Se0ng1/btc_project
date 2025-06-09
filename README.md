# BTC OP_CAT test

## Usage
```
./btc_node.sh
```

## Result
```
OP_CAT + OP_CHECKSIG test
 X-only Pubkey: 3c2b9d40b9f6999b32f2400420682949ece08085b0cef1b7dee8947923a74024
P2TR address: bcrt1pya9tueedf87y08h7jautrva2zq2ukwg7348glzpyfr8ztrl6upgsj73x78
create wallet
current amount: 50 BTC
send amount : 10000
UTXO: vout = 0, amount = 10000 sat
OP_CAT + OP_CHECKSIG tx send
txid: 42708648d33ba168ef332aa2f0a7ba3d6c0077d319d7431f45779ffc42fdeff8
txid: c234793ba77027a6f27a56fe7d9e19a0506ff47332e2553cfd48f87cce171cf1
confirmations: 2
block: 102
fee: 3300 sat

input (1):
prev txid : 8fc09d813345a02123a0f67531ef5c54690622a224209eccc6d5bb3a5df5dd36, prev vout : 0
amount: 5000000000 sat
witness: 2
[0] r part: 71 bytes
[1] s part: 33 bytes

output (2):
[0] 10000 sat
Type: P2TR (Taproot)
[1] 4999986700 sat
Type: P2TR (Taproot)

============================================================

txid: 42708648d33ba168ef332aa2f0a7ba3d6c0077d319d7431f45779ffc42fdeff8
confirmations: 1
block: 103
fee: 1000 sat

input (1):
prev txid : c234793ba77027a6f27a56fe7d9e19a0506ff47332e2553cfd48f87cce171cf1, prev vout : 0
amount: 10000 sat
witness: 4
[0] r part: 32 bytes
[1] s part: 32 bytes
[2] script: 35 bytes
[3] control block: 33 bytes

output (1):
[0] 9000 sat
Type: P2WPKH

============================================================

tx analyze
user1 -> taproot addr tx (c234793ba77027a6f27a56fe7d9e19a0506ff47332e2553cfd48f87cce171cf1)
- output[0] : 10000 sat
        │
        ▼
taproot addr -> user2 tx (42708648d33ba168ef332aa2f0a7ba3d6c0077d319d7431f45779ffc42fdeff8)
- input [0] : 10000 sat
- output 총합 : 9000 sat
- fee : 1000 sat

OP_CAT + OP_CHECKSIG 실행:
r (32 bytes) + s (32 bytes)
```
