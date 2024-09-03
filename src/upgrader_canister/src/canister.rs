use std::collections::BTreeMap;

use candid::Principal;
use ic_exports::ic_cdk::{init, query, post_upgrade, update};
use ic_exports::ic_kit::ic;
use ic_stable_structures::stable_structures::Memory;
use log::info;
use upgrader_canister_did::error::Result;
use upgrader_canister_did::{
    BuildData, ClosedPoll, PendingPoll, Permission, PermissionList, Poll, PollCreateData, PollType,
    ProjectData, UpgraderCanisterInitData, UpgraderError,
};

use crate::constant::POLL_TIMER_INTERVAL;
use crate::state::permission::Permissions;
use crate::state::UpgraderCanisterState;

thread_local! {
    pub static STATE: UpgraderCanisterState = UpgraderCanisterState::default();
}

#[post_upgrade]
pub fn post_upgrade() {
    set_timers();
}

#[init]
pub fn init(data: UpgraderCanisterInitData) {
    STATE.with(|state| {
        let mut permissions = state.permissions.borrow_mut();
        permissions
            .add_permissions(data.admin, vec![Permission::Admin])
            .expect("failed to add admin permission");
    });    
    set_timers();
}

/// Initializes the timers
fn set_timers() {
    // This block of code only need to be run in the wasm environment
    if cfg!(target_family = "wasm") {
        use ic_exports::ic_cdk_timers::set_timer_interval;

        set_timer_interval(POLL_TIMER_INTERVAL, move || {
            STATE.with(|state| {
                let mut permissions = state.permissions.borrow_mut();
                state
                    .polls
                    .borrow_mut()
                    .finalize_polls(time_secs(), &mut permissions)
                    .expect("Finalize polls error");
            });
        });
    } else {
        info!("Not setting timers as not in wasm environment");
    }
}

/// Returns the build data of the canister
#[query]
pub fn canister_build_data() -> BuildData {
    crate::build_data::canister_build_data()
}

/// Returns the permissions of a principal
#[query]
pub fn admin_permissions_get(principal: Principal) -> Result<PermissionList> {
    STATE.with(|state| {
        let permissions = state.permissions.borrow();
        permissions.check_admin(&ic::caller())?;
        Ok(permissions.get_permissions(&principal))
    })
}

/// Adds permissions to a principal and returns the principal permissions
#[update]
pub fn admin_permissions_add(
    principal: Principal,
    permissions: Vec<Permission>,
) -> Result<PermissionList> {
    STATE.with(|state| {
        state.permissions.borrow().check_admin(&ic::caller())?;
        state
            .permissions
            .borrow_mut()
            .add_permissions(principal, permissions)
    })
}

/// Removes permissions from a principal and returns the principal permissions
#[update]
pub fn admin_permissions_remove(
    principal: Principal,
    permissions: Vec<Permission>,
) -> Result<PermissionList> {
    STATE.with(|state| {
        state.permissions.borrow().check_admin(&ic::caller())?;
        state
            .permissions
            .borrow_mut()
            .remove_permissions(principal, &permissions)
    })
}

/// Disable/Enable the inspect message
#[update]
pub fn admin_disable_inspect_message(value: bool) -> Result<()> {
    STATE.with(|state| {
        state.permissions.borrow().check_admin(&ic::caller())?;
        state.settings.borrow_mut().disable_inspect_message(value);
        Ok(())
    })
}

/// Returns whether the inspect message is disabled.
#[query]
pub fn is_inspect_message_disabled() -> bool {
    STATE.with(|state| state.settings.borrow().is_inspect_message_disabled())
}

/// Returns the permissions of the caller
#[query]
pub fn caller_permissions_get() -> Result<PermissionList> {
    STATE.with(|state| {
        let permissions = state.permissions.borrow();
        Ok(permissions.get_permissions(&ic::caller()))
    })
}

/// Returns all projects
#[query]
pub fn project_get_all() -> Vec<ProjectData> {
    STATE.with(|state| state.projects.borrow().all())
}

/// Returns a project by key
#[query]
pub fn project_get(key: String) -> Option<ProjectData> {
    STATE.with(|state| state.projects.borrow().get(&key))
}

/// Inspects permissions for the project_create method
pub fn project_create_inspect<M: Memory>(
    permissions: &Permissions<M>,
    caller: &Principal,
) -> Result<()> {
    permissions.check_has_all_permissions(caller, &[Permission::CreateProject])
}

/// Creates a new project
#[update]
pub fn project_create(project: ProjectData) -> Result<()> {
    STATE.with(|state| {
        project_create_inspect(&state.permissions.borrow(), &ic::caller())?;
        state.projects.borrow_mut().insert(project)
    })
}

/// Returns all pending polls
#[query]
pub fn poll_get_all_pending() -> BTreeMap<u64, PendingPoll> {
    STATE.with(|state| state.polls.borrow().all_pending())
}

/// Returns all pending polls
#[query]
pub fn poll_get_all_closed() -> BTreeMap<u64, ClosedPoll> {
    STATE.with(|state| state.polls.borrow().all_closed())
}

/// Returns a poll by id
#[query]
pub fn poll_get(id: u64) -> Option<Poll> {
    STATE.with(|state| state.polls.borrow().get(&id))
}

/// Returns a poll by id searching in the pending polls
#[query]
pub fn poll_get_pending(id: u64) -> Option<PendingPoll> {
    STATE.with(|state| state.polls.borrow().get_pending(&id))
}

/// Returns a poll by id searching in the closed polls
#[query]
pub fn poll_get_closed(id: u64) -> Option<ClosedPoll> {
    STATE.with(|state| state.polls.borrow().get_closed(&id))
}

/// Inspects permissions for the poll_create method
pub fn poll_create_inspect<M: Memory>(
    permissions: &Permissions<M>,
    caller: &Principal,
) -> Result<()> {
    permissions.check_has_all_permissions(caller, &[Permission::CreatePoll])
}

/// Creates a new poll and returns the generated poll id
#[update]
pub fn poll_create(poll: PollCreateData) -> Result<u64> {
    STATE.with(|state| {
        poll_create_inspect(&state.permissions.borrow(), &ic::caller())?;

        if let PollType::ProjectHash { project, hash: _ } = &poll.poll_type {
            state.projects.borrow().get(project).ok_or_else(|| {
                UpgraderError::BadRequest(format!(
                    "Cannot create poll, project [{}] does not exist",
                    project
                ))
            })?;
        }

        Ok(state.polls.borrow_mut().insert(poll))
    })
}

/// Inspects permissions for the poll_vote method
pub fn poll_vote_inspect<M: Memory>(
    permissions: &Permissions<M>,
    caller: &Principal,
) -> Result<()> {
    permissions.check_has_all_permissions(caller, &[Permission::VotePoll])
}

/// Votes for a poll. If the voter has already voted, the previous vote is replaced.
#[update]
pub fn poll_vote(poll_id: u64, approved: bool) -> Result<()> {
    STATE.with(|state| {
        let caller = ic::caller();
        poll_vote_inspect(&state.permissions.borrow(), &caller)?;
        state
            .polls
            .borrow_mut()
            .vote(poll_id, caller, approved, time_secs())
    })
}

/// returns the timestamp in seconds
#[inline]
pub fn time_secs() -> u64 {
    #[cfg(not(target_family = "wasm"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .expect("get current timestamp error")
            .as_secs()
    }

    // ic::time() return the nano_sec, we need to change it to sec.
    #[cfg(target_family = "wasm")]
    (ic_exports::ic_kit::ic::time() / 1_000_000_000)
}

// Enable Candid export
ic_exports::ic_cdk::export_candid!();
