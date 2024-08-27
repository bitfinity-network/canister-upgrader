use std::collections::HashSet;

use candid::{CandidType, Principal};
use ic_stable_structures::Storable;
use serde::{Deserialize, Serialize};

pub mod codec;
pub mod error;

pub use error::*;

/// Contains the build data.
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct BuildData {
    pub cargo_target_triple: String,
    pub cargo_features: String,
    pub pkg_name: String,
    pub pkg_version: String,
    pub rustc_semver: String,
    pub build_timestamp: String,
    pub cargo_debug: String,
    pub git_branch: String,
    pub git_sha: String,
    pub git_commit_timestamp: String,
}

#[derive(Debug, Clone, CandidType, Deserialize)]
pub struct UpgraderCanisterInitData {
    /// Admin of the EVM Canister
    pub admin: Principal,
}

/// Principal specific permission
#[derive(
    Debug, Clone, CandidType, Deserialize, Hash, PartialEq, Eq, serde::Serialize,
)]
pub enum Permission {
    /// Gives administrator permissions
    Admin,
    /// Allows calling the endpoints to create a project (e.g. evm, bridge, etc.)
    CreateProject,
    /// Allows calling the endpoints to create a poll
    CreatePoll,
    /// Allows calling the endpoints to vote in a poll
    VotePoll,
}

#[derive(Debug, Clone, Default, CandidType, Deserialize, PartialEq, Eq, serde::Serialize)]
pub struct PermissionList {
    pub permissions: HashSet<Permission>,
}

impl Storable for PermissionList {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        codec::encode(self).into()
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        codec::decode(&bytes)
    }

    const BOUND: ic_stable_structures::Bound = ic_stable_structures::Bound::Unbounded;
}

/// Contains the project data.
#[derive(
    Debug, Clone, CandidType, Deserialize, PartialEq, Eq, serde::Serialize,
)]
pub struct ProjectData {
    /// The unique key identifier of the project.
    pub key: String,
    /// The name of the project.
    pub name: String,
    /// The description of the project.
    pub description: String,
}

impl Storable for ProjectData {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        codec::encode(self).into()
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        codec::decode(&bytes)
    }

    const BOUND: ic_stable_structures::Bound = ic_stable_structures::Bound::Unbounded;
}

/// Data required to create a poll.
#[derive(
    Debug, Clone, CandidType, Deserialize, PartialEq, Eq, serde::Serialize,
)]
pub struct PollCreateData {
    /// The description of the poll.
    pub description: String,
    /// The type of poll.
    pub poll_type: PollType,
    /// The timestamp when the poll opens.
    pub start_timestamp_secs: u64,
    /// The timestamp when the poll closes.
    pub end_timestamp_secs: u64,
}

/// Describes a pending poll.
#[derive(
    Debug, Clone, CandidType, Deserialize, PartialEq, Eq, serde::Serialize,
)]
pub struct PendingPoll {
    /// The description of the poll.
    pub description: String,
    /// The type of poll.
    pub poll_type: PollType,
    /// The list of principals that voted no.
    pub no_voters: Vec<Principal>,
    /// The list of principals that voted yes.
    pub yes_voters: Vec<Principal>,
    /// The timestamp when the poll opens.
    pub start_timestamp_secs: u64,
    /// The timestamp when the poll closes.
    pub end_timestamp_secs: u64,
}

impl PendingPoll {
    /// Returns the total number of votes.
    pub fn total_votes(&self) -> u64 {
        (self.no_voters.len() + self.yes_voters.len()) as u64
    }

    /// Returns the number of yes votes.
    pub fn yes_votes(&self) -> u64 {
        self.yes_voters.len() as u64
    }

    /// Returns the number of no votes.
    pub fn no_votes(&self) -> u64 {
        self.no_voters.len() as u64
    }

    /// Closes the poll
    pub fn close(self, result: PollResult) -> ClosedPoll {
        ClosedPoll {
            description: self.description,
            poll_type: self.poll_type,
            no_voters: self.no_voters,
            yes_voters: self.yes_voters,
            start_timestamp_secs: self.start_timestamp_secs,
            end_timestamp_secs: self.end_timestamp_secs,
            result,
        }
    }

}

impl Storable for PendingPoll {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        codec::encode(self).into()
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        codec::decode(&bytes)
    }

    const BOUND: ic_stable_structures::Bound = ic_stable_structures::Bound::Unbounded;
}
/// Describes the result of a poll.
#[derive(
    Debug, Clone, CandidType, Deserialize, PartialEq, Eq, serde::Serialize,
)]
pub enum PollResult {
    /// The poll is accepted.
    Accepted,
    /// The poll is rejected.
    Rejected,
}

/// Describes the a poll already closed.
#[derive(
    Debug, Clone, CandidType, Deserialize, PartialEq, Eq, serde::Serialize,
)]
pub struct ClosedPoll {
    /// The description of the poll.
    pub description: String,
    /// The type of poll.
    pub poll_type: PollType,
    /// The list of principals that voted no.
    pub no_voters: Vec<Principal>,
    /// The list of principals that voted yes.
    pub yes_voters: Vec<Principal>,
    /// The timestamp when the poll opens.
    pub start_timestamp_secs: u64,
    /// The timestamp when the poll closes.
    pub end_timestamp_secs: u64,
    /// The result of the poll.
    pub result: PollResult,
}

impl Storable for ClosedPoll {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        codec::encode(self).into()
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        codec::decode(&bytes)
    }

    const BOUND: ic_stable_structures::Bound = ic_stable_structures::Bound::Unbounded;
}

/// A poll data.
#[derive(
    Debug, Clone, CandidType, Deserialize, PartialEq, Eq, serde::Serialize,
)]
pub enum Poll {
    /// The poll is pending.
    Pending(PendingPoll),
    /// The poll is closed.
    Closed(ClosedPoll),
}

impl From<PollCreateData> for PendingPoll {
    fn from(value: PollCreateData) -> Self {
        Self {
            description: value.description,
            poll_type: value.poll_type,
            no_voters: Vec::new(),
            yes_voters: Vec::new(),
            start_timestamp_secs: value.start_timestamp_secs,
            end_timestamp_secs: value.end_timestamp_secs,
        }
    }
}

/// Describes the type of poll.
#[derive(
    Debug, Clone, CandidType, Deserialize, PartialEq, Eq, serde::Serialize,
)]
pub enum PollType {
    /// A poll to approve a project hash
    ProjectHash { project: String, hash: String },
    /// A poll to add permissions to principals
    AddPermission {
        principals: Vec<Principal>,
        permissions: Vec<Permission>,
    },
    /// A poll to remove permissions from principals
    RemovePermission {
        principals: Vec<Principal>,
        permissions: Vec<Permission>,
    },
}

#[cfg(test)]
mod test {

    use candid::{Decode, Encode};

    use super::*;

    #[test]
    fn test_candid_permission_list() {
        let permission_list = PermissionList {
            permissions: HashSet::from_iter(vec![Permission::Admin, Permission::CreatePoll]),
        };

        let serialized = Encode!(&permission_list).unwrap();
        let deserialized = Decode!(serialized.as_slice(), PermissionList).unwrap();

        assert_eq!(permission_list, deserialized);
    }

    #[test]
    fn test_storable_permission_list() {
        let permission_list = PermissionList {
            permissions: HashSet::from_iter(vec![Permission::Admin, Permission::CreateProject]),
        };

        let serialized = permission_list.to_bytes();
        let deserialized = PermissionList::from_bytes(serialized);

        assert_eq!(permission_list, deserialized);
    }

    #[test]
    fn test_candid_project_data() {
        let project = ProjectData {
            key: "key".to_string(),
            name: "Project".to_string(),
            description: "Description".to_string(),
        };

        let serialized = Encode!(&project).unwrap();
        let deserialized = Decode!(serialized.as_slice(), ProjectData).unwrap();

        assert_eq!(project, deserialized);
    }

    #[test]
    fn test_storable_project_data() {
        let project = ProjectData {
            key: "key".to_string(),
            name: "Project".to_string(),
            description: "Description".to_string(),
        };

        let serialized = project.to_bytes();
        let deserialized = ProjectData::from_bytes(serialized);

        assert_eq!(project, deserialized);
    }

    #[test]
    fn test_candid_poll_data() {
        let poll = PendingPoll {
            description: "Description".to_string(),
            poll_type: PollType::ProjectHash {
                project: "project".to_string(),
                hash: "hash".to_string(),
            },
            no_voters: vec![Principal::from_slice(&[1u8; 29])],
            yes_voters: vec![Principal::from_slice(&[2u8; 29])],
            start_timestamp_secs: 0,
            end_timestamp_secs: 1,
        };

        let serialized = Encode!(&poll).unwrap();
        let deserialized = Decode!(serialized.as_slice(), PendingPoll).unwrap();

        assert_eq!(poll, deserialized);
    }

    #[test]
    fn test_storable_poll_data() {
        let poll = PendingPoll {
            description: "Description".to_string(),
            poll_type: PollType::AddPermission {
                principals: vec![Principal::from_slice(&[1u8; 29])],
                permissions: vec![Permission::Admin],
            },
            no_voters: vec![Principal::from_slice(&[1u8; 29])],
            yes_voters: vec![Principal::from_slice(&[2u8; 29])],
            start_timestamp_secs: 0,
            end_timestamp_secs: 1,
        };

        let serialized = poll.to_bytes();
        let deserialized = PendingPoll::from_bytes(serialized);

        assert_eq!(poll, deserialized);
    }
}
