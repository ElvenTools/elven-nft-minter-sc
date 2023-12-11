multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use crate::operations;
use crate::storage;

#[derive(TypeAbi, TopEncode, TopDecode)]
pub enum NFTProperties {
    CanFreeze,
    CanWipe,
    CanPause,
    CanTransferCreateRole,
    CanChangeOwner,
    CanUpgrade,
    CanAddSpecialRoles,
}

#[multiversx_sc::module]
pub trait Setup: storage::Storage + operations::Operations {
    // Issue main collection token/handler
    #[allow_multiple_var_args]
    #[only_owner]
    #[payable("EGLD")]
    #[endpoint(issueToken)]
    fn issue_token(
        &self,
        collection_token_name: ManagedBuffer,
        collection_token_ticker: ManagedBuffer,
        is_not_number_in_name: bool,
        nft_token_name: ManagedBuffer,
        token_properties: OptionalValue<MultiValueEncoded<NFTProperties>>,
    ) {
        let issue_cost = self.call_value().egld_value();
        require!(self.nft_token_id().is_empty(), "Token already issued!");

        let mut nfts_name = nft_token_name;

        if nfts_name.is_empty() {
          nfts_name = collection_token_name.clone();
        }

        self.no_number_in_nft_name().set(is_not_number_in_name);
        self.nft_token_name().set(&nfts_name);
        self.collection_token_name().set(&collection_token_name);

        let mut properties = NonFungibleTokenProperties {
            can_freeze: false,
            can_wipe: false,
            can_pause: false,
            can_transfer_create_role: false,
            can_change_owner: false,
            can_upgrade: false,
            can_add_special_roles: true, // to proceed it is required anyway, so there is no sense to leave it false
        };

        let properties_option = token_properties.into_option();

        match properties_option {
            Some(value) => {
                for token_propery in value.into_iter() {
                    match token_propery {
                        NFTProperties::CanFreeze => properties.can_freeze = true,
                        NFTProperties::CanWipe => properties.can_wipe = true,
                        NFTProperties::CanPause => properties.can_pause = true,
                        NFTProperties::CanTransferCreateRole => {
                            properties.can_transfer_create_role = true
                        }
                        NFTProperties::CanChangeOwner => properties.can_change_owner = true,
                        NFTProperties::CanUpgrade => properties.can_upgrade = true,
                        NFTProperties::CanAddSpecialRoles => {
                            properties.can_add_special_roles = true
                        }
                    };
                }
            }
            None => {}
        }

        self.send()
            .esdt_system_sc_proxy()
            .issue_non_fungible(
                issue_cost.clone_value(),
                &collection_token_name,
                &collection_token_ticker,
                properties,
            )
            .async_call()
            .with_callback(self.callbacks().issue_callback())
            .call_and_exit();
    }

    #[only_owner]
    #[endpoint(setLocalRoles)]
    fn set_local_roles(&self) {
        require!(!self.nft_token_id().is_empty(), "Token not issued!");

        self.send()
            .esdt_system_sc_proxy()
            .set_special_roles(
                &self.blockchain().get_sc_address(),
                &self.nft_token_id().get(),
                (&[EsdtLocalRole::NftCreate][..]).into_iter().cloned(),
            )
            .async_call()
            .call_and_exit();
    }

    #[callback]
    fn issue_callback(
        &self,
        #[call_result] result: ManagedAsyncCallResult<EgldOrEsdtTokenIdentifier>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(token_id) => {
                let tokens_number = self.amount_of_tokens_total().get();
                self.nft_token_id().set(&token_id.unwrap_esdt());
                self.tokens_left_to_mint()
                    .set_initial_len(tokens_number.try_into().unwrap());
                self.shuffle();
            }
            ManagedAsyncCallResult::Err(_) => {
                let caller = self.blockchain().get_owner_address();
                let returned = self.call_value().egld_or_single_esdt();
                if returned.token_identifier.is_egld() && returned.amount > 0 {
                    self.send()
                        .direct(&caller, &returned.token_identifier, 0, &returned.amount);
                }
            }
        }
    }

    #[only_owner]
    #[endpoint(pauseMinting)]
    fn pause_minting(&self) {
        let paused = true;
        self.paused().set(&paused);
    }

    #[only_owner]
    #[endpoint(startMinting)]
    fn start_minting(&self) {
        require!(!self.nft_token_id().is_empty(), "Token not issued!");

        self.paused().clear();
    }

    #[only_owner]
    #[endpoint(setDrop)]
    fn set_drop(
        &self,
        amount_of_tokens_per_drop: u32,
        tokens_limit_per_address_per_drop: OptionalValue<u32>,
    ) {
        let total_tokens_left = self.total_tokens_left();

        require!(
            amount_of_tokens_per_drop <= total_tokens_left,
            "The number of tokens per drop can't be higher than the total amount of tokens left!"
        );

        let tokens_limit = tokens_limit_per_address_per_drop
            .into_option()
            .unwrap_or_default();
        let tokens_limit_total = self.tokens_limit_per_address_total().get();

        require!(tokens_limit <= tokens_limit_total, "The tokens limit per address per drop should be smaller or equal to the total limit of tokens per address!");

        if tokens_limit > 0 {
            self.tokens_limit_per_address_per_drop().set(tokens_limit);
        } else {
            self.tokens_limit_per_address_per_drop()
                .set(amount_of_tokens_per_drop);
        }

        self.minted_indexes_by_drop().clear();
        self.amount_of_tokens_per_drop()
            .set(&amount_of_tokens_per_drop);

        if self.last_drop().is_empty() {
            self.last_drop().set(1);
        } else {
            self.last_drop().update(|sum| *sum += 1);
        }

        self.is_drop_active().set(true);
    }

    #[only_owner]
    #[endpoint(unsetDrop)]
    fn unset_drop(&self) {
        self.minted_indexes_by_drop().clear();
        self.amount_of_tokens_per_drop().clear();
        self.tokens_limit_per_address_per_drop().clear();
        self.is_drop_active().set(false);
    }

    // The owner can change the price, for example, a new price for the next nft drop.
    #[only_owner]
    #[endpoint(setNewPrice)]
    fn set_new_price(&self, price: BigUint) {
        self.selling_price().set(&price);
    }

    // The owner can change CIDs only before any NFT is minted!
    #[only_owner]
    #[endpoint(changeBaseCids)]
    fn change_base_cids(&self, image_base_cid: ManagedBuffer, metadata_base_cid: ManagedBuffer) {
        require!(
            self.minted_indexes_total().is_empty(),
            "You can't change the CIDs. There are some tokens minted already!"
        );

        self.image_base_cid().set(&image_base_cid);
        self.metadata_base_cid().set(&metadata_base_cid);
    }

    #[only_owner]
    #[endpoint(setNewTokensLimitPerAddress)]
    fn set_new_tokens_limit_per_address(&self, limit: u32) {
        self.tokens_limit_per_address_total().set(limit);
    }

    // As an owner of the smart contract, you can send randomly minted NFTs to chosen addresses.
    #[only_owner]
    #[endpoint(giveaway)]
    fn giveaway(&self, addresses: ManagedVec<ManagedAddress>, amount_of_tokens_per_address: u32) {
        require!(!self.nft_token_id().is_empty(), "Token not issued!");

        let token = self.nft_token_id().get();
        let roles = self.blockchain().get_esdt_local_roles(&token);

        require!(
            roles.has_role(&EsdtLocalRole::NftCreate),
            "NFTCreate role not set!"
        );

        require!(
            self.get_current_left_tokens_amount() >= amount_of_tokens_per_address * addresses.len() as u32,
            "All tokens have been minted already or the amount you want to mint is too much. Check limits! (totally or per drop)!"
        );

        for address in addresses.into_iter() {
            for _ in 0..amount_of_tokens_per_address {
                self.mint_single_nft(BigUint::zero(), OptionalValue::Some(address.clone()));
            }
        }
    }

    // As an owner, claim Smart Contract balance - temporary solution for royalities, the SC has to be payable to be able to get royalties
    #[only_owner]
    #[endpoint(claimScFunds)]
    fn claim_sc_funds(&self) {
        self.send().direct_egld(
            &self.blockchain().get_caller(),
            &self
                .blockchain()
                .get_sc_balance(&EgldOrEsdtTokenIdentifier::egld(), 0),
        );
    }

    #[only_owner]
    #[endpoint(enableAllowlist)]
    fn enable_allowlist(&self) {
        self.is_allowlist_enabled().set(true);
    }

    #[only_owner]
    #[endpoint(disableAllowlist)]
    fn disable_allowlist(&self) {
        self.is_allowlist_enabled().set(false);
    }

    #[only_owner]
    #[endpoint(populateAllowlist)]
    fn populate_allowlist(&self, addresses: ManagedVec<ManagedAddress>) {
        self.allowlist().extend(&addresses);
    }

    #[only_owner]
    #[endpoint(clearAllowlist)]
    fn clear_allowlist(&self) {
        self.allowlist().clear();
    }

    #[only_owner]
    #[endpoint(removeAllowlistAddress)]
    fn remove_allowlist_address(&self, address: ManagedAddress) {
        self.allowlist().remove(&address);
    }
}
