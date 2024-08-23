use std::borrow::Cow;

use candid::{CandidType, Deserialize};
use ic_stable_structures::stable_structures::Memory;
use ic_stable_structures::{Bound, CellStructure, MemoryManager, StableCell, Storable};
use serde::Serialize;
use upgrader_canister_did::codec;

use crate::constant::SETTINGS_MAP_MEMORY_ID;

pub struct Settings<M: Memory> {
    settings: StableCell<SettingsData, M>,
}

impl<M: Memory> Settings<M> {
    /// Create new settings
    pub fn new(memory_manager: &dyn MemoryManager<M, u8>) -> Self {
        let settings = StableCell::new(
            memory_manager.get(SETTINGS_MAP_MEMORY_ID),
            Default::default(),
        )
        .expect("failed to initialize settings in stable memory");

        Self { settings }
    }

    /// Disable the inspect message
    pub fn disable_inspect_message(&mut self, disable: bool) {
        self.update(|s| {
            s.disable_inspect_message = disable;
        });
    }

    /// Returns true if the inspect message is disabled
    pub fn is_inspect_message_disabled(&self) -> bool {
        self.read(|s| s.disable_inspect_message)
    }

    fn read<F, T>(&self, f: F) -> T
    where
        for<'a> F: FnOnce(&'a SettingsData) -> T,
    {
        f(self.settings.get())
    }

    fn update<F, T>(&mut self, f: F) -> T
    where
        for<'a> F: FnOnce(&'a mut SettingsData) -> T,
    {
        let cell = &mut self.settings;
        let mut new_settings = cell.get().clone();
        let result = f(&mut new_settings);
        cell.set(new_settings).expect("failed to set evm settings");
        result
    }
}

#[derive(Debug, Default, Deserialize, CandidType, Clone, PartialEq, Eq, Serialize)]
pub struct SettingsData {
    disable_inspect_message: bool,
}

impl Storable for SettingsData {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        codec::encode(self).into()
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        codec::decode(&bytes)
    }

    const BOUND: Bound = Bound::Unbounded;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storable_settings_data() {
        let settings = SettingsData::default();

        let serialized = settings.to_bytes();
        let deserialized = SettingsData::from_bytes(serialized);

        assert_eq!(settings, deserialized);
    }

    /// Test inspect message is not disabled by default
    #[test]
    fn test_default_inspect_message_disabled() {
        let settings = SettingsData::default();
        assert_eq!(settings.disable_inspect_message, false);
    }

    /// Test disabling the inspect message
    #[test]
    fn test_disable_inspect_message() {
        let mut settings = Settings::new(&ic_stable_structures::default_ic_memory_manager());
        assert_eq!(settings.is_inspect_message_disabled(), false);
        settings.disable_inspect_message(true);
        assert_eq!(settings.is_inspect_message_disabled(), true);
    }
}
