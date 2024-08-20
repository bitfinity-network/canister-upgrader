use candid::Principal;
use ic_canister::{init, post_upgrade, Canister, MethodType, PreUpdate};
use upgrader_canister_did::UpgraderCanisterInitData;

use crate::state::UpgraderCanisterState;

thread_local! {
    pub static STATE: UpgraderCanisterState = UpgraderCanisterState::default();
}

#[derive(Canister, Clone)]
pub struct UpgraderCanister {
    #[id]
    principal: Principal,
}

impl PreUpdate for UpgraderCanister {
    fn pre_update(&self, _method_name: &str, _method_type: MethodType) {}
}

impl UpgraderCanister {
    #[post_upgrade]
    pub fn post_upgrade(&mut self) {}

    #[init]
    pub fn init(&mut self, _data: UpgraderCanisterInitData) {}
}
