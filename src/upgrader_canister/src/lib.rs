use ic_canister::generate_idl;

pub mod build_data;
pub mod canister;
pub mod constant;
pub mod inspect_message;
pub mod state;

pub fn idl() -> String {
    use std::collections::BTreeMap;

    use candid::Principal;
    use ic_canister::Idl;
    use upgrader_canister_did::*;

    let canister_idl = generate_idl!();

    candid::pretty::candid::compile(&canister_idl.env.env, &Some(canister_idl.actor))
}
