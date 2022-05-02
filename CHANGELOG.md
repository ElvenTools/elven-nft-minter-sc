### [1.6.2](https://github.com/ElvenTools/elven-nft-minter-sc/releases/tag/v1.6.2) (2022-05-02)
- new endpoints:
  - `getTotalSupply` - total collection supply
  - `isMintingPaused` - check if minting is currently paused
  - `getTotalSupplyOfCurrentDrop` - total supply per current drop

### [1.6.1](https://github.com/ElvenTools/elven-nft-minter-sc/releases/tag/v1.6.1) (2022-04-30)
- new endpoint `clearAllowlist` - It will clear the whole allowlist. The best is to keep max 1300 addresses in the allowlist at a time. Of course, if only you plan to clear it later. If you keep more and want to clear it, you can reach the gas limit for a transaction. So it would be best to split the allowlist per drop, keep it as small as possible and clear it each time.
- new endpoint `removeAllowlistAddress` - removes singe address from allowlist

### [1.6.0](https://github.com/ElvenTools/elven-nft-minter-sc/releases/tag/v1.6.0) (2022-04-24)
- elrond-wasm upgraded to 0.30.0
- allow using a different name for NFT tokens. Till now, it was the name of the collection handler
- name change for the storage for the collection token name. `getCollectionTokenName` will now return collection name, and `getNftTokenName` will return the NFTs name if set.

### [1.5.2](https://github.com/ElvenTools/elven-nft-minter-sc/releases/tag/v1.5.2) (2022-03-05)
- elrond-wasm upgraded to 0.29.3
- cleanup for temporary sha256 functionality

### [1.5.1](https://github.com/ElvenTools/elven-nft-minter-sc/releases/tag/v1.5.1) (2022-02-27)
- bugfix related to not correctly handling the `unsetDrop` endpoint. The limits per address per drop were not correct when using `unsetDrop`.
- new query endpoint added `isDropActive`.

### [1.5.0](https://github.com/ElvenTools/elven-nft-minter-sc/releases/tag/v1.5.0) (2022-02-25)
- important performance fixes provided by Dorin (Elrond core dev)
- elrond-wasm upgraded to 0.28.0
- rewrite the URL creation to use static array instead of to_string
- removed SCResult (no longer needed, require stops execution immediately)
- using static buffers for hashing instead of the default legacy implementation (which uses dynamic allocation)

### [1.4.2](https://github.com/ElvenTools/elven-nft-minter-sc/releases/tag/v1.4.2) (2022-02-17)
- fixed bug related to the limits per address - u32 underflow in one case

### [1.4.1](https://github.com/ElvenTools/elven-nft-minter-sc/releases/tag/v1.4.1) (2022-02-14)
- fix populate indexes after changes in the Elrond's architecture, check out: [#34](https://github.com/ElvenTools/elven-nft-minter-sc/issues/34) for more details. It results in more transactions that have to be done. But for now, it is necessary.

### [1.4.0](https://github.com/ElvenTools/elven-nft-minter-sc/releases/tag/v1.4.0) (2022-02-13)
- allowlist functionality - when you enable it, only eligible addresses from the list can mint. You can add more addresses at any time. The amount of tokens per address is the same as usual. You can always change that per drop.
- replace all storage `set` to `set_if_empty` - preparation for the upgrade tests and CLI tooling 

### [1.3.1](https://github.com/ElvenTools/elven-nft-minter-sc/releases/tag/v1.3.0) (2022-02-09)
- Fixed the bug related to not optimal usage of storage clearing functions for managing the 'drops'.

### [1.3.0](https://github.com/ElvenTools/elven-nft-minter-sc/releases/tag/v1.3.0) (2022-02-06)
- the `amount_of_tokens` for the `mint` endpoint is now mandatory. It cleans up the code a little bit and makes it less prone to bugs. CLI requires the amount anyway, the same with the `giveaway` endpoint. So it is now unified.

### [1.2.0](https://github.com/ElvenTools/elven-nft-minter-sc/releases/tag/v1.2.0) (2022-02-04)
- Metadata JSON file can be now attached also in the Assets/Uris section (some marketplaces require that).

- There will be no `ipfs://` schema-based Uri from the Assets/Uris. It is because there are usually gateway Uris only. It is still possible to add the ipfs schema-based Uri to the metadata JSON file.
- By default, the CLI tool will use the Smart Contract from a specific compatible version tag, not from the main branch. The version of the Smart Contract will be shown in the package.json file.

### [1.1.1](https://github.com/ElvenTools/elven-nft-minter-sc/releases/tag/v1.1.1) (2022-02-03)
- Fixed the problematic bug related to how the public `shuffle` endpoint worked.
- Fixed the bug related to the `giveaway` and initial `shuffle`

### [1.1.0](https://github.com/ElvenTools/elven-nft-minter-sc/releases/tag/v1.1.0) (2022-01-28)
- Added tokens limit per address per drop as an option - each drop will reset the counter. The drop limit is optional. If not set, the general limit will be used. The global limit will be used when the drop is unset, and the address will still mint to the limit. The general limit is always more important than the drop limit. The address won't mint if the general limit is over and the drop limit still allows the mint. Always rethink the logic of the general limit and the drop limit. You will be able to change the general limit at any time if needed.
- fixed critical bug regarding the shuffling mechanism - it was not optimized enough
  - rewrites for the shuffle, it is now based on VecMapper, RandomnessSource, and swap_remove
  - added endpoint for populating the VecMapper because the operation for very big collections doesn't fit into one transaction. The CLI tool handles that automatically. Otherwise, you would need to split your collection tokens count and call the `populateIndexes` endpoint with the max amount of 5k

### [1.0.0](https://github.com/ElvenTools/elven-nft-minter-sc/releases/tag/v1.0.0) (2022-01-24)
- first proper version of the Smart Contract
