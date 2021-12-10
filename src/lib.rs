#![no_std]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

const NFT_AMOUNT: u32 = 1;
const ROYALTIES_MAX: u32 = 10_000;

#[elrond_wasm::contract]
pub trait ElvenTools {
    #[init]
    fn init(&self) {}

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

    #[only_owner]
    #[endpoint(createNft)]
    fn create_nft(
        &self,
        token: TokenIdentifier,
        name: ManagedBuffer,
        uri: ManagedBuffer,
        attributes: ManagedBuffer,
        hash: ManagedBuffer,
        royalties: BigUint,
        selling_price: BigUint,
        #[var_args] with_claim: OptionalArg<bool>,
    ) -> SCResult<u64> {
        require!(!self.nft_token_id().is_empty(), "Token not issued");
        require!(royalties <= ROYALTIES_MAX, "Royalties cannot exceed 100%");

        let amount = &BigUint::from(NFT_AMOUNT);

        let mut uris = ManagedVec::new();
        uris.push(uri);

        let roles = self.blockchain().get_esdt_local_roles(&token);

        require!(
            roles.has_role(&EsdtLocalRole::NftCreate),
            "NFTCreate role not set"
        );

        let nonce = self.send().esdt_nft_create(
            &token,
            &amount,
            &name,
            &royalties,
            &hash,
            &attributes,
            &uris,
        );

        if (with_claim.into_option().unwrap_or_default()) {
          self.send().direct(
            &self.blockchain().get_caller(),
            &token,
            nonce,
            &BigUint::from(NFT_AMOUNT),
            &[],
          );
        }

        self.nft_price(nonce).set(&selling_price);

        Ok(nonce)
    }

    #[payable("EGLD")]
    #[endpoint(buyNft)]
    fn buy_nft(&self, #[payment_amount] payment_amount: BigUint, nft_nonce: u64) -> SCResult<()> {
        require!(!self.nft_token_id().is_empty(), "Token not issued");

        require!(
            !self.nft_price(nft_nonce).is_empty(),
            "Invalid nonce or NFT was already sold"
        );

        let price_tag = self.nft_price(nft_nonce).get();
        require!(payment_amount == price_tag, "Invalid amount as payment");

        self.nft_price(nft_nonce).clear();

        let nft_token_id = self.nft_token_id().get();
        let caller = self.blockchain().get_caller();
        self.send().direct(
            &caller,
            &nft_token_id,
            nft_nonce,
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

    #[view(getNftPrice)]
    #[storage_mapper("nftPrice")]
    fn nft_price(&self, nft_nonce: u64) -> SingleValueMapper<BigUint>;
}
