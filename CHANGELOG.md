### [1.1.0](https://github.com/juliancwirko/elven-nft-minter-sc/releases/tag/v1.1.0) (2022-01-28)
- Added tokens limit per address per drop as an option - each drop will reset the counter. The drop limit is optional. If not set, the general limit will be used. When the drop is unset, the global limit will be used, and the address will still mint to the limit. The general limit is always more important than the drop limit. The address won't be able to mint if the general limit is over and the drop limit still allows the mint. Always rethink the logic of the general limit and the drop limit. You will be able to change the general limit at any time if needed.
- fixed important bug

### [1.0.0](https://github.com/juliancwirko/elven-nft-minter-sc/releases/tag/v1.0.0) (2022-01-24)
- first proper version of the Smart Contract
