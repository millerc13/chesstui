use tokio::sync::mpsc;
use uuid::Uuid;

pub struct WaitingPlayer {
    pub user_id: Uuid,
    pub display_name: String,
    pub elo: i32,
    pub tx: mpsc::Sender<MatchResult>,
}

#[derive(Debug, Clone)]
pub struct MatchResult {
    pub game_id: String,
    pub opponent_name: String,
    pub my_color: String,
}

pub struct Matchmaker {
    queue: Vec<WaitingPlayer>,
}

impl Matchmaker {
    pub fn new() -> Self {
        Self { queue: Vec::new() }
    }

    pub fn enqueue(&mut self, player: WaitingPlayer) -> Option<(WaitingPlayer, WaitingPlayer)> {
        if let Some(opponent) = self.queue.pop() {
            Some((opponent, player))
        } else {
            self.queue.push(player);
            None
        }
    }

    pub fn remove(&mut self, user_id: &Uuid) {
        self.queue.retain(|p| p.user_id != *user_id);
    }
}

impl Default for Matchmaker {
    fn default() -> Self {
        Self::new()
    }
}
