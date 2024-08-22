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
    Debug, Clone, CandidType, Deserialize, Hash, PartialEq, Eq, PartialOrd, Ord, serde::Serialize,
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
    Debug, Clone, CandidType, Deserialize, Hash, PartialEq, Eq, PartialOrd, Ord, serde::Serialize,
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

/// Describes the type of poll.
#[derive(
    Debug, Clone, CandidType, Deserialize, Hash, PartialEq, Eq, PartialOrd, Ord, serde::Serialize,
)]
pub struct Poll {
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

/// Describes the type of poll.
#[derive(
    Debug, Clone, CandidType, Deserialize, Hash, PartialEq, Eq, PartialOrd, Ord, serde::Serialize,
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

impl Storable for Poll {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        codec::encode(self).into()
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        codec::decode(&bytes)
    }

    const BOUND: ic_stable_structures::Bound = ic_stable_structures::Bound::Unbounded;
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
}
