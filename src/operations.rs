const NFT_AMOUNT: u32 = 1;
// This is the most popular gateway, but it doesn't matter the most important is IPFS CID
const IPFS_GATEWAY_HOST: &[u8] = "https://ipfs.io/ipfs/".as_bytes();
const METADATA_KEY_NAME: &[u8] = "metadata:".as_bytes();
const METADATA_FILE_EXTENSION: &[u8] = ".json".as_bytes();
const ATTR_SEPARATOR: &[u8] = ";".as_bytes();
const URI_SLASH: &[u8] = "/".as_bytes();
const TAGS_KEY_NAME: &[u8] = "tags:".as_bytes();

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use crate::storage;

#[multiversx_sc::module]

pub trait Operations: storage::Storage {
  // Main mint function - requires the payment sum for all tokens to mint.
  #[only_user_account]
  #[payable("EGLD")]
  #[endpoint(mint)]
  fn mint(&self, amount_of_tokens: u32) {
      let payment_amount = self.call_value().egld_value();
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

      let single_payment_amount = payment_amount.clone_value() / amount_of_tokens;

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

      let hash_buffer = self.crypto().sha256(&attributes);

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
          .direct_esdt(&receiver, &token, nonce, &BigUint::from(NFT_AMOUNT));

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
          let payment_token = &EgldOrEsdtTokenIdentifier::egld();

          let owner = self.blockchain().get_owner_address();
          self.send()
              .direct(&owner, &payment_token, payment_nonce, &payment_amount);
      }

      // Choose next index to mint here
      self.handle_next_index_setup(next_index_to_mint_tuple);
  }

  #[only_user_account]
  #[endpoint(shuffle)]
  fn shuffle(&self) {
      require!(!self.nft_token_id().is_empty(), "Token not issued!");
      let uid_mapper = self.tokens_left_to_mint();
      require!(
          !uid_mapper.is_empty(),
          "There is nothing to shuffle. Indexes not populated or there are no tokens to mint left!"
      );

      self.do_shuffle();
  }

  fn do_shuffle(&self) {
      let uid = self.tokens_left_to_mint();

      let uid_len = uid.len();
      let mut rand_source = RandomnessSource::new();

      let index = rand_source.next_usize_in_range(1, uid_len + 1);

      let choosen_item = uid.get(index);

      self.next_index_to_mint().set((index, choosen_item));
  }

  fn handle_next_index_setup(&self, minted_index_tuple: (usize, usize)) {
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
          let mut uid = self.tokens_left_to_mint();
          let _ = uid.swap_remove(minted_index_tuple.0);
          self.do_shuffle();
      }
  }

  fn build_uris_vec(&self, index_to_mint: usize) -> ManagedVec<ManagedBuffer> {
      let is_metadata_in_uris = self.is_metadata_in_uris().get();

      let mut uris = ManagedVec::new();

      let image_cid = self.image_base_cid().get();
      let metadata_cid = self.metadata_base_cid().get();
      let uri_slash = ManagedBuffer::new_from_bytes(URI_SLASH);
      let metadata_file_extension = ManagedBuffer::new_from_bytes(METADATA_FILE_EXTENSION);
      let image_file_extension = self.file_extension().get();
      let file_index = self.decimal_to_ascii(index_to_mint.try_into().unwrap());

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
  fn build_attributes_buffer(&self, index_to_mint: usize) -> ManagedBuffer {
      let metadata_key_name = ManagedBuffer::new_from_bytes(METADATA_KEY_NAME);
      let metadata_index_file = self.decimal_to_ascii(index_to_mint.try_into().unwrap());
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

  fn build_token_name_buffer(&self, index_to_mint: usize) -> ManagedBuffer {
      let mut full_token_name = ManagedBuffer::new();

      let token_name_from_storage = self.nft_token_name().get();

      let no_number_in_name = self.no_number_in_nft_name().get();

      full_token_name.append(&token_name_from_storage);

      if !no_number_in_name {
          let token_index = self.decimal_to_ascii(index_to_mint.try_into().unwrap());
          let hash_and_space_sign = ManagedBuffer::new_from_bytes(" #".as_bytes());

          full_token_name.append(&hash_and_space_sign);
          full_token_name.append(&token_index);
      }

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
}
