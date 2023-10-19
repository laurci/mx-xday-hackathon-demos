mxpy --verbose contract call erd1vdtu708ntm7328yvptpesrtw6jzj2dke5jahulw8yh469rs8mjdqsffef4 \
    --gas-limit 15000000 \
    --keyfile ~/robo_wallet.json \
    --recall-nonce --proxy https://gateway.multiversx.com --chain M \
    --function ESDTNFTCreate \
    --arguments str:ROBOTWARS-79fd78 100 str:RobotWars-Winner 0x0 0x0 0x0 str:https://reframed-test.s3.eu-central-1.amazonaws.com/nft/robot-wars.png \
    --send


