#![no_std]

use core::convert::TryInto;

const NFT_AMOUNT: u32 = 1;
const ROYALTIES_MAX: u32 = 10_000;
const HASH_DATA_BUFFER_LEN: usize = 1024;
// This is the most popular gateway, but it doesn't matter the most important is IPFS CID
const IPFS_GATEWAY_HOST: &[u8] = "https://ipfs.io/ipfs/".as_bytes();
const METADATA_KEY_NAME: &[u8] = "metadata:".as_bytes();
const METADATA_FILE_EXTENSION: &[u8] = ".json".as_bytes();
const ATTR_SEPARATOR: &[u8] = ";".as_bytes();
const URI_SLASH: &[u8] = "/".as_bytes();
const TAGS_KEY_NAME: &[u8] = "tags:".as_bytes();
const DEFAULT_IMG_FILE_EXTENSION: &[u8] = ".png".as_bytes();

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
        tokens_limit_per_address: u32,
        royalties: BigUint,
        selling_price: BigUint,
        #[var_args] file_extension: OptionalValue<ManagedBuffer>,
        #[var_args] tags: OptionalValue<ManagedBuffer>,
        #[var_args] provenance_hash: OptionalValue<ManagedBuffer>,
        #[var_args] is_metadata_in_uris: OptionalValue<bool>,
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

    // Issue main collection token/handler
    #[only_owner]
    #[payable("EGLD")]
    #[endpoint(issueToken)]
    fn issue_token(
        &self,
        #[payment] issue_cost: BigUint,
        token_name: ManagedBuffer,
        token_ticker: ManagedBuffer,
    ) {
        require!(self.nft_token_id().is_empty(), "Token already issued!");

        self.nft_token_name().set(&token_name);

        self.send()
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
        #[var_args] tokens_limit_per_address_per_drop: OptionalValue<u32>,
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
    fn giveaway(&self, address: ManagedAddress, amount_of_tokens: u32) {
        require!(!self.nft_token_id().is_empty(), "Token not issued!");

        require!(
            self.initial_shuffle_triggered().get(),
            "Run the shuffle mechanism at least once!"
        );

        let token = self.nft_token_id().get();
        let roles = self.blockchain().get_esdt_local_roles(&token);

        require!(
            roles.has_role(&EsdtLocalRole::NftCreate),
            "NFTCreate role not set!"
        );

        require!(
            self.get_current_left_tokens_amount() >= amount_of_tokens,
            "All tokens have been minted already or the amount you want to mint is too much. Check limits! (totally or per drop)!"
        );

        for _ in 0..amount_of_tokens {
            self.mint_single_nft(BigUint::zero(), OptionalValue::Some(address.clone()));
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
                .get_sc_balance(&TokenIdentifier::egld(), 0),
            &[],
        );
    }

    #[only_owner]
    #[endpoint(populateIndexes)]
    fn populate_indexes(&self, amount: u32) {
        let initial_indexes_populate_done = self.initial_indexes_populate_done();

        require!(
            !initial_indexes_populate_done.get(),
            "The indexes are already properly populated!"
        );

        let amount_of_tokens = self.amount_of_tokens_total().get();
        let mut v_mapper = self.tokens_left_to_mint();
        let v_mapper_len = v_mapper.len() as u32;
        let total_amount = v_mapper_len + amount;

        require!(
            amount > 0 && total_amount <= amount_of_tokens,
            "Wrong amount of tokens!"
        );

        let from = v_mapper_len + 1;
        let to = from + amount - 1;
        for i in from..=to {
            v_mapper.push(&i);
        }

        if amount_of_tokens == total_amount {
            self.initial_indexes_populate_done().set(true);
        }
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

    // Main mint function - takes the payment sum for all tokens to mint.
    #[payable("EGLD")]
    #[endpoint(mint)]
    fn mint(&self, #[payment_amount] payment_amount: BigUint, amount_of_tokens: u32) {
        let caller = self.blockchain().get_caller();

        let is_allowlist_enabled = self.is_allowlist_enabled().get();
        if is_allowlist_enabled {
            require!(
                self.allowlist().contains(&caller),
                "The allowlist is enabled. Only eligible addresses can mint!"
            );
        }

        require!(
            amount_of_tokens > 0,
            "The number of tokens provided can't be less than 1!"
        );
        require!(!self.nft_token_id().is_empty(), "Token not issued!");

        let token = self.nft_token_id().get();
        let roles = self.blockchain().get_esdt_local_roles(&token);

        require!(
            roles.has_role(&EsdtLocalRole::NftCreate),
            "ESDTNFTCreate role not set!"
        );
        require!(
          self.initial_indexes_populate_done().get(),
          "The indexes are not properly populated! Double check the deployment process and populateIndexes endpoint calls."
        );
        require!(
            self.initial_shuffle_triggered().get(),
            "Run the shuffle mechanism at least once!"
        );
        require!(
            self.paused().is_empty(),
            "The minting is paused or haven't started yet!"
        );

        require!(
            self.get_current_left_tokens_amount() >= amount_of_tokens,
            "All tokens have been minted already or the amount you want to mint is to much. Check limits (totally or per drop). You have to fit in limits with the whole amount."
        );

        let minted_per_address = self.minted_per_address_total(&caller).get();
        let tokens_limit_per_address = self.tokens_limit_per_address_total().get();

        let tokens_left_to_mint: u32;

        if tokens_limit_per_address < minted_per_address {
            tokens_left_to_mint = 0;
        } else {
            tokens_left_to_mint = tokens_limit_per_address - minted_per_address;
        }

        require!(
            tokens_left_to_mint > 0 && tokens_left_to_mint >= amount_of_tokens,
            "You can't mint such an amount of tokens. Check the limits by one address!"
        );

        // Check if there is a drop set and the limits per address for the drop are set
        if self.is_drop_active().get() && !self.last_drop().is_empty() {
            let last_drop_id = self.last_drop().get();
            let minted_per_address_per_drop = self
                .minted_per_address_per_drop(last_drop_id)
                .get(&caller)
                .unwrap_or_default();
            let tokens_limit_per_address_per_drop = self.tokens_limit_per_address_per_drop().get();

            let tokens_left_to_mint_per_drop;

            if tokens_limit_per_address_per_drop < minted_per_address_per_drop {
                tokens_left_to_mint_per_drop = 0;
            } else {
                tokens_left_to_mint_per_drop =
                    tokens_limit_per_address_per_drop - minted_per_address_per_drop;
            }

            require!(
              tokens_left_to_mint_per_drop > 0 && tokens_left_to_mint_per_drop >= amount_of_tokens,
              "You can't mint such an amount of tokens. Check the limits by one address! You have to fit in limits with the whole amount."
            );
        }

        let single_payment_amount = payment_amount / amount_of_tokens;

        let price_tag = self.selling_price().get();
        require!(
            single_payment_amount == price_tag,
            "Invalid amount as payment"
        );

        for _ in 0..amount_of_tokens {
            self.mint_single_nft(single_payment_amount.clone(), OptionalValue::None);
        }
    }

    // Private single token mint function. It is also used for the giveaway.
    fn mint_single_nft(
        &self,
        payment_amount: BigUint,
        giveaway_address: OptionalValue<ManagedAddress>,
    ) {
        let next_index_to_mint_tuple = self.next_index_to_mint().get();

        let amount = &BigUint::from(NFT_AMOUNT);

        let token = self.nft_token_id().get();
        let token_name = self.build_token_name_buffer(next_index_to_mint_tuple.1);

        let royalties = self.royalties().get();

        let attributes = self.build_attributes_buffer(next_index_to_mint_tuple.1);
        
        let hash_buffer = self
            .crypto()
            .sha256_legacy_managed::<HASH_DATA_BUFFER_LEN>(&attributes);
          
        let attributes_hash = hash_buffer.as_managed_buffer();

        let uris = self.build_uris_vec(next_index_to_mint_tuple.1);

        let nonce = self.send().esdt_nft_create(
            &token,
            &amount,
            &token_name,
            &royalties,
            &attributes_hash,
            &attributes,
            &uris,
        );

        let giveaway_address = giveaway_address
            .into_option()
            .unwrap_or_else(|| ManagedAddress::zero());

        let caller = self.blockchain().get_caller();

        let receiver;

        if giveaway_address.is_zero() {
            receiver = &caller;
        } else {
            receiver = &giveaway_address;
        }

        self.send()
            .direct(&receiver, &token, nonce, &BigUint::from(NFT_AMOUNT), &[]);

        if payment_amount > 0 {
            self.minted_per_address_total(&caller)
                .update(|sum| *sum += 1);

            if self.is_drop_active().get() && !self.last_drop().is_empty() {
                let last_drop_id = self.last_drop().get();
                let existing_address_value = self
                    .minted_per_address_per_drop(last_drop_id)
                    .get(&caller)
                    .unwrap_or_default();
                if existing_address_value > 0 {
                    let next_value = existing_address_value + 1;
                    self.minted_per_address_per_drop(last_drop_id)
                        .insert(caller, next_value);
                } else {
                    self.minted_per_address_per_drop(last_drop_id)
                        .insert(caller, 1);
                }
            }

            let payment_nonce: u64 = 0;
            let payment_token = &TokenIdentifier::egld();

            let owner = self.blockchain().get_owner_address();
            self.send()
                .direct(&owner, &payment_token, payment_nonce, &payment_amount, &[]);
        }

        // Choose next index to mint here from shuffled Vec
        self.handle_next_index_setup(next_index_to_mint_tuple);
    }

    #[endpoint(shuffle)]
    fn shuffle(&self) {
        require!(!self.nft_token_id().is_empty(), "Token not issued!");
        let v_mapper = self.tokens_left_to_mint();
        require!(
            !v_mapper.is_empty(),
            "There is nothing to shuffle. Indexes not populated or there are no tokens to mint left!"
        );

        let initial_shuffle_triggered = self.initial_shuffle_triggered().get();

        if !initial_shuffle_triggered {
            self.initial_shuffle_triggered().set(true);
        }

        self.do_shuffle();
    }

    fn do_shuffle(&self) {
        let vec = self.tokens_left_to_mint();

        let vec_len = vec.len();
        let mut rand_source = RandomnessSource::<Self::Api>::new();

        let index = rand_source.next_usize_in_range(1, vec_len + 1);

        let choosen_item = vec.get(index);

        self.next_index_to_mint().set((index, choosen_item));
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

    fn initial_random_mint_index(&self) -> u32 {
        let total_tokens = self.amount_of_tokens_total().get();
        let mut rand_source = RandomnessSource::<Self::Api>::new();
        let rand_index = rand_source.next_u32_in_range(1, total_tokens + 1);

        rand_index
    }

    fn handle_next_index_setup(&self, minted_index_tuple: (usize, u32)) {
        let is_minted_indexes_total_empty = self.minted_indexes_total().is_empty();
        if is_minted_indexes_total_empty {
            self.minted_indexes_total().set(1);
        } else {
            self.minted_indexes_total().update(|sum| *sum += 1);
        }

        let drop_amount = self.amount_of_tokens_per_drop().get();
        if drop_amount > 0 {
            let is_minted_indexes_by_drop_empty = self.minted_indexes_by_drop().is_empty();
            if is_minted_indexes_by_drop_empty {
                self.minted_indexes_by_drop().set(1);
            } else {
                self.minted_indexes_by_drop().update(|sum| *sum += 1);
            }
        }

        let total_tokens_left = self.total_tokens_left();

        if total_tokens_left > 0 {
            let mut vec = self.tokens_left_to_mint();
            vec.swap_remove(minted_index_tuple.0);
            self.do_shuffle();
        }
    }

    fn build_uris_vec(&self, index_to_mint: u32) -> ManagedVec<ManagedBuffer> {
        let is_metadata_in_uris = self.is_metadata_in_uris().get();

        let mut uris = ManagedVec::new();

        let image_cid = self.image_base_cid().get();
        let metadata_cid = self.metadata_base_cid().get();
        let uri_slash = ManagedBuffer::new_from_bytes(URI_SLASH);
        let metadata_file_extension = ManagedBuffer::new_from_bytes(METADATA_FILE_EXTENSION);
        let image_file_extension = self.file_extension().get();
        let file_index = self.decimal_to_ascii(index_to_mint);

        let mut img_ipfs_gateway_uri = ManagedBuffer::new_from_bytes(IPFS_GATEWAY_HOST);
        img_ipfs_gateway_uri.append(&image_cid);
        img_ipfs_gateway_uri.append(&uri_slash);
        img_ipfs_gateway_uri.append(&file_index);
        img_ipfs_gateway_uri.append(&image_file_extension);

        uris.push(img_ipfs_gateway_uri);

        if is_metadata_in_uris {
            let mut ipfs_metadata_uri = ManagedBuffer::new_from_bytes(IPFS_GATEWAY_HOST);
            ipfs_metadata_uri.append(&metadata_cid);
            ipfs_metadata_uri.append(&uri_slash);
            ipfs_metadata_uri.append(&file_index);
            ipfs_metadata_uri.append(&metadata_file_extension);

            uris.push(ipfs_metadata_uri);
        }

        uris
    }

    // This can be probably optimized with attributes struct, had problems with decoding on the api side
    fn build_attributes_buffer(&self, index_to_mint: u32) -> ManagedBuffer {
        let metadata_key_name = ManagedBuffer::new_from_bytes(METADATA_KEY_NAME);
        let metadata_index_file = self.decimal_to_ascii(index_to_mint);
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

    fn build_token_name_buffer(&self, index_to_mint: u32) -> ManagedBuffer {
        let mut full_token_name = ManagedBuffer::new();
        let token_name_from_storage = self.nft_token_name().get();
        let token_index = self.decimal_to_ascii(index_to_mint);
        let hash_sign = ManagedBuffer::new_from_bytes(" #".as_bytes());

        full_token_name.append(&token_name_from_storage);
        full_token_name.append(&hash_sign);
        full_token_name.append(&token_index);

        full_token_name
    }

    fn decimal_to_ascii(&self, mut number: u32) -> ManagedBuffer {
        const MAX_NUMBER_CHARACTERS: usize = 10;
        const ZERO_ASCII: u8 = b'0';

        let mut as_ascii = [0u8; MAX_NUMBER_CHARACTERS];
        let mut nr_chars = 0;

        loop {
            unsafe {
                let reminder: u8 = (number % 10).try_into().unwrap_unchecked();
                number /= 10;

                as_ascii[nr_chars] = ZERO_ASCII + reminder;
                nr_chars += 1;
            }

            if number == 0 {
                break;
            }
        }

        let slice = &mut as_ascii[..nr_chars];
        slice.reverse();

        ManagedBuffer::new_from_bytes(slice)
    }

    fn get_current_left_tokens_amount(&self) -> u32 {
        let drop_amount = self.amount_of_tokens_per_drop().get();
        let tokens_left;
        let paused = true;
        if drop_amount > 0 {
            tokens_left = self.drop_tokens_left();
        } else {
            tokens_left = self.total_tokens_left();
        }

        if tokens_left == 0 {
            self.paused().set(&paused);
        }

        tokens_left
    }

    #[view(getDropTokensLeft)]
    fn drop_tokens_left(&self) -> u32 {
        let minted_tokens = self.minted_indexes_by_drop().get();
        let amount_of_tokens = self.amount_of_tokens_per_drop().get();
        let left_tokens: u32 = amount_of_tokens - minted_tokens as u32;

        left_tokens
    }

    #[view(getTotalTokensLeft)]
    fn total_tokens_left(&self) -> u32 {
        let minted_tokens = self.minted_indexes_total().get();
        let amount_of_tokens = self.amount_of_tokens_total().get();
        let left_tokens: u32 = amount_of_tokens - minted_tokens as u32;

        left_tokens
    }

    #[view(getMintedPerAddressPerDrop)]
    fn get_minted_per_address_per_drop(&self, address: ManagedAddress) -> u32 {
        let minted_per_address_per_drop: u32;
        if self.is_drop_active().get() && !self.last_drop().is_empty() {
            let last_drop_id = self.last_drop().get();
            minted_per_address_per_drop = self
                .minted_per_address_per_drop(last_drop_id)
                .get(&address)
                .unwrap_or_default();
        } else {
            minted_per_address_per_drop = 0;
        }

        minted_per_address_per_drop
    }

    #[view(getAllowlistAddressCheck)]
    fn allowlist_address_check(&self, address: ManagedAddress) -> bool {
        self.allowlist().contains(&address)
    }

    #[view(getAllowlistSize)]
    fn allowlist_size(&self) -> usize {
        self.allowlist().len()
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

    #[storage_mapper("amountOfTokensTotal")]
    fn amount_of_tokens_total(&self) -> SingleValueMapper<u32>;

    #[storage_mapper("mintedIndexesTotal")]
    fn minted_indexes_total(&self) -> SingleValueMapper<u32>;

    #[storage_mapper("mintedIndexesByDrop")]
    fn minted_indexes_by_drop(&self) -> SingleValueMapper<u32>;

    #[storage_mapper("royalties")]
    fn royalties(&self) -> SingleValueMapper<BigUint>;

    #[storage_mapper("paused")]
    fn paused(&self) -> SingleValueMapper<bool>;

    #[storage_mapper("tags")]
    fn tags(&self) -> SingleValueMapper<ManagedBuffer>;

    #[storage_mapper("amountOfTokensPerDrop")]
    fn amount_of_tokens_per_drop(&self) -> SingleValueMapper<u32>;

    #[storage_mapper("nextIndexToMint")]
    fn next_index_to_mint(&self) -> SingleValueMapper<(usize, u32)>;

    #[storage_mapper("tokensLeftToMint")]
    fn tokens_left_to_mint(&self) -> VecMapper<u32>;

    #[storage_mapper("initialShuffleTriggered")]
    fn initial_shuffle_triggered(&self) -> SingleValueMapper<bool>;

    #[storage_mapper("initialIndexesPopulateDone")]
    fn initial_indexes_populate_done(&self) -> SingleValueMapper<bool>;

    #[storage_mapper("isMetadataInUris")]
    fn is_metadata_in_uris(&self) -> SingleValueMapper<bool>;
}
