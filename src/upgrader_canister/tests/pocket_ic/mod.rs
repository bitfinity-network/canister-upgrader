use std::sync::Arc;

use candid::Principal;
use ic_canister_client::PocketIcClient;
use ic_exports::pocket_ic::PocketIc;
use upgrader_canister_client::UpgraderCanisterClient;
use upgrader_canister_did::UpgraderCanisterInitData;
use wasm_utils::get_upgrader_canister_bytecode;

pub mod wasm_utils;

pub const ADMIN: Principal = Principal::from_slice(&[212u8; 29]);

/// Deploys the upgrader canister and returns its principal
pub async fn deploy_canister(env: Option<PocketIc>) -> (Arc<PocketIc>, Principal) {
    let env = if let Some(env) = env {
        env
    } else {
        ic_exports::pocket_ic::init_pocket_ic().await
    };
    let wasm = get_upgrader_canister_bytecode();
    let init_data = UpgraderCanisterInitData {
        admin: ADMIN,
    };
    let args = candid::encode_args((init_data,)).unwrap();
    let canister = env.create_canister().await;
    env.add_cycles(canister, 10_u128.pow(12)).await;
    env.install_canister(canister, wasm.to_vec(), args, None)
        .await;
    (Arc::new(env), canister)
}

/// Builds an upgrader canister client
pub fn build_client(
    pocket: Arc<PocketIc>,
    canister_principal: Principal,
    caller_principal: Principal,
) -> UpgraderCanisterClient<PocketIcClient> {
    let client = PocketIcClient::from_client(pocket, canister_principal, caller_principal);
    UpgraderCanisterClient::new(client)
}
