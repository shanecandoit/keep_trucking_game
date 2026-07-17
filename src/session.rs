use bevy::prelude::*;

pub const DEFAULT_SEED: u64 = 0x4B45_4550_5452_554B;

#[derive(Resource)]
pub struct GameSession {
    random_state: u64,
    next_contract_id: u64,
}

impl GameSession {
    pub fn new(seed: u64) -> Self {
        Self {
            random_state: seed,
            next_contract_id: 1,
        }
    }

    pub fn next_contract_id(&mut self) -> u64 {
        let id = self.next_contract_id;
        self.next_contract_id += 1;
        id
    }

    pub fn next_index(&mut self, len: usize) -> usize {
        assert!(len > 0, "cannot choose from an empty collection");
        // A small deterministic generator is sufficient for session content;
        // saving `random_state` later will preserve exact continuation.
        self.random_state = self
            .random_state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (self.random_state % len as u64) as usize
    }
}

impl Default for GameSession {
    fn default() -> Self {
        Self::new(DEFAULT_SEED)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_seeds_generate_equal_session_sequences() {
        let mut first = GameSession::new(42);
        let mut second = GameSession::new(42);

        let first_sequence: Vec<_> = (0..6).map(|_| first.next_index(19)).collect();
        let second_sequence: Vec<_> = (0..6).map(|_| second.next_index(19)).collect();

        assert_eq!(first_sequence, second_sequence);
        assert_eq!(first.next_contract_id(), second.next_contract_id());
    }
}
