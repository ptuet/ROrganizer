//! Keys of the game that ROrganizer presses itself in the Dofus windows
//! (group invite, zaap, NPC travel): opening the chat and "follow the group
//! leader". Players can rebind them in Dofus, so they are settings too.

use serde::{Deserialize, Serialize};

use crate::triggers::Trigger;

/// A key with its modifiers, as pressed in the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameKey {
    /// Win32 virtual-key code, on the user's layout.
    pub vk: u32,
    #[serde(default)]
    pub ctrl: bool,
    #[serde(default)]
    pub shift: bool,
    #[serde(default)]
    pub alt: bool,
}

impl GameKey {
    pub const fn plain(vk: u32) -> Self {
        Self { vk, ctrl: false, shift: false, alt: false }
    }

    /// "Ctrl+W", "Space", "Ctrl+Maj+²"…
    pub fn label(&self) -> String {
        let mut out = String::new();
        for (on, name) in [(self.ctrl, "Ctrl+"), (self.shift, "Maj+"), (self.alt, "Alt+")] {
            if on {
                out.push_str(name);
            }
        }
        out.push_str(&Trigger::Key(self.vk).display_label());
        out
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameKeys {
    #[serde(default = "default_open_chat")]
    pub open_chat: GameKey,
    #[serde(default = "default_follow_leader")]
    pub follow_leader: GameKey,
}

/// Space: what opens the chat in the author's client (Enter did nothing).
fn default_open_chat() -> GameKey {
    GameKey::plain(0x20)
}

/// Ctrl+W: the game's default "follow the group leader" shortcut.
fn default_follow_leader() -> GameKey {
    GameKey { vk: 0x57, ctrl: true, shift: false, alt: false }
}

impl Default for GameKeys {
    fn default() -> Self {
        Self { open_chat: default_open_chat(), follow_leader: default_follow_leader() }
    }
}

/// How fast the chat commands are typed. The delays between keys (chat
/// opening, paste, send, window switch) are tuned on a fast PC; a slow one
/// may miss keys, so they can be multiplied.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CommandSpeed {
    #[default]
    Normal,
    Slow,
    VerySlow,
}

impl CommandSpeed {
    pub const ALL: [CommandSpeed; 3] = [Self::Normal, Self::Slow, Self::VerySlow];

    /// Multiplier applied to every delay of the chat commands.
    pub fn factor(self) -> u32 {
        match self {
            Self::Normal => 1,
            Self::Slow => 2,
            Self::VerySlow => 3,
        }
    }
}

/// Which game key the capture modal is setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameKeyTarget {
    OpenChat,
    FollowLeader,
}

impl GameKeys {
    pub fn get(&self, target: GameKeyTarget) -> GameKey {
        match target {
            GameKeyTarget::OpenChat => self.open_chat,
            GameKeyTarget::FollowLeader => self.follow_leader,
        }
    }

    pub fn set(&mut self, target: GameKeyTarget, key: GameKey) {
        match target {
            GameKeyTarget::OpenChat => self.open_chat = key,
            GameKeyTarget::FollowLeader => self.follow_leader = key,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_space_and_ctrl_w() {
        let keys = GameKeys::default();
        assert_eq!(keys.open_chat.label(), "Space");
        assert_eq!(keys.follow_leader.label(), "Ctrl+W");
    }

    #[test]
    fn label_lists_every_modifier() {
        let k = GameKey { vk: 0x41, ctrl: true, shift: true, alt: true };
        assert_eq!(k.label(), "Ctrl+Maj+Alt+A");
    }

    #[test]
    fn missing_fields_fall_back_to_the_defaults() {
        let keys: GameKeys = serde_json::from_str("{}").unwrap();
        assert_eq!(keys, GameKeys::default());
        let keys: GameKeys = serde_json::from_str(r#"{"open_chat":{"vk":13}}"#).unwrap();
        assert_eq!(keys.open_chat, GameKey::plain(13));
        assert_eq!(keys.follow_leader, GameKeys::default().follow_leader);
    }

    #[test]
    fn speed_multiplies_the_delays() {
        let factors: Vec<u32> = CommandSpeed::ALL.iter().map(|s| s.factor()).collect();
        assert_eq!(factors, [1, 2, 3]);
        assert_eq!(CommandSpeed::default(), CommandSpeed::Normal);
    }

    #[test]
    fn get_and_set_by_target() {
        let mut keys = GameKeys::default();
        keys.set(GameKeyTarget::OpenChat, GameKey::plain(13));
        assert_eq!(keys.get(GameKeyTarget::OpenChat), GameKey::plain(13));
    }
}
