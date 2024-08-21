use ic_stable_structures::stable_structures::Memory;
use ic_stable_structures::{BTreeMapStructure, MemoryManager, StableBTreeMap};
use upgrader_canister_did::error::{Result, UpgraderError};
use upgrader_canister_did::ProjectData;

use crate::constant::PROJECTS_MAP_MEMORY_ID;

/// Manages available projects
pub struct Projects<M: Memory> {
    projects: StableBTreeMap<String, ProjectData, M>,
}

impl<M: Memory> Projects<M> {
    pub fn new(memory_manager: &dyn MemoryManager<M, u8>) -> Self {
        Self {
            projects: StableBTreeMap::new(memory_manager.get(PROJECTS_MAP_MEMORY_ID)),
        }
    }

    /// Returns the project data for the given key
    pub fn get(&self, key: &String) -> Option<ProjectData> {
        self.projects.get(key)
    }

    /// Inserts the project data for the given key
    /// Returns an error if the key already exists
    pub fn insert(&mut self, project: ProjectData) -> Result<()> {
        if self.projects.contains_key(&project.key) {
            Err(UpgraderError::NotUniqueKey(project.key))
        } else {
            self.projects.insert(project.key.clone(), project);
            Ok(())
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_project_insert() {
        // Arrange
        let mut projects = Projects::new(&ic_stable_structures::default_ic_memory_manager());
        let project = ProjectData {
            key: "key".to_string(),
            name: "Project".to_string(),
            description: "Description".to_string(),
        };

        // Act
        assert!(projects.insert(project.clone()).is_ok());

        // Assert
        assert_eq!(projects.get(&project.key), Some(project));
    }

    #[test]
    fn test_project_insert_duplicate() {
        // Arrange
        let mut projects = Projects::new(&ic_stable_structures::default_ic_memory_manager());
        let project = ProjectData {
            key: "key".to_string(),
            name: "Project".to_string(),
            description: "Description".to_string(),
        };

        // Act
        assert!(projects.insert(project.clone()).is_ok());

        // Assert
        assert_eq!(projects.get(&project.key), Some(project.clone()));
        assert_eq!(
            projects.insert(project),
            Err(UpgraderError::NotUniqueKey("key".to_string()))
        );
    }

    /// Verifies that multiple projects can be inserted
    #[test]
    fn test_project_insert_multiple() {
        // Arrange
        let mut projects = Projects::new(&ic_stable_structures::default_ic_memory_manager());
        let project1 = ProjectData {
            key: "key1".to_string(),
            name: "Project1".to_string(),
            description: "Description1".to_string(),
        };
        let project2 = ProjectData {
            key: "key2".to_string(),
            name: "Project2".to_string(),
            description: "Description2".to_string(),
        };

        // Act
        assert!(projects.insert(project1.clone()).is_ok());
        assert!(projects.insert(project2.clone()).is_ok());

        // Assert
        assert_eq!(projects.get(&project1.key), Some(project1));
        assert_eq!(projects.get(&project2.key), Some(project2));
    }
}
