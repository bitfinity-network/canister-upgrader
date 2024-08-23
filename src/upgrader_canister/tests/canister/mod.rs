use std::sync::Arc;

use candid::Principal;
use ic_canister_client::CanisterClientResult;
use ic_exports::pocket_ic::PocketIc;
use upgrader_canister_did::{Permission, PollCreateData, PollType, ProjectData};

use crate::pocket_ic::{build_client, deploy_canister, ADMIN};

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
    // Arrange
    let (pocket, canister_principal) = deploy_canister(None).await;
    let caller_principal = ADMIN;
    let client = build_client(pocket, canister_principal, caller_principal);

    // Act
    let permissions = client.admin_permissions_get(ADMIN).await.unwrap().unwrap();

    // Assert
    assert!(permissions.permissions.contains(&Permission::Admin));
}

/// Test that the admin can get/add/set permissions
#[tokio::test]
async fn test_admin_can_manage_permissions() {
    // Arrange
    let (pocket, canister_principal) = deploy_canister(None).await;
    let caller_principal = ADMIN;
    let client = build_client(pocket, canister_principal, caller_principal);
    let principal = Principal::from_slice(&[1u8; 29]);

    // Act
    let permissions_before = client
        .admin_permissions_get(principal)
        .await
        .unwrap()
        .unwrap();
    let permissions_on_create = client
        .admin_permissions_add(principal, &[Permission::CreatePoll])
        .await
        .unwrap()
        .unwrap();
    let permissions_after_create = client
        .admin_permissions_get(principal)
        .await
        .unwrap()
        .unwrap();
    let permissions_on_update = client
        .admin_permissions_add(principal, &[Permission::CreateProject])
        .await
        .unwrap()
        .unwrap();
    let permissions_after_update = client
        .admin_permissions_get(principal)
        .await
        .unwrap()
        .unwrap();
    let permissions_on_remove = client
        .admin_permissions_remove(principal, &[Permission::CreatePoll])
        .await
        .unwrap()
        .unwrap();
    let permissions_after_remove = client
        .admin_permissions_get(principal)
        .await
        .unwrap()
        .unwrap();

    // Assert
    assert_eq!(permissions_before.permissions.len(), 0);

    assert_eq!(permissions_on_create.permissions.len(), 1);
    assert!(permissions_on_create
        .permissions
        .contains(&Permission::CreatePoll));
    assert_eq!(permissions_after_create, permissions_on_create);

    assert_eq!(permissions_on_update.permissions.len(), 2);
    assert!(permissions_on_update
        .permissions
        .contains(&Permission::CreatePoll));
    assert!(permissions_on_update
        .permissions
        .contains(&Permission::CreateProject));
    assert_eq!(permissions_after_update, permissions_on_update);

    assert_eq!(permissions_on_remove.permissions.len(), 1);
    assert!(permissions_on_remove
        .permissions
        .contains(&Permission::CreateProject));
    assert_eq!(permissions_after_remove, permissions_on_remove);
}

/// Test that only the admin can get/add/set permissions
#[tokio::test]
async fn test_only_admin_can_manage_permissions() {
    // Arrange
    let (pocket, canister_principal) = deploy_canister(None).await;
    let caller_principal = Principal::from_slice(&[1u8; 29]);
    let client = build_client(pocket.clone(), canister_principal, caller_principal);

    // Act & Assert
    assert!(client
        .admin_permissions_get(caller_principal)
        .await
        .unwrap()
        .is_err());

    assert_inspect_message_error(
        &client
            .admin_permissions_add(caller_principal, &[Permission::CreateProject])
            .await,
    );
    assert_inspect_message_error(
        &client
            .admin_permissions_remove(caller_principal, &[Permission::CreateProject])
            .await,
    );

    // Permission check should fail even if the inspect message is disabled
    {
        disable_inspect_message(pocket, canister_principal).await;

        // Act & Assert
        assert!(client
            .admin_permissions_add(caller_principal, &[Permission::CreateProject])
            .await
            .unwrap()
            .is_err());
        assert!(client
            .admin_permissions_remove(caller_principal, &[Permission::CreateProject])
            .await
            .unwrap()
            .is_err());
    }
}

/// Test that the admin can disable/enable the inspect message
#[tokio::test]
async fn test_admin_can_disable_inspect_message() {
    // Arrange
    let (pocket, canister_principal) = deploy_canister(None).await;
    let caller_principal = ADMIN;
    let client = build_client(pocket, canister_principal, caller_principal);

    // Act
    let inspect_message_disabled_before = client.is_inspect_message_disabled().await.unwrap();
    client
        .admin_disable_inspect_message(true)
        .await
        .unwrap()
        .unwrap();
    let inspect_message_disabled_after = client.is_inspect_message_disabled().await.unwrap();

    // Assert
    assert!(!inspect_message_disabled_before);
    assert!(inspect_message_disabled_after);
}

/// Test that only the admin can disable/enable the inspect message
#[tokio::test]
async fn test_only_admin_can_disable_inspect_message() {
    // Arrange
    let (pocket, canister_principal) = deploy_canister(None).await;
    let caller_principal = Principal::from_slice(&[1u8; 29]);
    let client = build_client(pocket.clone(), canister_principal, caller_principal);

    // Act & Assert
    assert_inspect_message_error(&client.admin_disable_inspect_message(true).await);

    // Permission check should fail even if the inspect message is disabled
    {
        disable_inspect_message(pocket, canister_principal).await;

        // Act & Assert
        assert!(client
            .admin_disable_inspect_message(false)
            .await
            .unwrap()
            .is_err());
    }
}

/// Test that the caller can get their own permissions
#[tokio::test]
async fn test_caller_can_get_own_permissions() {
    // Arrange
    let (pocket, canister_principal) = deploy_canister(None).await;
    let user_principal = Principal::from_slice(&[1u8; 29]);
    let user_client = build_client(pocket.clone(), canister_principal, user_principal);
    let admin_client = build_client(pocket, canister_principal, ADMIN);

    admin_client
        .admin_permissions_add(user_principal, &[Permission::CreatePoll])
        .await
        .unwrap()
        .unwrap();

    // Act
    let user_permissions_from_admin = admin_client
        .admin_permissions_get(user_principal)
        .await
        .unwrap()
        .unwrap();
    let user_permissions_from_user = user_client.caller_permissions_get().await.unwrap().unwrap();

    // Assert
    assert_eq!(user_permissions_from_admin.permissions.len(), 1);
    assert_eq!(user_permissions_from_user, user_permissions_from_admin);
}

/// Test that the caller can create and get projects
#[tokio::test]
async fn test_caller_can_create_and_get_projects() {
    // Arrange
    let (pocket, canister_principal) = deploy_canister(None).await;

    let admin_client = build_client(pocket.clone(), canister_principal, ADMIN);

    // User with permission to create projects
    let user_1_principal = Principal::from_slice(&[1u8; 29]);
    let user_1_client = build_client(pocket.clone(), canister_principal, user_1_principal);
    admin_client
        .admin_permissions_add(user_1_principal, &[Permission::CreateProject])
        .await
        .unwrap()
        .unwrap();

    // User with no permissions
    let user_2_principal = Principal::from_slice(&[2u8; 29]);
    let user_2_client = build_client(pocket, canister_principal, user_2_principal);

    // Act
    let project = ProjectData {
        key: "key".to_string(),
        name: "Project".to_string(),
        description: "Description".to_string(),
    };
    user_1_client
        .project_create(&project)
        .await
        .unwrap()
        .unwrap();

    // Assert
    let projects = user_2_client.project_get_all().await.unwrap();
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0], project);

    let project_from_get = user_2_client
        .project_get(&project.key)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(project_from_get, project);
}

/// Test that the caller can't create projects if not allowed
#[tokio::test]
async fn test_caller_cant_create_projects_if_not_allowed() {
    // Arrange
    let (pocket, canister_principal) = deploy_canister(None).await;
    let user_1_principal = Principal::from_slice(&[1u8; 29]);
    let user_1_client = build_client(pocket.clone(), canister_principal, user_1_principal);

    // Act
    let project = ProjectData {
        key: "key".to_string(),
        name: "Project".to_string(),
        description: "Description".to_string(),
    };
    assert_inspect_message_error(&user_1_client.project_create(&project).await);

    // Permission check should fail even if the inspect message is disabled
    {
        disable_inspect_message(pocket, canister_principal).await;

        // Act
        assert!(user_1_client
            .project_create(&project)
            .await
            .unwrap()
            .is_err());
    }

    // Assert
    let projects = user_1_client.project_get_all().await.unwrap();
    assert!(projects.is_empty());
}

/// Test that the caller can create and get polls
#[tokio::test]
async fn test_caller_can_create_and_get_polls() {
    // Arrange
    let (pocket, canister_principal) = deploy_canister(None).await;

    let admin_client = build_client(pocket.clone(), canister_principal, ADMIN);
    let project_key = "project-0";
    create_project(pocket.clone(), canister_principal, project_key).await;

    // User with permission to create polls
    let user_1_principal = Principal::from_slice(&[1u8; 29]);
    let user_1_client = build_client(pocket.clone(), canister_principal, user_1_principal);
    admin_client
        .admin_permissions_add(user_1_principal, &[Permission::CreatePoll])
        .await
        .unwrap()
        .unwrap();

    // User with no permissions
    let user_2_principal = Principal::from_slice(&[2u8; 29]);
    let user_2_client = build_client(pocket, canister_principal, user_2_principal);

    // Act
    let poll = PollCreateData {
        description: "Description".to_string(),
        poll_type: PollType::ProjectHash {
            project: project_key.to_string(),
            hash: "hash".to_string(),
        },
        start_timestamp_secs: 0,
        end_timestamp_secs: 1,
    };
    let poll_id = user_1_client.poll_create(&poll).await.unwrap().unwrap();

    // Assert
    let polls = user_2_client.poll_get_all().await.unwrap();
    assert_eq!(polls.len(), 1);
    assert_eq!(polls[&poll_id], poll.clone().into());

    let poll_from_get = user_2_client.poll_get(poll_id).await.unwrap().unwrap();
    assert_eq!(poll_from_get, poll.into());
}

/// Test that the caller cannot create a poll for a not existing project
#[tokio::test]
async fn test_caller_cant_create_poll_for_not_existing_project() {
    // Arrange
    let (pocket, canister_principal) = deploy_canister(None).await;

    let admin_client = build_client(pocket.clone(), canister_principal, ADMIN);

    // User with permission to create polls
    let user_1_principal = Principal::from_slice(&[1u8; 29]);
    let user_1_client = build_client(pocket.clone(), canister_principal, user_1_principal);
    admin_client
        .admin_permissions_add(
            user_1_principal,
            &[Permission::CreatePoll, Permission::VotePoll],
        )
        .await
        .unwrap()
        .unwrap();

    let poll = PollCreateData {
        description: "Description".to_string(),
        poll_type: PollType::ProjectHash {
            project: "project".to_string(),
            hash: "hash".to_string(),
        },
        start_timestamp_secs: 0,
        end_timestamp_secs: 1,
    };

    // Act & Assert
    assert!(user_1_client.poll_create(&poll).await.unwrap().is_err());
}

/// Test that the caller can't create polls if not allowed
#[tokio::test]
async fn test_caller_cant_create_polls_if_not_allowed() {
    // Arrange
    let (pocket, canister_principal) = deploy_canister(None).await;
    let user_1_principal = Principal::from_slice(&[1u8; 29]);
    let user_1_client = build_client(pocket.clone(), canister_principal, user_1_principal);

    let project_key = "project-1";
    create_project(pocket.clone(), canister_principal, project_key).await;

    // Act
    let poll = PollCreateData {
        description: "Description".to_string(),
        poll_type: PollType::ProjectHash {
            project: project_key.to_string(),
            hash: "hash".to_string(),
        },
        start_timestamp_secs: 0,
        end_timestamp_secs: 1,
    };
    assert_inspect_message_error(&user_1_client.poll_create(&poll).await);

    // Permission check should fail even if the inspect message is disabled
    {
        disable_inspect_message(pocket, canister_principal).await;

        // Act
        assert!(user_1_client.poll_create(&poll).await.unwrap().is_err());
    }

    // Assert
    let polls = user_1_client.poll_get_all().await.unwrap();
    assert!(polls.is_empty());
}

/// Test that the caller can vote in a poll
#[tokio::test]
async fn test_caller_can_vote_in_poll() {
    // Arrange
    let (pocket, canister_principal) = deploy_canister(None).await;
    let user_1_principal = Principal::from_slice(&[1u8; 29]);
    let user_1_client = build_client(pocket.clone(), canister_principal, user_1_principal);
    let user_2_principal = Principal::from_slice(&[2u8; 29]);
    let user_2_client = build_client(pocket.clone(), canister_principal, user_2_principal);
    let admin_client = build_client(pocket.clone(), canister_principal, ADMIN);

    let project_key = "project-10";
    create_project(pocket.clone(), canister_principal, project_key).await;

    admin_client
        .admin_permissions_add(
            user_1_principal,
            &[Permission::CreatePoll, Permission::VotePoll],
        )
        .await
        .unwrap()
        .unwrap();
    admin_client
        .admin_permissions_add(user_2_principal, &[Permission::VotePoll])
        .await
        .unwrap()
        .unwrap();

    let poll = PollCreateData {
        description: "Description".to_string(),
        poll_type: PollType::ProjectHash {
            project: project_key.to_string(),
            hash: "hash".to_string(),
        },
        start_timestamp_secs: 0,
        end_timestamp_secs: u64::MAX,
    };
    let poll_id = user_1_client.poll_create(&poll).await.unwrap().unwrap();

    // Act
    user_1_client
        .poll_vote(poll_id, false)
        .await
        .unwrap()
        .unwrap();
    user_2_client
        .poll_vote(poll_id, true)
        .await
        .unwrap()
        .unwrap();

    // Assert
    let poll = user_1_client.poll_get(poll_id).await.unwrap().unwrap();
    assert_eq!(poll.yes_voters.len(), 1);
    assert!(poll.yes_voters.contains(&user_2_principal));
    assert_eq!(poll.no_voters.len(), 1);
    assert!(poll.no_voters.contains(&user_1_principal));
}

/// Test that the caller can't vote in a poll if not allowed
#[tokio::test]
async fn test_caller_cant_vote_in_poll_if_not_allowed() {
    // Arrange
    let (pocket, canister_principal) = deploy_canister(None).await;
    let user_1_principal = Principal::from_slice(&[1u8; 29]);
    let user_1_client = build_client(pocket.clone(), canister_principal, user_1_principal);
    let user_2_principal = Principal::from_slice(&[2u8; 29]);
    let user_2_client = build_client(pocket.clone(), canister_principal, user_2_principal);
    let admin_client = build_client(pocket.clone(), canister_principal, ADMIN);

    let project_key = "project-10";
    create_project(pocket.clone(), canister_principal, project_key).await;

    admin_client
        .admin_permissions_add(user_1_principal, &[Permission::CreatePoll])
        .await
        .unwrap()
        .unwrap();

    let poll = PollCreateData {
        description: "Description".to_string(),
        poll_type: PollType::ProjectHash {
            project: project_key.to_string(),
            hash: "hash".to_string(),
        },
        start_timestamp_secs: 0,
        end_timestamp_secs: u64::MAX,
    };
    let poll_id = user_1_client.poll_create(&poll).await.unwrap().unwrap();

    // Act
    assert_inspect_message_error(&user_2_client.poll_vote(poll_id, true).await);

    // Permission check should fail even if the inspect message is disabled
    {
        disable_inspect_message(pocket, canister_principal).await;

        // Act
        assert!(user_2_client
            .poll_vote(poll_id, true)
            .await
            .unwrap()
            .is_err());
    }

    // Assert
    let poll = user_1_client.poll_get(poll_id).await.unwrap().unwrap();
    assert!(poll.yes_voters.is_empty());
    assert!(poll.no_voters.is_empty());
}

fn assert_inspect_message_error<T: std::fmt::Debug>(result: &CanisterClientResult<T>) {
    assert!(result.is_err());
    let error = result.as_ref().unwrap_err();
    assert!(error.to_string().contains("Call rejected by inspect check"));
}

async fn disable_inspect_message(pocket: Arc<PocketIc>, canister_principal: Principal) {
    let admin_client = build_client(pocket, canister_principal, ADMIN);
    admin_client
        .admin_disable_inspect_message(true)
        .await
        .unwrap()
        .unwrap();
}

async fn create_project(pocket: Arc<PocketIc>, canister_principal: Principal, project_key: &str) {
    let user_1_principal = Principal::from_slice(&[199u8; 29]);
    let user_1_client = build_client(pocket.clone(), canister_principal, user_1_principal);
    let admin_client = build_client(pocket.clone(), canister_principal, ADMIN);

    admin_client
        .admin_permissions_add(user_1_principal, &[Permission::CreateProject])
        .await
        .unwrap()
        .unwrap();

    // Act
    let project = ProjectData {
        key: project_key.to_string(),
        name: format!("Project {}", project_key),
        description: format!("Description {}", project_key),
    };
    user_1_client
        .project_create(&project)
        .await
        .unwrap()
        .unwrap();
}
