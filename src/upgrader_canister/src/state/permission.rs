use candid::Principal;

use ic_stable_structures::stable_structures::Memory;
use ic_stable_structures::{BTreeMapStructure, MemoryManager, StableBTreeMap};
use log::info;
use upgrader_canister_did::error::UpgraderError;
use upgrader_canister_did::{error::Result, Permission, PermissionList};

use crate::constant::PERMISSIONS_MAP_MEMORY_ID;

/// Manages IC principals that have special votign rights
pub struct Permissions<M: Memory> {
    permission_data: StableBTreeMap<Principal, PermissionList, M>,
}

impl<M: Memory> Permissions<M> {
    pub fn new(memory_manager: &dyn MemoryManager<M, u8>) -> Self {
        Self {
            permission_data: StableBTreeMap::new(memory_manager.get(PERMISSIONS_MAP_MEMORY_ID)),
        }
    }

    /// Checks if the user has the Admin permission
    pub fn check_admin(&self, principal: &Principal) -> Result<()> {
        self.check_has_all_permissions(principal, &[Permission::Admin])
    }

    /// Returns NotAuthorized error if the user does not have all permissions
    pub fn check_has_all_permissions(
        &self,
        principal: &Principal,
        permissions: &[Permission],
    ) -> Result<()> {
        if self.has_all_permissions(principal, permissions) {
            Ok(())
        } else {
            Err(UpgraderError::NotAuthorized)
        }
    }

    /// Returns whether the user has all the required permissions
    pub fn has_all_permissions(&self, principal: &Principal, permissions: &[Permission]) -> bool {
        if let Some(permissions_list) = self.permission_data.get(principal) {
            permissions
                .iter()
                .all(|item| permissions_list.permissions.contains(item))
        } else {
            permissions.is_empty()
        }
    }

    /// Returns NotAuthorized error if the user does not have at least one of the permissions
    pub fn check_has_any_permission(
        &self,
        principal: &Principal,
        permissions: &[Permission],
    ) -> Result<()> {
        if self.has_any_permission(principal, permissions) {
            Ok(())
        } else {
            Err(UpgraderError::NotAuthorized)
        }
    }

    /// Return whether the user has at least one of the required permissions
    pub fn has_any_permission(&self, principal: &Principal, permissions: &[Permission]) -> bool {
        if let Some(permissions_list) = self.permission_data.get(principal) {
            permissions
                .iter()
                .any(|item| permissions_list.permissions.contains(item))
                || permissions.is_empty()
        } else {
            permissions.is_empty()
        }
    }

    /// Add permissions to a user
    pub fn add_permissions(
        &mut self,
        principal: Principal,
        permissions: Vec<Permission>,
    ) -> Result<PermissionList> {
        self.check_anonymous_principal(&principal)?;

        info!(
            "Adding permissions {:?} to principal {}",
            permissions, principal
        );

        let mut existing_permissions = self.permission_data.get(&principal).unwrap_or_default();
        for permission in permissions {
            existing_permissions.permissions.insert(permission);
        }
        self.permission_data
            .insert(principal, existing_permissions.clone());
        Ok(existing_permissions)
    }

    /// Remove permissions from a user
    pub fn remove_permissions(
        &mut self,
        principal: Principal,
        permissions: &[Permission],
    ) -> PermissionList {
        let mut existing_permissions = self.permission_data.get(&principal).unwrap_or_default();
        existing_permissions
            .permissions
            .retain(|x| !permissions.contains(x));
        if !existing_permissions.permissions.is_empty() {
            self.permission_data
                .insert(principal, existing_permissions.clone());
        } else {
            self.permission_data.remove(&principal);
        }
        existing_permissions
    }

    /// Return the user permissions
    pub fn get_permissions(&self, principal: &Principal) -> PermissionList {
        self.permission_data.get(principal).unwrap_or_default()
    }

    /// Clear the Whitelist state
    pub fn clear(&mut self) {
        self.permission_data.clear()
    }

    fn check_anonymous_principal(&self, principal: &Principal) -> Result<()> {
        if principal == &Principal::anonymous() {
            return Err(UpgraderError::AnonymousPrincipalNotAllowed);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use std::collections::HashSet;

    use ic_exports::ic_kit::MockContext;
    use ic_stable_structures::default_ic_memory_manager;

    use super::*;

    #[test]
    fn should_have_no_permissions() {
        // Arrange
        MockContext::new().inject();
        let mut permissions = Permissions::new(&default_ic_memory_manager());

        let principal = Principal::from_slice(&[1; 29]);

        // Assert
        assert!(permissions.has_all_permissions(&principal, &[]));
        assert!(!permissions.has_all_permissions(&principal, &[Permission::CreatePoll]));
        assert!(permissions.has_any_permission(&principal, &[]));
        assert!(!permissions.has_any_permission(&principal, &[Permission::VotePoll]));

        permissions
            .add_permissions(principal, vec![Permission::CreatePoll])
            .unwrap();

        assert!(permissions.has_all_permissions(&principal, &[]));
        assert!(!permissions.has_all_permissions(&principal, &[Permission::VotePoll]));
        assert!(permissions.has_any_permission(&principal, &[]));
        assert!(!permissions.has_any_permission(&principal, &[Permission::VotePoll]));
    }

    #[test]
    fn should_return_the_user_permissions() {
        // Arrange
        MockContext::new().inject();
        let mut permissions = Permissions::new(&default_ic_memory_manager());
        permissions.clear();

        let principal = Principal::from_slice(&[1; 29]);

        // Assert
        assert_eq!(
            PermissionList::default(),
            permissions.get_permissions(&principal)
        );

        assert_eq!(
            PermissionList {
                permissions: HashSet::from_iter(vec![Permission::CreatePoll])
            },
            permissions
                .add_permissions(principal, vec![Permission::CreatePoll])
                .unwrap()
        );
        assert_eq!(
            PermissionList {
                permissions: HashSet::from_iter(vec![Permission::CreatePoll])
            },
            permissions.get_permissions(&principal)
        );

        assert_eq!(
            PermissionList {
                permissions: HashSet::from_iter(vec![Permission::CreatePoll])
            },
            permissions
                .add_permissions(
                    principal,
                    vec![Permission::CreatePoll, Permission::CreatePoll]
                )
                .unwrap()
        );
        assert_eq!(
            PermissionList {
                permissions: HashSet::from_iter(vec![Permission::CreatePoll])
            },
            permissions.get_permissions(&principal)
        );

        assert_eq!(
            PermissionList {
                permissions: HashSet::from_iter(vec![Permission::CreatePoll, Permission::VotePoll])
            },
            permissions
                .add_permissions(principal, vec![Permission::VotePoll])
                .unwrap()
        );
        assert_eq!(
            PermissionList {
                permissions: HashSet::from_iter(vec![Permission::CreatePoll, Permission::VotePoll])
            },
            permissions.get_permissions(&principal)
        );

        assert_eq!(
            PermissionList::default(),
            permissions.remove_permissions(
                principal,
                &[
                    Permission::VotePoll,
                    Permission::CreatePoll,
                    Permission::Admin
                ]
            )
        );
        assert_eq!(
            PermissionList::default(),
            permissions.get_permissions(&principal)
        );

        assert_eq!(
            PermissionList::default(),
            permissions
                .remove_permissions(principal, &[Permission::VotePoll, Permission::CreatePoll])
        );
        assert_eq!(
            PermissionList::default(),
            permissions.get_permissions(&principal)
        );
    }

    #[test]
    fn should_add_and_remove_permissions() {
        // Arrange
        MockContext::new().inject();
        let mut permissions = Permissions::new(&default_ic_memory_manager());

        let principal_1 = Principal::from_slice(&[1; 29]);
        let principal_2 = Principal::from_slice(&[2; 29]);
        let principal_3 = Principal::from_slice(&[3; 29]);
        let principal_4 = Principal::from_slice(&[4; 29]);
        let principal_5 = Principal::from_slice(&[5; 29]);

        // Add permissions
        {
            permissions
                .add_permissions(principal_2, vec![Permission::CreatePoll])
                .unwrap();
            permissions
                .add_permissions(principal_3, vec![Permission::VotePoll])
                .unwrap();
            permissions
                .add_permissions(
                    principal_4,
                    vec![Permission::CreatePoll, Permission::VotePoll],
                )
                .unwrap();
            permissions
                .add_permissions(principal_5, vec![Permission::CreatePoll])
                .unwrap();
            permissions
                .add_permissions(principal_5, vec![Permission::VotePoll])
                .unwrap();

            // Assert
            assert!(!permissions.has_all_permissions(&principal_1, &[Permission::CreatePoll]));
            assert!(!permissions.has_all_permissions(&principal_1, &[Permission::VotePoll]));
            assert!(!permissions.has_all_permissions(
                &principal_1,
                &[Permission::CreatePoll, Permission::VotePoll]
            ));
            assert!(!permissions.has_any_permission(&principal_1, &[Permission::CreatePoll]));
            assert!(!permissions.has_any_permission(&principal_1, &[Permission::VotePoll]));
            assert!(!permissions.has_any_permission(
                &principal_1,
                &[Permission::CreatePoll, Permission::VotePoll]
            ));

            assert!(permissions.has_all_permissions(&principal_2, &[Permission::CreatePoll]));
            assert!(!permissions.has_all_permissions(&principal_2, &[Permission::VotePoll]));
            assert!(!permissions.has_all_permissions(
                &principal_2,
                &[Permission::CreatePoll, Permission::VotePoll]
            ));
            assert!(permissions.has_any_permission(&principal_2, &[Permission::CreatePoll]));
            assert!(!permissions.has_any_permission(&principal_2, &[Permission::VotePoll]));
            assert!(permissions.has_any_permission(
                &principal_2,
                &[Permission::CreatePoll, Permission::VotePoll]
            ));

            assert!(!permissions.has_all_permissions(&principal_3, &[Permission::CreatePoll]));
            assert!(permissions.has_all_permissions(&principal_3, &[Permission::VotePoll]));
            assert!(!permissions.has_all_permissions(
                &principal_3,
                &[Permission::CreatePoll, Permission::VotePoll]
            ));
            assert!(!permissions.has_any_permission(&principal_3, &[Permission::CreatePoll]));
            assert!(permissions.has_any_permission(&principal_3, &[Permission::VotePoll]));
            assert!(permissions.has_any_permission(
                &principal_3,
                &[Permission::CreatePoll, Permission::VotePoll]
            ));

            assert!(permissions.has_all_permissions(&principal_4, &[Permission::CreatePoll]));
            assert!(permissions.has_all_permissions(&principal_4, &[Permission::VotePoll]));
            assert!(permissions.has_all_permissions(
                &principal_4,
                &[Permission::CreatePoll, Permission::VotePoll]
            ));
            assert!(permissions.has_any_permission(&principal_4, &[Permission::CreatePoll]));
            assert!(permissions.has_any_permission(&principal_4, &[Permission::VotePoll]));
            assert!(permissions.has_any_permission(
                &principal_4,
                &[Permission::CreatePoll, Permission::VotePoll]
            ));

            assert!(permissions.has_all_permissions(&principal_5, &[Permission::CreatePoll]));
            assert!(permissions.has_all_permissions(&principal_5, &[Permission::VotePoll]));
            assert!(permissions.has_all_permissions(
                &principal_5,
                &[Permission::CreatePoll, Permission::VotePoll]
            ));
            assert!(permissions.has_any_permission(&principal_5, &[Permission::CreatePoll]));
            assert!(permissions.has_any_permission(&principal_5, &[Permission::VotePoll]));
            assert!(permissions.has_any_permission(
                &principal_5,
                &[Permission::CreatePoll, Permission::VotePoll]
            ));
        }

        // remove permissions
        {
            permissions.remove_permissions(principal_1, &[Permission::CreatePoll]);
            permissions.remove_permissions(principal_2, &[Permission::CreatePoll]);
            permissions.remove_permissions(principal_3, &[Permission::CreatePoll]);
            permissions.remove_permissions(principal_4, &[Permission::CreatePoll]);
            permissions
                .remove_permissions(principal_5, &[Permission::CreatePoll, Permission::VotePoll]);

            // Assert
            assert!(!permissions.has_all_permissions(&principal_1, &[Permission::CreatePoll]));
            assert!(!permissions.has_all_permissions(&principal_1, &[Permission::VotePoll]));
            assert!(!permissions.has_all_permissions(
                &principal_1,
                &[Permission::CreatePoll, Permission::VotePoll]
            ));
            assert!(!permissions.has_any_permission(&principal_1, &[Permission::CreatePoll]));
            assert!(!permissions.has_any_permission(&principal_1, &[Permission::VotePoll]));
            assert!(!permissions.has_any_permission(
                &principal_1,
                &[Permission::CreatePoll, Permission::VotePoll]
            ));

            assert!(!permissions.has_all_permissions(&principal_2, &[Permission::CreatePoll]));
            assert!(!permissions.has_all_permissions(&principal_2, &[Permission::VotePoll]));
            assert!(!permissions.has_all_permissions(
                &principal_2,
                &[Permission::CreatePoll, Permission::VotePoll]
            ));
            assert!(!permissions.has_any_permission(&principal_2, &[Permission::CreatePoll]));
            assert!(!permissions.has_any_permission(&principal_2, &[Permission::VotePoll]));
            assert!(!permissions.has_any_permission(
                &principal_2,
                &[Permission::CreatePoll, Permission::VotePoll]
            ));

            assert!(!permissions.has_all_permissions(&principal_3, &[Permission::CreatePoll]));
            assert!(permissions.has_all_permissions(&principal_3, &[Permission::VotePoll]));
            assert!(!permissions.has_all_permissions(
                &principal_3,
                &[Permission::CreatePoll, Permission::VotePoll]
            ));
            assert!(!permissions.has_any_permission(&principal_3, &[Permission::CreatePoll]));
            assert!(permissions.has_any_permission(&principal_3, &[Permission::VotePoll]));
            assert!(permissions.has_any_permission(
                &principal_3,
                &[Permission::CreatePoll, Permission::VotePoll]
            ));

            assert!(!permissions.has_all_permissions(&principal_4, &[Permission::CreatePoll]));
            assert!(permissions.has_all_permissions(&principal_4, &[Permission::VotePoll]));
            assert!(!permissions.has_all_permissions(
                &principal_4,
                &[Permission::CreatePoll, Permission::VotePoll]
            ));
            assert!(!permissions.has_any_permission(&principal_4, &[Permission::CreatePoll]));
            assert!(permissions.has_any_permission(&principal_4, &[Permission::VotePoll]));
            assert!(permissions.has_any_permission(
                &principal_4,
                &[Permission::CreatePoll, Permission::VotePoll]
            ));

            assert!(!permissions.has_all_permissions(&principal_5, &[Permission::CreatePoll]));
            assert!(!permissions.has_all_permissions(&principal_5, &[Permission::VotePoll]));
            assert!(!permissions.has_all_permissions(
                &principal_5,
                &[Permission::CreatePoll, Permission::VotePoll]
            ));
            assert!(!permissions.has_any_permission(&principal_5, &[Permission::CreatePoll]));
            assert!(!permissions.has_any_permission(&principal_5, &[Permission::VotePoll]));
            assert!(!permissions.has_any_permission(
                &principal_5,
                &[Permission::CreatePoll, Permission::VotePoll]
            ));
        }
    }

    #[test]
    fn should_check_permissions_and_return_error() {
        // Arrange
        MockContext::new().inject();
        let mut permissions = Permissions::new(&default_ic_memory_manager());

        let principal_1 = Principal::from_slice(&[1; 29]);

        permissions
            .add_permissions(principal_1, vec![Permission::CreatePoll])
            .unwrap();

        // Assert
        assert_eq!(
            Err(UpgraderError::NotAuthorized),
            permissions.check_has_all_permissions(
                &principal_1,
                &[Permission::CreatePoll, Permission::VotePoll]
            )
        );
        assert!(permissions
            .check_has_all_permissions(&principal_1, &[Permission::CreatePoll])
            .is_ok());
        assert!(permissions
            .check_has_all_permissions(&principal_1, &[Permission::VotePoll])
            .is_err());

        assert!(permissions
            .check_has_any_permission(
                &principal_1,
                &[Permission::CreatePoll, Permission::VotePoll]
            )
            .is_ok());
        assert!(permissions
            .check_has_any_permission(&principal_1, &[Permission::CreatePoll])
            .is_ok());
        assert_eq!(
            Err(UpgraderError::NotAuthorized),
            permissions.check_has_any_permission(&principal_1, &[Permission::VotePoll])
        );
    }

    #[test]
    fn should_check_if_user_is_admin() {
        // Arrange
        MockContext::new().inject();
        let mut permissions = Permissions::new(&default_ic_memory_manager());

        let principal_1 = Principal::from_slice(&[1; 29]);
        assert_eq!(
            Err(UpgraderError::NotAuthorized),
            permissions.check_admin(&principal_1)
        );

        permissions
            .add_permissions(principal_1, vec![Permission::CreatePoll])
            .unwrap();
        assert_eq!(
            Err(UpgraderError::NotAuthorized),
            permissions.check_admin(&principal_1)
        );

        permissions
            .add_permissions(principal_1, vec![Permission::Admin])
            .unwrap();
        assert_eq!(Ok(()), permissions.check_admin(&principal_1));

        permissions.remove_permissions(principal_1, &[Permission::Admin]);
        assert_eq!(
            Err(UpgraderError::NotAuthorized),
            permissions.check_admin(&principal_1)
        );
    }

    #[test]
    fn check_anonymous_principal_is_rejected() {
        // Arrange
        MockContext::new().inject();
        let mut permissions = Permissions::new(&default_ic_memory_manager());
        permissions.clear();

        let principal_1 = Principal::anonymous();

        let res = permissions
            .add_permissions(principal_1, vec![Permission::CreatePoll])
            .unwrap_err();

        assert_eq!(UpgraderError::AnonymousPrincipalNotAllowed, res);
    }
}
