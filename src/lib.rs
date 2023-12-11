#![no_std]

const ROYALTIES_MAX: u32 = 10_000;
const DEFAULT_IMG_FILE_EXTENSION: &[u8] = ".png".as_bytes();

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub mod storage;
pub mod setup;
pub mod operations;

#[multiversx_sc::contract]
pub trait ElvenTools: storage::Storage + setup::Setup + operations::Operations {
    #[allow_multiple_var_args]
    #[init]
    fn init(
        &self,
        image_base_cid: ManagedBuffer,
        metadata_base_cid: ManagedBuffer,
        amount_of_tokens: u32,
        tokens_limit_per_address: u32,
        royalties: BigUint,
        selling_price: BigUint,
        file_extension: OptionalValue<ManagedBuffer>,
        tags: OptionalValue<ManagedBuffer>,
        provenance_hash: OptionalValue<ManagedBuffer>,
        is_metadata_in_uris: OptionalValue<bool>,
    ) {
        require!(royalties <= ROYALTIES_MAX, "Royalties cannot exceed 100%!");
        require!(
            amount_of_tokens >= 1,
            "Amount of tokens to mint should be at least 1!"
        );
        require!(
            tokens_limit_per_address >= 1,
            "Tokens limit per address should be at least 1!"
        );

        self.image_base_cid().set_if_empty(&image_base_cid);
        self.metadata_base_cid().set_if_empty(&metadata_base_cid);
        self.amount_of_tokens_total()
            .set_if_empty(&amount_of_tokens);
        self.tokens_limit_per_address_total()
            .set_if_empty(&tokens_limit_per_address);
        self.provenance_hash()
            .set_if_empty(&provenance_hash.into_option().unwrap_or_default());
        self.royalties().set_if_empty(&royalties);
        self.selling_price().set_if_empty(&selling_price);
        self.tags()
            .set_if_empty(&tags.into_option().unwrap_or_default());
        self.file_extension().set_if_empty(
            &file_extension
                .into_option()
                .unwrap_or_else(|| ManagedBuffer::new_from_bytes(DEFAULT_IMG_FILE_EXTENSION)),
        );
        self.is_metadata_in_uris()
            .set_if_empty(&is_metadata_in_uris.into_option().unwrap_or_default());

        let paused = true;
        self.paused().set_if_empty(&paused);
    }
}
