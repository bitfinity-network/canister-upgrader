use candid::{Encode, Principal};
use ic_exports::pocket_ic::PocketIc;
use upgrader_canister_did::UpgraderCanisterInitData;
use wasm_utils::get_upgrader_canister_bytecode;

pub mod wasm_utils;

/// Deploys the upgrader canister and returns its principal
pub async fn deploy_canister(env: &PocketIc) -> Principal {
    let wasm = get_upgrader_canister_bytecode();
    let init_data = UpgraderCanisterInitData {};
    let args = Encode!(&(init_data,)).unwrap();
    let canister = env.create_canister().await;
    env.add_cycles(canister, 10_u128.pow(12)).await;
    env.install_canister(canister, wasm.to_vec(), args, None)
        .await;
    canister
}
