use ic_canister_client::{CanisterClient, CanisterClientResult};
use upgrader_canister_did::BuildData;

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
    pub async fn get_canister_build_data(&self) -> CanisterClientResult<BuildData> {
        self.client.query("get_canister_build_data", ()).await
    }
}
