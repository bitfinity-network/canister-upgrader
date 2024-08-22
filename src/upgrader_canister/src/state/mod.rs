use std::{cell::RefCell, rc::Rc};

use ic_stable_structures::{
    default_ic_memory_manager, stable_structures::DefaultMemoryImpl, VirtualMemory,
};
use permission::Permissions;
use polls::Polls;

pub mod permission;
pub mod polls;
pub mod projects;

/// State of the upgrader canister
pub struct UpgraderCanisterState {
    pub permissions: Rc<RefCell<Permissions<VirtualMemory<DefaultMemoryImpl>>>>,
    pub polls: Rc<RefCell<Polls<VirtualMemory<DefaultMemoryImpl>>>>,
    pub projects: Rc<RefCell<projects::Projects<VirtualMemory<DefaultMemoryImpl>>>>,
}

impl Default for UpgraderCanisterState {
    fn default() -> Self {
        let memory_manager = default_ic_memory_manager();

        Self {
            permissions: Rc::new(RefCell::new(Permissions::new(&memory_manager))),
            polls: Rc::new(RefCell::new(Polls::new(&memory_manager))),
            projects: Rc::new(RefCell::new(projects::Projects::new(&memory_manager))),
        }
    }
}
