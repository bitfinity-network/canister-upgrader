use candid::Principal;

use crate::pocket_ic::{build_client, deploy_canister};

/// Test that the canister_build_data query returns the correct data
#[tokio::test]
async fn test_should_query_build_data() {
    // Arrange
    let (pocket, canister_principal) = deploy_canister(None).await;
    let caller_principal = Principal::anonymous();
    let client = build_client(pocket, canister_principal, caller_principal);

    // Act
    let result = client.canister_build_data().await.unwrap();

    // Assert
    assert_eq!("upgrader_canister", result.pkg_name);
}

/// Test that the init.admin has admin permissions
#[tokio::test]
async fn test_should_grant_admin_permissions_on_init() {
    assert!(false)
}

/// Test that the admin can get/add/set permissions
#[tokio::test]
async fn test_admin_can_manage_permissions() {

    // add/remove/get permissions
    assert!(false)
}

/// Test that only the admin can get/add/set permissions
#[tokio::test]
async fn test_only_admin_can_manage_permissions() {

    // add/remove/get permissions
    assert!(false)
}

/// Test that the caller can get their own permissions
#[tokio::test]
async fn test_caller_can_get_own_permissions() {
    assert!(false)
}

/// Test that the caller can create and get projects
#[tokio::test]
async fn test_caller_can_create_and_get_projects() {
    assert!(false)
}

/// Test that the caller can't create projects if not allowed
#[tokio::test]
async fn test_caller_cant_create_projects_if_not_allowed() {
    assert!(false)
}

/// Test that the caller can create and get polls
#[tokio::test]
async fn test_caller_can_create_and_get_polls() {
    assert!(false)
}

/// Test that the caller can't create polls if not allowed
#[tokio::test]
async fn test_caller_cant_create_polls_if_not_allowed() {
    assert!(false)
}