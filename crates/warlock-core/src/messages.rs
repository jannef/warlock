//! The in-game message log.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

const CAPACITY: usize = 200;

/// One logged message. Repeats of the same text in a row are folded into one entry.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    /// Turn of the latest repeat.
    pub turn: u64,
    pub text: String,
    /// How many times in a row it was logged (1 for a single message).
    #[serde(default = "one", skip_serializing_if = "is_one")]
    pub count: u32,
}

fn one() -> u32 {
    1
}

fn is_one(n: &u32) -> bool {
    *n == 1
}

impl Message {
    /// The text as shown to the player, e.g. `You open the door. (×2)`.
    pub fn display(&self) -> String {
        if self.count > 1 {
            format!("{} (×{})", self.text, self.count)
        } else {
            self.text.clone()
        }
    }
}

/// The most recent messages, oldest first.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageLog {
    entries: VecDeque<Message>,
}

impl MessageLog {
    pub fn push(&mut self, turn: u64, text: impl Into<String>) {
        let text = text.into();
        if let Some(last) = self.entries.back_mut()
            && last.text == text
        {
            last.count = last.count.saturating_add(1);
            last.turn = turn;
            return;
        }
        if self.entries.len() == CAPACITY {
            self.entries.pop_front();
        }
        self.entries.push_back(Message {
            turn,
            text,
            count: 1,
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
    fn repeats_fold_into_one_entry() {
        let mut log = MessageLog::default();
        log.push(1, "You open the door.");
        log.push(2, "You open the door.");
        log.push(3, "You close the door.");
        log.push(4, "You open the door.");
        let shown: Vec<_> = log.iter().map(Message::display).collect();
        assert_eq!(
            shown,
            [
                "You open the door. (×2)",
                "You close the door.",
                "You open the door."
            ]
        );
        assert_eq!(log.iter().next().map(|m| m.turn), Some(2));
    }

    #[test]
    fn single_messages_serialize_without_a_count() {
        let mut log = MessageLog::default();
        log.push(1, "Hello.");
        let json = serde_json::to_string(&log).unwrap();
        assert!(!json.contains("count"), "{json}");
        let back: MessageLog = serde_json::from_str(&json).unwrap();
        assert_eq!(back, log);
    }

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
