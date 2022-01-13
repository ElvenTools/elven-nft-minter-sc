### Example Smart Contract [WIP]

**Rust devs, I would appreciate it if you could leave your insights ❤️**

### Pre requirements:

1. Installed the latest version of [erdpy](https://docs.elrond.com/sdk-and-tools/erdpy/installing-erdpy/)
2. Wallet pem file. How to derive it from seed phrases: [here](https://docs.elrond.com/sdk-and-tools/erdpy/deriving-the-wallet-pem-file/)

### Usage (devnet):

**You can also use [elven-tools-cli](https://github.com/juliancwirko/elven-tools-cli) for that!**

1. Clone the repo

2. `cd elven-nft-minter-sc`

3. Build the SC using `erdpy contract build`

TODO: next steps and description

### Learning resources

- [julian.io](https://www.julian.io/)


#### TODO:

- argument for file extension when deploying
- shuffle mechanism (RandomnessSource)
- handle royalties (claim), dev rewards (payable, claim)
- tests
- data checks 
- comments
- proper handling of errors - return funds if applicable
- additional nft roles management (optional with default set)
- perf rewrites (help needed)

#### TODO (for later)
- more advanced functionality in future iterations (bidding, clearing, drops)
- rethink usage of ESDTRoleNFTUpdateAttributes and ESDTRoleNFTAddURI
