// required by the inspect_message macro
#[allow(unused_imports)]
use ic_exports::ic_cdk::{self, api};
use ic_exports::ic_kit::ic;

use crate::state::UpgraderCanisterState;

/// NOTE: inspect is disabled for non-wasm targets because without it we are getting a weird compilation error
/// in CI:
/// > multiple definition of `canister_inspect_message'
#[cfg(target_family = "wasm")]
#[ic_exports::ic_cdk_macros::inspect_message]
fn inspect_messages() {
    crate::canister::STATE.with(|state| inspect_message_impl(state))
}

#[allow(dead_code)]
fn inspect_message_impl(state: &UpgraderCanisterState) {
    // If inspect message is disabled, accept the message
    if state.settings.borrow().is_inspect_message_disabled() {
        api::call::accept_message();
        return;
    }

    let permissions = state.permissions.borrow();
    let method = api::call::method_name();

    let check_result = match method.as_str() {
        method if method.starts_with("admin_") => permissions.check_admin(&ic::caller()),
        "project_create" => crate::canister::project_create_inspect(&permissions, &ic::caller()),
        "poll_create" => crate::canister::poll_create_inspect(&permissions, &ic::caller()),
        "poll_vote" => crate::canister::poll_vote_inspect(&permissions, &ic::caller()),
        _ => Ok(()),
    };

    if let Err(e) = check_result {
        ic::trap(&format!("Call rejected by inspect check: {e:?}"));
    } else {
        api::call::accept_message();
    }
}
