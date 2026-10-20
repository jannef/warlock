//! The in-game message log.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// How many messages the log keeps: older ones drop off its front (the message history says so).
pub const CAPACITY: usize = 200;

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

/// Names in a message the key that runs a command, by the command's id in the app's key
/// settings (`help`): core knows no bindings, so the text holds `{key:help}` and the app fills
/// in the key as bound ([`Message::display_with`]).
pub fn key(command: &str) -> String {
    format!("{KEY_OPEN}{command}}}")
}

const KEY_OPEN: &str = "{key:";

impl Message {
    /// The text with its counted repeats, e.g. `You open the door. (×2)`, keys named in it
    /// ([`key`]) left as written: for dev tools, which know no bindings.
    pub fn display(&self) -> String {
        self.display_with(key)
    }

    /// The text as shown to the player: [`Message::display`] with every key named in it filled
    /// in by `key_name` from the command's id (`?` for `help` with the default keys).
    pub fn display_with(&self, key_name: impl Fn(&str) -> String) -> String {
        let mut text = String::with_capacity(self.text.len());
        let mut rest = self.text.as_str();
        while let Some(at) = rest.find(KEY_OPEN) {
            let after = &rest[at + KEY_OPEN.len()..];
            let Some(end) = after.find('}') else { break };
            text.push_str(&rest[..at]);
            text.push_str(&key_name(&after[..end]));
            rest = &after[end + 1..];
        }
        text.push_str(rest);
        if self.count > 1 {
            text.push_str(&format!(" (×{})", self.count));
        }
        text
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
    fn keys_named_in_a_message_are_filled_in_as_bound() {
        let mut log = MessageLog::default();
        log.push(1, format!("Press {} or {}.", key("help"), key("look")));
        log.push(2, format!("Press {} or {}.", key("help"), key("look")));
        let message = log.iter().next().unwrap();
        let bound = |id: &str| match id {
            "help" => "F1".to_string(),
            other => other.to_uppercase(),
        };
        assert_eq!(message.display_with(bound), "Press F1 or LOOK. (×2)");
        assert_eq!(message.display(), "Press {key:help} or {key:look}. (×2)");
        let plain = Message {
            turn: 0,
            text: "No {keys} here, {key:open".into(),
            count: 1,
        };
        assert_eq!(plain.display_with(bound), plain.text);
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
