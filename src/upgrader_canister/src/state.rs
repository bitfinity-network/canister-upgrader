use ic_stable_structures::default_ic_memory_manager;

/// State of the upgrader canister
pub struct UpgraderCanisterState {}

impl Default for UpgraderCanisterState {
    fn default() -> Self {
        let _memory_manager = default_ic_memory_manager();

        Self {
            // permissions: Rc::new(RefCell::new(Permissions::new(&memory_manager))),
        }
    }
}
