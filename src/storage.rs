multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait Storage {
    #[view(getNftTokenId)]
    #[storage_mapper("nftTokenId")]
    fn nft_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getCollectionTokenName)]
    #[storage_mapper("collectionTokenName")]
    fn collection_token_name(&self) -> SingleValueMapper<ManagedBuffer>;

    #[view(getNftTokenName)]
    #[storage_mapper("nftTokenName")]
    fn nft_token_name(&self) -> SingleValueMapper<ManagedBuffer>;

    #[view(getNftPrice)]
    #[storage_mapper("nftPrice")]
    fn selling_price(&self) -> SingleValueMapper<BigUint>;

    #[view(getProvenanceHash)]
    #[storage_mapper("provenanceHash")]
    fn provenance_hash(&self) -> SingleValueMapper<ManagedBuffer>;

    #[view(getTokensLimitPerAddressTotal)]
    #[storage_mapper("tokensLimitPerAddressTotal")]
    fn tokens_limit_per_address_total(&self) -> SingleValueMapper<u32>;

    #[view(getMintedPerAddressTotal)]
    #[storage_mapper("mintedPerAddressTotal")]
    fn minted_per_address_total(&self, address: &ManagedAddress) -> SingleValueMapper<u32>;

    #[view(getTokensLimitPerAddressPerDrop)]
    #[storage_mapper("tokensLimitPerAddressPerDrop")]
    fn tokens_limit_per_address_per_drop(&self) -> SingleValueMapper<u32>;

    #[view(isAllowlistEnabled)]
    #[storage_mapper("isAllowlistEnabled")]
    fn is_allowlist_enabled(&self) -> SingleValueMapper<bool>;

    #[view(isDropActive)]
    #[storage_mapper("isDropActive")]
    fn is_drop_active(&self) -> SingleValueMapper<bool>;

    #[view(getTotalSupply)]
    #[storage_mapper("amountOfTokensTotal")]
    fn amount_of_tokens_total(&self) -> SingleValueMapper<u32>;

    #[view(isMintingPaused)]
    #[storage_mapper("paused")]
    fn paused(&self) -> SingleValueMapper<bool>;

    #[view(getTotalSupplyOfCurrentDrop)]
    #[storage_mapper("amountOfTokensPerDrop")]
    fn amount_of_tokens_per_drop(&self) -> SingleValueMapper<u32>;

    #[storage_mapper("lastDrop")]
    fn last_drop(&self) -> SingleValueMapper<u16>;

    #[storage_mapper("allowlist")]
    fn allowlist(&self) -> SetMapper<ManagedAddress>;

    #[storage_mapper("mintedPerAddressPerDrop")]
    fn minted_per_address_per_drop(&self, id: u16) -> MapMapper<ManagedAddress, u32>;

    #[storage_mapper("imageBaseCid")]
    fn image_base_cid(&self) -> SingleValueMapper<ManagedBuffer>;

    #[storage_mapper("metadaBaseCid")]
    fn metadata_base_cid(&self) -> SingleValueMapper<ManagedBuffer>;

    #[storage_mapper("file_extension")]
    fn file_extension(&self) -> SingleValueMapper<ManagedBuffer>;

    #[storage_mapper("mintedIndexesTotal")]
    fn minted_indexes_total(&self) -> SingleValueMapper<u32>;

    #[storage_mapper("mintedIndexesByDrop")]
    fn minted_indexes_by_drop(&self) -> SingleValueMapper<u32>;

    #[storage_mapper("royalties")]
    fn royalties(&self) -> SingleValueMapper<BigUint>;

    #[storage_mapper("tags")]
    fn tags(&self) -> SingleValueMapper<ManagedBuffer>;

    #[storage_mapper("nextIndexToMint")]
    fn next_index_to_mint(&self) -> SingleValueMapper<(usize, usize)>;

    #[storage_mapper("tokensLeftToMint")]
    fn tokens_left_to_mint(&self) -> UniqueIdMapper<Self::Api>;

    #[storage_mapper("isMetadataInUris")]
    fn is_metadata_in_uris(&self) -> SingleValueMapper<bool>;

    #[storage_mapper("noNumberInNftName")]
    fn no_number_in_nft_name(&self) -> SingleValueMapper<bool>;
}
