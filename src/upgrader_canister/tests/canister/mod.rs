use candid::Principal;

use crate::pocket_ic::{build_client, deploy_canister};

#[tokio::test]
async fn test_should_query_build_data() {
    // Arrange
    let (pocket, canister_principal) = deploy_canister(None).await;
    let caller_principal = Principal::anonymous();
    let client = build_client(pocket, canister_principal, caller_principal);

    // Act
    let result = client.get_canister_build_data().await.unwrap();

    // Assert
    assert_eq!("upgrader_canister", result.pkg_name);
}
