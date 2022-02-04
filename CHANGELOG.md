### [1.2.0](https://github.com/juliancwirko/elven-nft-minter-sc/releases/tag/v1.2.0) (2022-02-04)
- Metadata JSON file can be now attached also in the Assets/Uris section (some marketplaces require that).
- There will be no `ipfs://` schema-based Uri from the Assets/Uris. It is because there are usually gateway Uris only. It is still possible to add the ipfs schema-based Uri to the metadata JSON file.
- By default, the CLI tool will use the Smart Contract from a specific compatible version tag, not from the main branch. The version of the Smart Contract will be shown in the package.json file.

### [1.1.1](https://github.com/juliancwirko/elven-nft-minter-sc/releases/tag/v1.1.1) (2022-02-03)
- Fixed the problematic bug related to how the public `shuffle` endpoint worked.
- Fixed the bug related to the `giveaway` and initial `shuffle`

### [1.1.0](https://github.com/juliancwirko/elven-nft-minter-sc/releases/tag/v1.1.0) (2022-01-28)
- Added tokens limit per address per drop as an option - each drop will reset the counter. The drop limit is optional. If not set, the general limit will be used. The global limit will be used when the drop is unset, and the address will still mint to the limit. The general limit is always more important than the drop limit. The address won't mint if the general limit is over and the drop limit still allows the mint. Always rethink the logic of the general limit and the drop limit. You will be able to change the general limit at any time if needed.
- fixed critical bug regarding the shuffling mechanism - it was not optimized enough
  - rewrites for the shuffle, it is now based on VecMapper, RandomnessSource, and swap_remove
  - added endpoint for populating the VecMapper because the operation for very big collections doesn't fit into one transaction. The CLI tool handles that automatically. Otherwise, you would need to split your collection tokens count and call the `populateIndexes` endpoint with the max amount of 5k

### [1.0.0](https://github.com/juliancwirko/elven-nft-minter-sc/releases/tag/v1.0.0) (2022-01-24)
- first proper version of the Smart Contract
