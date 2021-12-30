#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

const NFT_AMOUNT: u32 = 1;
const ROYALTIES_MAX: u32 = 10_000;

#[elrond_wasm::contract]
pub trait ElvenTools {
    #[init]
    fn init(
        &self,
        image_base_cid: ManagedBuffer,
        metadata_base_cid: ManagedBuffer,
        number_of_tokens: u32,
        start_timestamp: u64,
        end_timestamp: u64,
        royalties: BigUint,
        selling_price: BigUint,
        #[var_args] provenance_hash: OptionalArg<ManagedBuffer>,
    ) -> SCResult<()> {
        require!(royalties <= ROYALTIES_MAX, "Royalties cannot exceed 100%");
        require!(start_timestamp < end_timestamp, "Start timestamp should be before the end timestamp");

        self.image_base_cid().set(&image_base_cid);
        self.metadata_base_cid().set(&metadata_base_cid);
        self.number_of_tokens().set(&number_of_tokens);
        self.provenance_hash().set(&provenance_hash.into_option().unwrap_or_default());
        self.start_time().set(&start_timestamp);
        self.end_time().set(&end_timestamp);
        self.royalties().set(&royalties);
        self.selling_price().set(&selling_price);

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
        require!(self.nft_token_id().is_empty(), "Token already issued");

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
        require!(!self.nft_token_id().is_empty(), "Token not issued");

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

    #[payable("EGLD")]
    #[endpoint(mintNft)]
    fn mint_nft(&self, #[payment_amount] payment_amount: BigUint) -> SCResult<()> {
        require!(!self.nft_token_id().is_empty(), "Token not issued");

        let price_tag = self.selling_price().get();
        require!(payment_amount == price_tag, "Invalid amount as payment");

        let amount = &BigUint::from(NFT_AMOUNT);

        let token = self.nft_token_id().get();
        let token_name = self.nft_token_name().get();

        let royalties = self.royalties().get();

        // TODO: prepare proper attributes
        let attributes = ManagedBuffer::new();

        // TODO: preapre proper hash from attributes
        let hash = ManagedBuffer::new();

        let mut uris = ManagedVec::new();
        // TODO: build proper uris here and for arguments (metadata CID path)
        uris.push(ManagedBuffer::new());

        let roles = self.blockchain().get_esdt_local_roles(&token);

        require!(
            roles.has_role(&EsdtLocalRole::NftCreate),
            "NFTCreate role not set"
        );

        let nonce = self.send().esdt_nft_create(
            &token,
            &amount,
            &token_name,
            &royalties,
            &hash,
            &attributes,
            &uris,
        );

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

    #[endpoint(shuffle)]
    fn shuffle(&self) -> SCResult<()> {
      // TODO:
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

    #[storage_mapper("numberOfTokens")]
    fn number_of_tokens(&self) -> SingleValueMapper<u32>;

    #[storage_mapper("startTime")]
    fn start_time(&self) -> SingleValueMapper<u64>;

    #[storage_mapper("endTime")]
    fn end_time(&self) -> SingleValueMapper<u64>;

    #[storage_mapper("alreadyMintedIndexes")]
    fn already_minted_indexes(&self, index: u32) -> SingleValueMapper<u32>;

    #[storage_mapper("royalties")]
    fn royalties(&self) -> SingleValueMapper<BigUint>;
}
