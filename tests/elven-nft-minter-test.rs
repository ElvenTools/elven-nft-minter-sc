use elrond_wasm::{
    elrond_codec::multi_types::OptionalValue,
    types::{Address, BigUint, ManagedBuffer},
};
use elrond_wasm_debug::{rust_biguint, testing_framework::*, DebugApi};
use elven_nft_minter::*;

const WASM_PATH: &'static str = "output/elven-nft-minter.wasm";

struct ElvenNftMinterSetup<ElvenNftMinterObjBuilder>
where
    ElvenNftMinterObjBuilder: 'static + Copy + Fn() -> elven_nft_minter::ContractObj<DebugApi>,
{
    pub blockchain_wrapper: BlockchainStateWrapper,
    pub owner_address: Address,
    pub em_wrapper:
        ContractObjWrapper<elven_nft_minter::ContractObj<DebugApi>, ElvenNftMinterObjBuilder>,
}

fn setup_elven_nft_minter<ElvenNftMinterObjBuilder>(
    em_builder: ElvenNftMinterObjBuilder,
) -> ElvenNftMinterSetup<ElvenNftMinterObjBuilder>
where
    ElvenNftMinterObjBuilder: 'static + Copy + Fn() -> elven_nft_minter::ContractObj<DebugApi>,
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

    blockchain_wrapper
        .execute_tx(&owner_address, &em_wrapper, &rust_zero, |sc| {
            let image_base_cid = ManagedBuffer::<DebugApi>::from(b"imageIpfsCID");
            let metadata_base_cid = ManagedBuffer::<DebugApi>::from(b"metadataIpfsCID");
            let number_of_tokens: u32 = 10000;
            let tokens_limit_per_address: u32 = 3;
            let royalties = BigUint::from(1000 as u32);
            let selling_price = BigUint::from(1000000000000000000 as u64);
            let tags = OptionalValue::Some(ManagedBuffer::<DebugApi>::from(b"tags:tag1,tag2"));
            let provenance_hash =
                OptionalValue::Some(ManagedBuffer::<DebugApi>::from(b"provenanceHash"));
            let file_extension = OptionalValue::Some(ManagedBuffer::<DebugApi>::from(b".jpg"));
            let is_metadata_in_uris = OptionalValue::Some(true);

            sc.init(
                image_base_cid,
                metadata_base_cid,
                number_of_tokens,
                tokens_limit_per_address,
                royalties,
                selling_price,
                file_extension,
                tags,
                provenance_hash,
                is_metadata_in_uris,
            );
        })
        .assert_ok();

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
