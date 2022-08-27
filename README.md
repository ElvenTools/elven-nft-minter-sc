### NFT minter Smart Contract 

- Docs: [www.elven.tools](https://www.elven.tools)
- Elven Tools Twitter: [www.twitter.com/ElvenTools](https://twitter.com/ElvenTools)
- Quick jumpstart: [www.elven.tools/docs/jump-start.html](https://www.elven.tools/docs/jump-start.html)
- Intro video: [www.youtu.be/Jou5jn8PFz8](https://youtu.be/Jou5jn8PFz8)

Be aware that the Smart Contract doesn't have any audits. It has complete functionality for the first version, but it still needs some improvements. Test it first on the devnet/testnet.

**You can use [elven-tools-cli](https://github.com/ElvenTools/elven-tools-cli) for deployment, setup and interactions!**

**You can use [elven-tools-dapp](https://github.com/ElvenTools/elven-tools-dapp) as your frontend dapp for minting process! (NextJS based app with 4 auth providers)**

### What is it?

You are reading about the Smart Contract designed for the Elrond blockchain. Its primary purpose is to provide a simple logic for minting and buying a previously configured collection of NFTs. It does it in a randomized way. Version 1 of it supports:

- issuing the collection token
- setting the proper roles
- pausing/unpausing the process
- random mint and distribution
- minting multiple NFTs in one transaction
- giveaway options + multiple addresses distribution in one transaction
- possibility to split the process into drops (waves/batches). It is named 'drop'
- configuring the allowlist, populate from a file or provide by hand
- claiming the developer rewards
- changing basic setup where it is possible
- and more...

Start here: [Elven Tools Jumpstart](https://www.elven.tools/docs/jump-start.html)

Check the [abi](https://github.com/ElvenTools/elven-nft-minter-sc/blob/main/output/elven-nft-minter.abi.json) file for more information.

Also, check how simple it is to deploy and interact with it using [elven-tools-cli](https://github.com/ElvenTools/elven-tools-cli).

### Check out possible workflows

Examples of how you can configure your Smart Contract in a couple of scenarios and how to use the CLI to do this faster and more efficient: [www.elven.tools/docs/elven-tools-workflows.html](https://www.elven.tools/docs/elven-tools-workflows.html)

### All endpoints with short descriptions

For all commands, check out the docs: [www.elven.tools/docs/sc-endpoints.html](https://www.elven.tools/docs/sc-endpoints.html)

### Other ways of using it

You can always clone it and change it as you need. The best is to use the Elven Tools CLI tool, which can also be configured after changes. But nothing stops you from using the [erdpy](https://docs.elrond.com/sdk-and-tools/erdpy/erdpy/). It is all up to you. Of course you will need to do more work when using the erdpy.

### Tracking the progress

- [Elven Tools Smart Contract kanban](https://github.com/orgs/ElvenTools/projects/4)

### Contact

- [Telegram](https://t.me/juliancwirko)
- [Twitter](https://twitter.com/JulianCwirko)

### You may also like

- [elven.js](https://github.com/juliancwirko/elven.js) - simplified wrapper over erdjs, designed to work as a plug-n-play solution for browser based use cases. No build steps and frameworks, just one file to rule it all! Check usage examples!
- [NFT Art Maker](https://github.com/juliancwirko/nft-art-maker) - generates images and metadata files and packs them into CAR files, all from provided PNG layers.
- [Buildo Begins](https://github.com/ElrondDevGuild/buildo-begins) - CLI toolset for interacting with the Elrond blockchain, APIs and smart contracts
- [Export collection owners to CSV](https://github.com/ElvenTools/elven-tools-collection-owners-csv)

### Issues and ideas

Please post issues and ideas [here](https://github.com/ElvenTools/elven-nft-minter-sc/issues).

### License

MIT + GPLv3 (Elrond tooling)
