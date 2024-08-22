use std::collections::BTreeMap;

use candid::Principal;
use ic_canister::{init, post_upgrade, query, update, Canister, MethodType, PreUpdate};
use ic_exports::ic_kit::ic;
use upgrader_canister_did::{error::Result, BuildData, Permission, PermissionList, Poll, ProjectData, UpgraderCanisterInitData};

use crate::{build_data::canister_build_data, state::UpgraderCanisterState};

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
    pub fn init(&mut self, data: UpgraderCanisterInitData) {

        STATE.with(|state| {
            let mut permissions = state.permissions.borrow_mut();
            permissions
                .add_permissions(data.admin, vec![Permission::Admin])
                .expect("failed to add admin permission");
        })
    }

    /// Returns the build data of the canister
    #[query]
    pub fn canister_build_data(&self) -> BuildData {
        canister_build_data()
    }

    /// Returns the permissions of a principal
    #[query]
    pub fn admin_permissions_get(&self, principal: Principal) -> Result<PermissionList> {
        STATE.with(|state| {
            let permissions = state.permissions.borrow();
            permissions.check_admin(&ic::caller())?;
            Ok(permissions.get_permissions(&principal))
        })
    }

    /// Adds permissions to a principal and returns the principal permissions
    #[update]
    pub fn admin_permissions_add(
        &mut self,
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
        &mut self,
        principal: Principal,
        permissions: Vec<Permission>,
    ) -> Result<PermissionList> {
        STATE.with(|state| {
            state.permissions.borrow().check_admin(&ic::caller())?;
            Ok(state
                .permissions
                .borrow_mut()
                .remove_permissions(principal, &permissions))
        })
    }

    /// Returns the permissions of the caller
    #[query]
    pub fn caller_permissions_get(&self) -> Result<PermissionList> {
        STATE.with(|state| {
            let permissions = state.permissions.borrow();
            Ok(permissions.get_permissions(&ic::caller()))
        })
    }

    /// Returns all projects
    #[query]
    pub fn project_get_all(&self) -> Vec<ProjectData> {
        STATE.with(|state| state.projects.borrow().all())
    }

    /// Returns a project by key
    #[query]
    pub fn project_get(&self, key: String) -> Option<ProjectData> {
        STATE.with(|state| state.projects.borrow().get(&key))
    }

    /// Creates a new project
    #[update]
    pub fn project_create(&mut self, project: ProjectData) -> Result<()> {
        STATE.with(|state| {
            state.permissions.borrow().check_has_all_permissions(&ic::caller(), &[Permission::CreateProject])?;
            state.projects.borrow_mut().insert(project)
        })
    }

    /// Returns all polls
    #[query]
    pub fn poll_get_all(&self) -> BTreeMap<u64, Poll> {
        STATE.with(|state| state.polls.borrow().all())
    }

    /// Returns a poll by id
    #[query]
    pub fn poll_get(&self, id: u64) -> Option<Poll> {
        STATE.with(|state| state.polls.borrow().get(&id))
    }

    /// Creates a new poll and returns the generated poll id
    #[update]
    pub fn poll_create(&mut self, poll: Poll) -> Result<u64> {
        STATE.with(|state| {
            state.permissions.borrow().check_has_all_permissions(&ic::caller(), &[Permission::CreatePoll])?;
            Ok(state.polls.borrow_mut().insert(poll))
        })
    }

    /// Votes for a poll. If the voter has already voted, the previous vote is replaced.
    #[update]
    pub fn poll_vote(&mut self, poll_id: u64, approved: bool) -> Result<()> {
        STATE.with(|state| {
            let caller = ic::caller();
            state.permissions.borrow().check_has_all_permissions(&caller, &[Permission::VotePoll])?;
            state.polls.borrow_mut().vote(poll_id, caller, approved)
        })
    }

}
