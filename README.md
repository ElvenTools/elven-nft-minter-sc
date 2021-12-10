### Example Smart Contract to be used with [elven-mint](https://github.com/juliancwirko/elven-mint)

**Please test it before using it for the real stuff. It can always be buggy. Not reviewed in any form and not tested in the mainnet! Still work in progress!**

Based on official [nft-minter](https://github.com/ElrondNetwork/elrond-wasm-rs/tree/master/contracts/examples/nft-minter) example.

### Pre requirements:

1. Installed the latest version of [erdpy](https://docs.elrond.com/sdk-and-tools/erdpy/installing-erdpy/)
2. Wallet pem file. How to derive it from seed phrases: [here](https://docs.elrond.com/sdk-and-tools/erdpy/deriving-the-wallet-pem-file/)

### Usage (devnet):

1. Clone the repo

2. `cd elven-nft-minter-sc`

3. Build the SC using `erdpy contract build`

4. Deploy the SC using (one level up from elven-nft-minter-sc directory):

```
erdpy --verbose contract deploy --chain="D" --project=elven-nft-minter-sc --pem="wallet.pem" --gas-limit=80000000 --proxy="https://devnet-gateway.elrond.com" --recall-nonce --send
```

You'll get back your smart contract address.

(remember to provide proper paths for --project and --pem file)

5. Issue collection ESDT token

```
erdpy --verbose contract call <smart_contract_address_here> --chain="D" --pem="wallet.pem" --gas-limit=60000000 --function="issueToken" --value=50000000000000000 --arguments 0x454c5557 0x454c5557 --proxy="https://devnet-gateway.elrond.com" --recall-nonce --send

```

arguments:
- token name in hex format (example: 0x + 454c5557 which is 0x + ELUW)
- token ticker in hex format (example: 0x + 454c5557 which is 0x + ELUW)

([elrond-converters](http://207.244.241.38/elrond-converters/))

In return, you will get a token identifier (in hex) that you need to use in the elven-minter tool.

6. Add special roles

```
erdpy --verbose contract call <smart_contract_address_here> --chain="D" --pem="wallet.pem" --gas-limit=60000000 --function="setLocalRoles" --proxy="https://devnet-gateway.elrond.com" --recall-nonce --send
```

7. Grab the Smart Contract address and token id and use [nft-art-maker](https://github.com/juliancwirko/nft-art-maker) or [elven-mint](https://github.com/juliancwirko/elven-mint) as helpers in mass minting.

### Learning resources

- [julian.io](https://www.julian.io/)
- [Youtube channel with walkthrough videos](https://www.youtube.com/channel/UCaj-mgcY9CWbLdZsC5Gt00g) (give me a sub! ;))
- (full path with video soon)
