use std::cell::RefCell;
use std::rc::Rc;

use ic_stable_structures::stable_structures::DefaultMemoryImpl;
use ic_stable_structures::{default_ic_memory_manager, VirtualMemory};
use permission::Permissions;
use polls::Polls;
use settings::Settings;

pub mod permission;
pub mod polls;
pub mod projects;
pub mod settings;

/// State of the upgrader canister
pub struct UpgraderCanisterState {
    pub permissions: Rc<RefCell<Permissions<VirtualMemory<DefaultMemoryImpl>>>>,
    pub polls: Rc<RefCell<Polls<VirtualMemory<DefaultMemoryImpl>>>>,
    pub projects: Rc<RefCell<projects::Projects<VirtualMemory<DefaultMemoryImpl>>>>,
    pub settings: Rc<RefCell<Settings<VirtualMemory<DefaultMemoryImpl>>>>,
}

impl Default for UpgraderCanisterState {
    fn default() -> Self {
        let memory_manager = default_ic_memory_manager();

        Self {
            permissions: Rc::new(RefCell::new(Permissions::new(&memory_manager))),
            polls: Rc::new(RefCell::new(Polls::new(&memory_manager))),
            projects: Rc::new(RefCell::new(projects::Projects::new(&memory_manager))),
            settings: Rc::new(RefCell::new(Settings::new(&memory_manager))),
        }
    }
}
