use std::collections::BTreeMap;

use candid::Principal;
use ic_stable_structures::stable_structures::Memory;
use ic_stable_structures::{
    BTreeMapStructure, CellStructure, MemoryManager, StableBTreeMap, StableCell,
};
use upgrader_canister_did::error::{Result, UpgraderError};
use upgrader_canister_did::Poll;

use crate::constant::{POLLS_ID_SEQUENCE_MEMORY_ID, POLLS_MAP_MEMORY_ID};

/// Manages polls
pub struct Polls<M: Memory> {
    polls: StableBTreeMap<u64, Poll, M>,
    polls_id_sequence: StableCell<u64, M>,
}

impl<M: Memory> Polls<M> {
    pub fn new(memory_manager: &dyn MemoryManager<M, u8>) -> Self {
        Self {
            polls: StableBTreeMap::new(memory_manager.get(POLLS_MAP_MEMORY_ID)),
            polls_id_sequence: StableCell::new(memory_manager.get(POLLS_ID_SEQUENCE_MEMORY_ID), 0)
                .expect("stable memory POLLS_ID_SEQUENCE_MEMORY_ID initialization failed"),
        }
    }

    /// Returns the poll data for the given key
    pub fn get(&self, id: &u64) -> Option<Poll> {
        self.polls.get(id)
    }

    /// Returns all polls
    pub fn all(&self) -> BTreeMap<u64, Poll> {
        self.polls.iter().collect()
    }

    /// Inserts a new poll and returns the generated key
    pub fn insert(&mut self, poll: Poll) -> u64 {
        let id = self.next_id();
        self.polls.insert(id, poll);
        id
    }

    /// Votes for a poll. If the voter has already voted, the previous vote is replaced.
    pub fn vote(&mut self, poll_id: u64, voter_principal: Principal, approved: bool) -> Result<()> {
        let mut poll = self.polls.get(&poll_id).ok_or_else(|| {
            UpgraderError::BadRequest(format!("Poll with id {} not found", poll_id))
        })?;

        // Remove the voter from the previous vote
        poll.yes_voters.retain(|x| x != &voter_principal);
        poll.no_voters.retain(|x| x != &voter_principal);

        if approved {
            poll.yes_voters.push(voter_principal);
        } else {
            poll.no_voters.push(voter_principal);
        }

        self.polls.insert(poll_id, poll);
        Ok(())
    }

    /// Returns the next poll id
    fn next_id(&mut self) -> u64 {
        // Polls could be removed from the map so we need to keep track of the next id
        let id = *self.polls_id_sequence.get();
        self.polls_id_sequence
            .set(id + 1)
            .expect("Unable to access the stable storage to set the next poll id");
        id
    }
}

#[cfg(test)]
mod test {
    use candid::Principal;
    use upgrader_canister_did::PollType;

    /// Verifies that the next id is generated correctly
    #[test]
    fn test_next_id() {
        let memory_manager = ic_stable_structures::default_ic_memory_manager();
        let mut polls = super::Polls::new(&memory_manager);

        assert_eq!(polls.next_id(), 0);
        assert_eq!(polls.next_id(), 1);
        assert_eq!(polls.next_id(), 2);
    }

    #[test]
    fn test_insert_polls() {
        // Arrange
        let memory_manager = ic_stable_structures::default_ic_memory_manager();
        let mut polls = super::Polls::new(&memory_manager);

        // Act
        let poll_0_id = polls.insert(upgrader_canister_did::Poll {
            description: "poll_0".to_string(),
            yes_voters: vec![],
            no_voters: vec![],
            poll_type: PollType::ProjectHash {
                project: "project".to_owned(),
                hash: "hash".to_owned(),
            },
            created_timestamp_millis: 123456,
            end_timestamp_millis: 234567,
        });

        let poll_1_id = polls.insert(upgrader_canister_did::Poll {
            description: "poll_1".to_string(),
            yes_voters: vec![],
            no_voters: vec![],
            poll_type: PollType::ProjectHash {
                project: "project".to_owned(),
                hash: "hash".to_owned(),
            },
            created_timestamp_millis: 123456,
            end_timestamp_millis: 234567,
        });

        // Assert
        assert_eq!(polls.next_id(), 2);
        assert_eq!(polls.get(&poll_0_id).unwrap().description, "poll_0");
        assert_eq!(polls.get(&poll_1_id).unwrap().description, "poll_1");
    }

    /// Should return an error if voting for a poll that does not exist
    #[test]
    fn test_vote_poll_not_found() {
        // Arrange
        let memory_manager = ic_stable_structures::default_ic_memory_manager();
        let mut polls = super::Polls::new(&memory_manager);

        // Act
        let result = polls.vote(0, candid::Principal::anonymous(), true);

        // Assert
        assert!(result.is_err());
    }

    /// Should vote for a poll
    #[test]
    fn test_vote_poll() {
        // Arrange
        let memory_manager = ic_stable_structures::default_ic_memory_manager();
        let mut polls = super::Polls::new(&memory_manager);
        let poll_id = polls.insert(upgrader_canister_did::Poll {
            description: "poll_0".to_string(),
            yes_voters: vec![],
            no_voters: vec![],
            poll_type: PollType::ProjectHash {
                project: "project".to_owned(),
                hash: "hash".to_owned(),
            },
            created_timestamp_millis: 123456,
            end_timestamp_millis: 234567,
        });

        let principal_1 = Principal::from_slice(&[1, 29]);
        let principal_2 = Principal::from_slice(&[2, 29]);
        let principal_3 = Principal::from_slice(&[3, 29]);

        // Act
        polls.vote(poll_id, principal_1, true).unwrap();
        polls.vote(poll_id, principal_2, false).unwrap();
        polls.vote(poll_id, principal_3, true).unwrap();

        // Assert
        let poll = polls.get(&poll_id).unwrap();
        assert_eq!(poll.yes_voters.len(), 2);
        assert_eq!(poll.no_voters.len(), 1);

        assert!(poll.yes_voters.contains(&principal_1));
        assert!(poll.yes_voters.contains(&principal_3));
        assert!(poll.no_voters.contains(&principal_2));
    }

    /// Should replace the vote if the voter has already voted
    #[test]
    fn test_vote_replace() {
        // Arrange
        let memory_manager = ic_stable_structures::default_ic_memory_manager();
        let mut polls = super::Polls::new(&memory_manager);
        let poll_id = polls.insert(upgrader_canister_did::Poll {
            description: "poll_0".to_string(),
            yes_voters: vec![],
            no_voters: vec![],
            poll_type: PollType::ProjectHash {
                project: "project".to_owned(),
                hash: "hash".to_owned(),
            },
            created_timestamp_millis: 123456,
            end_timestamp_millis: 234567,
        });

        let principal_1 = Principal::from_slice(&[1, 29]);
        let principal_2 = Principal::from_slice(&[2, 29]);
        let principal_3 = Principal::from_slice(&[3, 29]);
        let principal_4 = Principal::from_slice(&[4, 29]);

        // Act
        polls.vote(poll_id, principal_1, true).unwrap();
        polls.vote(poll_id, principal_2, true).unwrap();
        polls.vote(poll_id, principal_3, false).unwrap();
        polls.vote(poll_id, principal_4, false).unwrap();
        polls.vote(poll_id, principal_1, false).unwrap();
        polls.vote(poll_id, principal_4, true).unwrap();

        // Assert
        let poll = polls.get(&poll_id).unwrap();
        assert_eq!(poll.yes_voters.len(), 2);
        assert_eq!(poll.no_voters.len(), 2);

        assert!(poll.yes_voters.contains(&principal_2));
        assert!(poll.yes_voters.contains(&principal_4));
        assert!(poll.no_voters.contains(&principal_1));
        assert!(poll.no_voters.contains(&principal_3));
    }
}
