use elven_nft_minter::*;
use elrond_wasm::{
    types::{Address, SCResult, ManagedBuffer, OptionalArg, BigUint},
};
use elrond_wasm_debug::{
    rust_biguint, testing_framework::*,
    DebugApi,
};

use std::time::{SystemTime, UNIX_EPOCH};

const WASM_PATH: &'static str = "output/elven-nft-minter.wasm";
const ONE_WEEK: u64 = 7 * 24 * 60 * 60; // 1 week in seconds

struct ElvenNftMinterSetup<ElvenNftMinterObjBuilder>
where
    ElvenNftMinterObjBuilder: 'static + Copy + Fn(DebugApi) -> elven_nft_minter::ContractObj<DebugApi>,
{
    pub blockchain_wrapper: BlockchainStateWrapper,
    pub owner_address: Address,
    pub em_wrapper: ContractObjWrapper<elven_nft_minter::ContractObj<DebugApi>, ElvenNftMinterObjBuilder>,
}

fn setup_elven_nft_minter<ElvenNftMinterObjBuilder>(
    em_builder: ElvenNftMinterObjBuilder,
) -> ElvenNftMinterSetup<ElvenNftMinterObjBuilder>
where
    ElvenNftMinterObjBuilder:
        'static + Copy + Fn(DebugApi) -> elven_nft_minter::ContractObj<DebugApi>,
{
    let rust_zero = rust_biguint!(0u64);
    let mut blockchain_wrapper = BlockchainStateWrapper::new();
    let owner_address = blockchain_wrapper.create_user_account(&rust_zero);
    
    let em_wrapper = blockchain_wrapper.create_sc_account(
        &rust_zero,
        Some(&owner_address),
        em_builder,
        WASM_PATH,
    );

    blockchain_wrapper.execute_tx(&owner_address, &em_wrapper, &rust_zero, |sc| {
        let image_base_cid = ManagedBuffer::<DebugApi>::from(b"imageIpfsCID");
        let metadata_base_cid = ManagedBuffer::<DebugApi>::from(b"metadataIpfsCID");
        let number_of_tokens: u32 = 10000;
        let tokens_limit_per_address: u32 = 3;
        let start_timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let end_timestamp = &start_timestamp + ONE_WEEK;
        let royalties = BigUint::from(1000 as u32);
        let selling_price = BigUint::from(1000000000000000000 as u64);

        let tags = OptionalArg::Some(ManagedBuffer::<DebugApi>::from(b"tags:tag1,tag2"));
        let provenance_hash = OptionalArg::Some(ManagedBuffer::<DebugApi>::from(b"provenanceHash"));
        let file_extension = OptionalArg::Some(ManagedBuffer::<DebugApi>::from(b".jpg"));
        let result = sc.init(
          image_base_cid,
          metadata_base_cid,
          number_of_tokens,
          tokens_limit_per_address,
          start_timestamp,
          end_timestamp,
          royalties,
          selling_price,
          file_extension,
          tags,
          provenance_hash,
        );
        assert_eq!(result, SCResult::Ok(()));
        StateChange::Commit
    });

    blockchain_wrapper.add_mandos_set_account(em_wrapper.address_ref());

    ElvenNftMinterSetup {
        blockchain_wrapper,
        owner_address,
        em_wrapper,
    }
}

// //////////////////////////////////////////////////////////////

#[test]
fn init_test() {
    let em_setup = setup_elven_nft_minter(elven_nft_minter::contract_obj);
    em_setup
        .blockchain_wrapper
        .write_mandos_output("_generated_init.scen.json");
}

// TODO: just an initial state, write better tests
