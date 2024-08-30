use std::collections::BTreeMap;

use candid::Principal;
use ic_canister_client::{CanisterClient, CanisterClientResult};
use upgrader_canister_did::error::Result;
use upgrader_canister_did::{
    BuildData, ClosedPoll, PendingPoll, Permission, PermissionList, Poll, PollCreateData,
    ProjectData,
};

/// An upgrader canister client.
#[derive(Debug, Clone)]
pub struct UpgraderCanisterClient<C>
where
    C: CanisterClient,
{
    /// The canister client.
    client: C,
}

impl<C: CanisterClient> UpgraderCanisterClient<C> {
    /// Create a new upgrader canister client.
    ///
    /// # Arguments
    /// * `client` - The canister client.
    pub fn new(client: C) -> Self {
        Self { client }
    }

    /// Returns the build data of the canister
    pub async fn canister_build_data(&self) -> CanisterClientResult<BuildData> {
        self.client.query("canister_build_data", ()).await
    }

    /// Returns the permissions of a principal
    pub async fn admin_permissions_get(
        &self,
        principal: Principal,
    ) -> CanisterClientResult<Result<PermissionList>> {
        self.client
            .query("admin_permissions_get", (principal,))
            .await
    }

    /// Adds permissions to a principal and returns the principal permissions
    pub async fn admin_permissions_add(
        &self,
        principal: Principal,
        permissions: &[Permission],
    ) -> CanisterClientResult<Result<PermissionList>> {
        self.client
            .update("admin_permissions_add", (principal, permissions))
            .await
    }

    /// Removes permissions from a principal and returns the principal permissions
    pub async fn admin_permissions_remove(
        &self,
        principal: Principal,
        permissions: &[Permission],
    ) -> CanisterClientResult<Result<PermissionList>> {
        self.client
            .update("admin_permissions_remove", (principal, permissions))
            .await
    }

    /// Disable/Enable the inspect message
    pub async fn admin_disable_inspect_message(
        &self,
        value: bool,
    ) -> CanisterClientResult<Result<()>> {
        self.client
            .update("admin_disable_inspect_message", (value,))
            .await
    }

    /// Returns whether the inspect message is disabled.
    pub async fn is_inspect_message_disabled(&self) -> CanisterClientResult<bool> {
        self.client.query("is_inspect_message_disabled", ()).await
    }

    /// Returns the permissions of the caller
    pub async fn caller_permissions_get(&self) -> CanisterClientResult<Result<PermissionList>> {
        self.client.query("caller_permissions_get", ()).await
    }

    /// Returns all projects
    pub async fn project_get_all(&self) -> CanisterClientResult<Vec<ProjectData>> {
        self.client.query("project_get_all", ()).await
    }

    /// Returns a project by key
    pub async fn project_get(&self, key: &str) -> CanisterClientResult<Option<ProjectData>> {
        self.client.query("project_get", (key,)).await
    }

    /// Creates a new project
    pub async fn project_create(&self, project: &ProjectData) -> CanisterClientResult<Result<()>> {
        self.client.update("project_create", (project,)).await
    }

    /// Returns all pending polls
    pub async fn poll_get_all_pending(&self) -> CanisterClientResult<BTreeMap<u64, PendingPoll>> {
        self.client.query("poll_get_all_pending", ()).await
    }

    /// Returns all closed polls
    pub async fn poll_get_all_closed(&self) -> CanisterClientResult<BTreeMap<u64, ClosedPoll>> {
        self.client.query("poll_get_all_closed", ()).await
    }

    /// Returns a poll by id
    pub async fn poll_get(&self, id: u64) -> CanisterClientResult<Option<Poll>> {
        self.client.query("poll_get", (id,)).await
    }

    /// Returns a poll by id searching in the pending polls
    pub async fn poll_get_pending(&self, id: u64) -> CanisterClientResult<Option<PendingPoll>> {
        self.client.query("poll_get_pending", (id,)).await
    }

    /// Returns a poll by id searching in the closed polls
    pub async fn poll_get_closed(&self, id: u64) -> CanisterClientResult<Option<ClosedPoll>> {
        self.client.query("poll_get_closed", (id,)).await
    }

    /// Creates a new poll and returns the generated poll id
    pub async fn poll_create(&self, poll: &PollCreateData) -> CanisterClientResult<Result<u64>> {
        self.client.update("poll_create", (poll,)).await
    }

    /// Votes for a poll. If the voter has already voted, the previous vote is replaced.
    pub async fn poll_vote(
        &self,
        poll_id: u64,
        approved: bool,
    ) -> CanisterClientResult<Result<()>> {
        self.client.update("poll_vote", (poll_id, approved)).await
    }
}
