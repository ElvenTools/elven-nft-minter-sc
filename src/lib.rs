#![no_std]

extern crate alloc;

const NFT_AMOUNT: u32 = 1;
const ROYALTIES_MAX: u32 = 10_000;
const IPFS_GATEWAY_HOST: &[u8] = "https://ipfs.io/ipfs/".as_bytes();
const IPFS_SCHEME: &[u8] = "ipfs://".as_bytes();
const METADATA_KEY_NAME: &[u8] = "metadata:".as_bytes();
const METADATA_FILE_EXTENSION: &[u8] = ".json".as_bytes();
const ATTR_SEPARATOR: &[u8] = ";".as_bytes();
const URI_SLASH: &[u8] = "/".as_bytes();
const TAGS_KEY_NAME: &[u8] = "tags:".as_bytes();
const IMG_FILE_EXTENSION: &[u8] = ".png".as_bytes();

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

#[elrond_wasm::contract]
pub trait ElvenTools {
    #[init]
    fn init(
        &self,
        image_base_cid: ManagedBuffer,
        metadata_base_cid: ManagedBuffer,
        amount_of_tokens: u32,
        start_timestamp: u64,
        end_timestamp: u64,
        royalties: BigUint,
        selling_price: BigUint,
        #[var_args] tags: OptionalArg<ManagedBuffer>,
        #[var_args] provenance_hash: OptionalArg<ManagedBuffer>,
    ) -> SCResult<()> {
        require!(royalties <= ROYALTIES_MAX, "Royalties cannot exceed 100%!");
        require!(
            start_timestamp < end_timestamp,
            "Start timestamp should be before the end timestamp!"
        );
        require!(amount_of_tokens >= 1, "Amount of tokens to mint should be at least 1!");

        self.image_base_cid().set(&image_base_cid);
        self.metadata_base_cid().set(&metadata_base_cid);
        self.amount_of_tokens().set(&amount_of_tokens);
        self.provenance_hash()
            .set(&provenance_hash.into_option().unwrap_or_default());
        self.start_time().set(&start_timestamp);
        self.end_time().set(&end_timestamp);
        self.royalties().set(&royalties);
        self.selling_price().set(&selling_price);
        self.tags().set(&tags.into_option().unwrap_or_default());

        let range_end = amount_of_tokens + 1;
        let range = 1..range_end;
        let mut index_vec = ManagedVec::new();
        range.for_each(|value| {
            index_vec.push(value);
        });
        self.indexes_to_mint().set(&index_vec);

        // TODO: enable when shuffle is ready
        // self.shuffle_indexes();

        Ok(())
    }

    #[only_owner]
    #[payable("EGLD")]
    #[endpoint(issueToken)]
    fn issue_token(
        &self,
        #[payment] issue_cost: BigUint,
        token_name: ManagedBuffer,
        token_ticker: ManagedBuffer,
    ) -> SCResult<AsyncCall> {
        require!(self.nft_token_id().is_empty(), "Token already issued!");

        Ok(self
            .send()
            .esdt_system_sc_proxy()
            .issue_non_fungible(
                issue_cost,
                &token_name,
                &token_ticker,
                NonFungibleTokenProperties {
                    can_freeze: false,
                    can_wipe: false,
                    can_pause: false,
                    can_change_owner: false,
                    can_upgrade: false,
                    can_add_special_roles: true,
                },
            )
            .async_call()
            .with_callback(self.callbacks().issue_callback()))
    }

    #[only_owner]
    #[endpoint(setLocalRoles)]
    fn set_local_roles(&self) -> SCResult<AsyncCall> {
        require!(!self.nft_token_id().is_empty(), "Token not issued!");

        Ok(self
            .send()
            .esdt_system_sc_proxy()
            .set_special_roles(
                &self.blockchain().get_sc_address(),
                &self.nft_token_id().get(),
                (&[EsdtLocalRole::NftCreate][..]).into_iter().cloned(),
            )
            .async_call())
    }

    #[only_owner]
    #[endpoint(pauseMinting)]
    fn pause_minting(&self) -> SCResult<()> {
        self.paused().set(&true);

        Ok(())
    }

    #[only_owner]
    #[endpoint(resumeMinting)]
    fn resume_minting(&self) -> SCResult<()> {
        self.paused().clear();

        Ok(())
    }

    #[payable("EGLD")]
    #[endpoint(mintNft)]
    fn mint_nft(&self, #[payment_amount] payment_amount: BigUint) -> SCResult<()> {
        require!(self.paused().is_empty(), "The minting is paused!");
        require!(!self.nft_token_id().is_empty(), "Token not issued!");
        require!(
            self.blockchain().get_block_timestamp() >= self.start_time().get(),
            "The minting haven't started yet!"
        );
        require!(
            self.blockchain().get_block_timestamp() <= self.end_time().get(),
            "The minting is over!"
        );
        require!(self.tokens_left().unwrap() >= 1, "All tokens have been minted already!");

        let price_tag = self.selling_price().get();
        require!(payment_amount == price_tag, "Invalid amount as payment");

        let amount = &BigUint::from(NFT_AMOUNT);

        let token = self.nft_token_id().get();
        let token_name = self.nft_token_name().get();

        let royalties = self.royalties().get();

        let attributes = self.build_attributes_buffer();

        let attributes_hash = self
            .crypto()
            .sha256(&attributes.to_boxed_bytes().as_slice());
        let hash_buffer = ManagedBuffer::from(attributes_hash.as_bytes());

        let uris = self.build_uris_vec();

        let roles = self.blockchain().get_esdt_local_roles(&token);

        require!(
            roles.has_role(&EsdtLocalRole::NftCreate),
            "NFTCreate role not set!"
        );

        let nonce = self.send().esdt_nft_create(
            &token,
            &amount,
            &token_name,
            &royalties,
            &hash_buffer,
            &attributes,
            &uris,
        );

        self.remove_first_index_after_mint();

        let nft_token_id = self.nft_token_id().get();
        let caller = self.blockchain().get_caller();
        self.send().direct(
            &caller,
            &nft_token_id,
            nonce,
            &BigUint::from(NFT_AMOUNT),
            &[],
        );

        let payment_nonce: u64 = 0;
        let payment_token = &TokenIdentifier::egld();

        let owner = self.blockchain().get_owner_address();
        self.send()
            .direct(&owner, &payment_token, payment_nonce, &payment_amount, &[]);

        Ok(())
    }

    #[endpoint(shuffleIndex)]
    fn shuffle_index(&self) -> SCResult<()> {
        // TODO: enable when RandomnessSource is available
        // let mut indexes = self.indexes_to_mint().get();

        // let indexes_length = indexes.len();
        // let mut rand_source = RandomnessSource::<Self::Api>::new();
        // for i in 0..indexes_length {
        //     let rand_index = rand_source.next_u32_in_range(i, indexes_length);
        //     let first_item = indexes.get(i).unwrap();
        //     let second_item = indexes.get(rand_index).unwrap();

        //     indexes.set(i, &second_item);
        //     indexes.set(rand_index, &first_item);
        // }

        // self.indexes_to_mint().set(indexes);

        Ok(())
    }

    #[callback]
    fn issue_callback(&self, #[call_result] result: ManagedAsyncCallResult<TokenIdentifier>) {
        match result {
            ManagedAsyncCallResult::Ok(token_id) => {
                self.nft_token_id().set(&token_id);
            }
            ManagedAsyncCallResult::Err(_) => {
                let caller = self.blockchain().get_owner_address();
                let (returned_tokens, token_id) = self.call_value().payment_token_pair();
                if token_id.is_egld() && returned_tokens > 0 {
                    self.send()
                        .direct(&caller, &token_id, 0, &returned_tokens, &[]);
                }
            }
        }
    }

    fn build_uris_vec(&self) -> ManagedVec<ManagedBuffer> {
        use alloc::string::ToString;

        let indexes_to_mint = self.indexes_to_mint().get();
        let first_index_to_mint = indexes_to_mint.get(0).unwrap();
        let mut uris = ManagedVec::new();

        let cid = self.image_base_cid().get();
        let uri_slash = ManagedBuffer::new_from_bytes(URI_SLASH);
        let image_file_extension = ManagedBuffer::new_from_bytes(IMG_FILE_EXTENSION);
        let file_index = ManagedBuffer::from(first_index_to_mint.to_string().as_bytes());

        let mut img_ipfs_gateway_uri = ManagedBuffer::new_from_bytes(IPFS_GATEWAY_HOST);
        img_ipfs_gateway_uri.append(&cid);
        img_ipfs_gateway_uri.append(&uri_slash);
        img_ipfs_gateway_uri.append(&file_index);
        img_ipfs_gateway_uri.append(&image_file_extension);

        let mut img_ipfs_uri = ManagedBuffer::new_from_bytes(IPFS_SCHEME);
        img_ipfs_uri.append(&cid);
        img_ipfs_uri.append(&uri_slash);
        img_ipfs_uri.append(&file_index);
        img_ipfs_uri.append(&image_file_extension);

        uris.push(img_ipfs_gateway_uri);
        uris.push(img_ipfs_uri);

        uris
    }

    fn build_attributes_buffer(&self) -> ManagedBuffer {
        use alloc::string::ToString;

        let indexes_to_mint = self.indexes_to_mint().get();
        let first_index_to_mint = indexes_to_mint.get(0).unwrap();
        let metadata_key_name = ManagedBuffer::new_from_bytes(METADATA_KEY_NAME);
        let metadata_index_file =
            ManagedBuffer::new_from_bytes(first_index_to_mint.to_string().as_bytes());
        let metadata_file_extension = ManagedBuffer::new_from_bytes(METADATA_FILE_EXTENSION);
        let metadata_cid = self.metadata_base_cid().get();
        let separator = ManagedBuffer::new_from_bytes(ATTR_SEPARATOR);
        let metadata_slash = ManagedBuffer::new_from_bytes(URI_SLASH);
        let tags_key_name = ManagedBuffer::new_from_bytes(TAGS_KEY_NAME);

        let mut attributes = ManagedBuffer::new();
        attributes.append(&tags_key_name);
        attributes.append(&self.tags().get());
        attributes.append(&separator);
        attributes.append(&metadata_key_name);
        attributes.append(&metadata_cid);
        attributes.append(&metadata_slash);
        attributes.append(&metadata_index_file);
        attributes.append(&metadata_file_extension);

        attributes
    }

    fn remove_first_index_after_mint(&self) {
        let mut indexes_left = self.indexes_to_mint().get().into_vec();
        indexes_left.remove(0);
        self.indexes_to_mint().set(&ManagedVec::from(indexes_left));
    }

    #[view(tokensLeft)]
    fn tokens_left(&self) -> SCResult<usize> {
        let tokens_left = self.indexes_to_mint().get();

        Ok(tokens_left.len())
    }

    #[view(getNftTokenId)]
    #[storage_mapper("nftTokenId")]
    fn nft_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getNftTokenName)]
    #[storage_mapper("nftTokenName")]
    fn nft_token_name(&self) -> SingleValueMapper<ManagedBuffer>;

    #[view(getNftPrice)]
    #[storage_mapper("nftPrice")]
    fn selling_price(&self) -> SingleValueMapper<BigUint>;

    #[view(provenanceHash)]
    #[storage_mapper("provenanceHash")]
    fn provenance_hash(&self) -> SingleValueMapper<ManagedBuffer>;

    #[storage_mapper("iamgeBaseCid")]
    fn image_base_cid(&self) -> SingleValueMapper<ManagedBuffer>;

    #[storage_mapper("metadaBaseCid")]
    fn metadata_base_cid(&self) -> SingleValueMapper<ManagedBuffer>;

    #[storage_mapper("amountOfTokens")]
    fn amount_of_tokens(&self) -> SingleValueMapper<u32>;

    #[storage_mapper("startTime")]
    fn start_time(&self) -> SingleValueMapper<u64>;

    #[storage_mapper("endTime")]
    fn end_time(&self) -> SingleValueMapper<u64>;

    #[storage_mapper("indexesToMint")]
    fn indexes_to_mint(&self) -> SingleValueMapper<ManagedVec<u32>>;

    #[storage_mapper("royalties")]
    fn royalties(&self) -> SingleValueMapper<BigUint>;

    #[storage_mapper("paused")]
    fn paused(&self) -> SingleValueMapper<bool>;

    #[storage_mapper("tags")]
    fn tags(&self) -> SingleValueMapper<ManagedBuffer>;
}
