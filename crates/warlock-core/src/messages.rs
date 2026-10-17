//! The in-game message log.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

const CAPACITY: usize = 200;

/// One logged message.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    pub turn: u64,
    pub text: String,
}

/// The most recent messages, oldest first.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageLog {
    entries: VecDeque<Message>,
}

impl MessageLog {
    pub fn push(&mut self, turn: u64, text: impl Into<String>) {
        if self.entries.len() == CAPACITY {
            self.entries.pop_front();
        }
        self.entries.push_back(Message {
            turn,
            text: text.into(),
        });
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &Message> {
        self.entries.iter()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drops_oldest_beyond_capacity() {
        let mut log = MessageLog::default();
        for i in 0..(CAPACITY as u64 + 5) {
            log.push(i, format!("m{i}"));
        }
        assert_eq!(log.len(), CAPACITY);
        assert_eq!(log.iter().next().map(|m| m.turn), Some(5));
    }
}
