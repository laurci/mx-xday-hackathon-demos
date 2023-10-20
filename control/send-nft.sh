#!/bin/bash
mxpy --verbose contract call erd1vdtu708ntm7328yvptpesrtw6jzj2dke5jahulw8yh469rs8mjdqsffef4 \
    --gas-limit 15000000 \
    --pem ~/robot-wallet.pem \
    --recall-nonce --proxy https://gateway.multiversx.com --chain M \
    --function ESDTNFTTransfer \
    --arguments str:ROBOTWARS-79fd78 0x4 0x1 $1 \
    --send


