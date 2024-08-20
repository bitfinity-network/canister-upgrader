use std::sync::Arc;

use ic_canister_client::PocketIcClient;
use upgrader_canister_client::UpgraderCanisterClient;

use crate::pocket_ic::deploy_canister;

#[tokio::test]
async fn test_should_query_build_data() {
    let env = ic_exports::pocket_ic::init_pocket_ic().await;
    let canister_principal = deploy_canister(&env).await;
    let caller_principal = canister_principal;
    let client = PocketIcClient::from_client(Arc::new(env), canister_principal, caller_principal);
    let client = UpgraderCanisterClient::new(client);

    let result = client.get_canister_build_data().await.unwrap();

    assert_eq!("upgrader_canister", result.pkg_name);
}
