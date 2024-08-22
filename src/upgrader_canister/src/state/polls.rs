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
    pub fn vote(&mut self, poll_id: u64, voter_principal: Principal, approved: bool, timestamp_secs: u64) -> Result<()> {
        let mut poll = self.polls.get(&poll_id).ok_or_else(|| {
            UpgraderError::BadRequest(format!("Poll with id {} not found", poll_id))
        })?;

        // Check if the poll is open
        if timestamp_secs < poll.start_timestamp_secs {
            return Err(UpgraderError::BadRequest(
                "The poll is not opened yet".to_string(),
            ));
        }

        // Check if the poll is closed
        if timestamp_secs > poll.end_timestamp_secs {
            return Err(UpgraderError::BadRequest(
                "The poll is closed".to_string(),
            ));
        }

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
    use std::u64;

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
            start_timestamp_secs: 123456,
            end_timestamp_secs: 234567,
        });

        let poll_1_id = polls.insert(upgrader_canister_did::Poll {
            description: "poll_1".to_string(),
            yes_voters: vec![],
            no_voters: vec![],
            poll_type: PollType::ProjectHash {
                project: "project".to_owned(),
                hash: "hash".to_owned(),
            },
            start_timestamp_secs: 123456,
            end_timestamp_secs: 234567,
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
        let result = polls.vote(0, candid::Principal::anonymous(), true, 0);

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
            start_timestamp_secs: 0,
            end_timestamp_secs: 234567,
        });

        let principal_1 = Principal::from_slice(&[1, 29]);
        let principal_2 = Principal::from_slice(&[2, 29]);
        let principal_3 = Principal::from_slice(&[3, 29]);

        // Act
        polls.vote(poll_id, principal_1, true, 0).unwrap();
        polls.vote(poll_id, principal_2, false, 0).unwrap();
        polls.vote(poll_id, principal_3, true, 0).unwrap();

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
            start_timestamp_secs: 0,
            end_timestamp_secs: 234567,
        });

        let principal_1 = Principal::from_slice(&[1, 29]);
        let principal_2 = Principal::from_slice(&[2, 29]);
        let principal_3 = Principal::from_slice(&[3, 29]);
        let principal_4 = Principal::from_slice(&[4, 29]);

        // Act
        polls.vote(poll_id, principal_1, true, 0).unwrap();
        polls.vote(poll_id, principal_2, true, 0).unwrap();
        polls.vote(poll_id, principal_3, false, 0).unwrap();
        polls.vote(poll_id, principal_4, false, 0).unwrap();
        polls.vote(poll_id, principal_1, false, 0).unwrap();
        polls.vote(poll_id, principal_4, true, 0).unwrap();

        // Assert
        let poll = polls.get(&poll_id).unwrap();
        assert_eq!(poll.yes_voters.len(), 2);
        assert_eq!(poll.no_voters.len(), 2);

        assert!(poll.yes_voters.contains(&principal_2));
        assert!(poll.yes_voters.contains(&principal_4));
        assert!(poll.no_voters.contains(&principal_1));
        assert!(poll.no_voters.contains(&principal_3));
    }

    /// Should return an error if the poll is closed
    #[test]
    fn test_vote_closed_poll() {
        // Arrange
        let memory_manager = ic_stable_structures::default_ic_memory_manager();
        let mut polls = super::Polls::new(&memory_manager);

        let end_ts = 100;

        let poll_id = polls.insert(upgrader_canister_did::Poll {
            description: "poll_0".to_string(),
            yes_voters: vec![],
            no_voters: vec![],
            poll_type: PollType::ProjectHash {
                project: "project".to_owned(),
                hash: "hash".to_owned(),
            },
            start_timestamp_secs: 0,
            end_timestamp_secs: end_ts,
        });

        let principal_1 = Principal::from_slice(&[1, 29]);

        // Act & Assert
        assert!(polls.vote(poll_id, principal_1, true, 0).is_ok());
        assert!(polls.vote(poll_id, principal_1, true, end_ts-1).is_ok());
        assert!(polls.vote(poll_id, principal_1, true, end_ts).is_ok());
        assert!(polls.vote(poll_id, principal_1, true, end_ts+1).is_err());
        assert!(polls.vote(poll_id, principal_1, true, u64::MAX).is_err());

    }

    /// Should return an error if the poll is opened
    #[test]
    fn test_vote_opened_poll() {
        // Arrange
        let memory_manager = ic_stable_structures::default_ic_memory_manager();
        let mut polls = super::Polls::new(&memory_manager);

        let start_ts = 100;

        let poll_id = polls.insert(upgrader_canister_did::Poll {
            description: "poll_0".to_string(),
            yes_voters: vec![],
            no_voters: vec![],
            poll_type: PollType::ProjectHash {
                project: "project".to_owned(),
                hash: "hash".to_owned(),
            },
            start_timestamp_secs: start_ts,
            end_timestamp_secs: u64::MAX,
        });

        let principal_1 = Principal::from_slice(&[1, 29]);

        // Act & Assert
        assert!(polls.vote(poll_id, principal_1, true, start_ts).is_ok());
        assert!(polls.vote(poll_id, principal_1, true, start_ts+1).is_ok());
        assert!(polls.vote(poll_id, principal_1, true, start_ts-1).is_err());
        assert!(polls.vote(poll_id, principal_1, true, 0).is_err());
    }
}
