### NFT minter Smart Contract 

- Docs: [www.elven.tools](https://www.elven.tools)
- Intro video: [youtu.be/szkNE_qOy6Q](https://youtu.be/szkNE_qOy6Q)

🚨 Not enough tests! As for the mainnet, use it at your own risk! 🚨

**You can use [elven-tools-cli](https://github.com/juliancwirko/elven-tools-cli) for deployment and interactions!**

### What is it?

You are reading about the Smart Contract designed for the Elrond blockchain. Its primary purpose is to provide a simple logic for minting and buying a previously configured collection of NFTs. It does it in a randomized way. Version 1 of it supports:

- issuing the collection token
- setting the create role
- pausing/unpausing the process
- random mint and distribution
- minting multiple NFTs in one transaction
- giveaway options
- possibility to split the process into drops/waves
- claiming the developer rewards
- changing basic setup where it is possible

Check the [abi](https://github.com/juliancwirko/elven-nft-minter-sc/blob/main/output/elven-nft-minter.abi.json) file for more information.

Also, check how simple it is to deploy and interact with it using [elven-tools-cli](https://github.com/juliancwirko/elven-tools-cli).

### Other ways of using it

You can always clone it and change it as you need. The best is to use the Elven Tools CLI tool, which can also be configured after changes here. But nothing stops you from using the [erdpy](https://docs.elrond.com/sdk-and-tools/erdpy/erdpy/) and interacting with this Smart Contract. It is all up to you.

### Limitations and caveats

- Remember that it is most likely that because of the open-source nature of this Smart Contract, it won't be used only in a way that everyone would want to, be aware that you can always change the names of the endpoints in the Smart Contract. You can even deploy a couple of them. In the last minutes before the mint decide to use one of them. This will limit the bots. Remember always to inform which one is the official one.
- Smart Contract in version 1 doesn't have many mechanisms which will strongly limit unwanted behaviors. It only implements random minting, but in version 2, there will be more mechanisms for fair launches.

#### TODO:
- check [issues](https://github.com/juliancwirko/elven-nft-minter-sc/issues)

**Rust devs, I would appreciate it if you could leave your insights ❤️**

### Contact

- [Telegram](https://t.me/juliancwirko)
- [Twitter](https://twitter.com/JulianCwirko)

### Issues and ideas

Please post issues and ideas [here](https://github.com/juliancwirko/elven-nft-minter-sc/issues).

### License

MIT + GPLv3 (Elrond tooling)
