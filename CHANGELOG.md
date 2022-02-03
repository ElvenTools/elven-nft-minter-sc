### [1.1.0](https://github.com/juliancwirko/elven-nft-minter-sc/releases/tag/v1.1.0) (2022-01-28)
- Added tokens limit per address per drop as an option - each drop will reset the counter. The drop limit is optional. If not set, the general limit will be used. The global limit will be used when the drop is unset, and the address will still mint to the limit. The general limit is always more important than the drop limit. The address won't mint if the general limit is over and the drop limit still allows the mint. Always rethink the logic of the general limit and the drop limit. You will be able to change the general limit at any time if needed.
- fixed critical bug regarding the shuffling mechanism - it was not optimized enough
  - rewrites for the shuffle, it is now based on VecMapper, RandomnessSource, and swap_remove
  - added endpoint for populating the VecMapper because the operation for very big collections doesn't fit into one transaction. The CLI tool handles that automatically. Otherwise, you would need to split your collection tokens count and call the `populateIndexes` endpoint with the max amount of 5k

### [1.0.0](https://github.com/juliancwirko/elven-nft-minter-sc/releases/tag/v1.0.0) (2022-01-24)
- first proper version of the Smart Contract
