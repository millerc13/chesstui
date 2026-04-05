use serde::{Deserialize, Serialize};

// Client -> Server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    Authenticate { email: String },
    VerifyOtp { email: String, code: String },
    ResumeSession { token: String },
    SetDisplayName { name: String },
    FindGame,
    CancelSearch,
    MakeMove { game_id: String, mv: String }, // SAN notation
    Resign { game_id: String },
    OfferDraw { game_id: String },
    AcceptDraw { game_id: String },
    DeclineDraw { game_id: String },
    SetPassword { password: String },
    LoginWithPassword { email: String, password: String },
    GetPreferences,
    UpdatePreferences { preferences: serde_json::Value },
    AddFriend { email: String },
    RemoveFriend { friend_id: String },
    ListFriends,
    Ping,
}

// Server -> Client
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    AuthCodeSent,
    Authenticated { token: String, user: UserInfo },
    AuthError { reason: String },
    NeedDisplayName,
    Searching,
    GameStart { game_id: String, opponent: String, my_color: String },
    MoveMade { game_id: String, mv: String },
    MoveRejected { reason: String },
    GameOver { game_id: String, result: String, detail: String },
    DrawOffered { game_id: String },
    NeedPassword,
    PasswordSet,
    PreferencesLoaded { preferences: serde_json::Value },
    PreferencesUpdated,
    FriendsList { friends: Vec<FriendInfo> },
    FriendAdded { friend: FriendInfo },
    FriendRemoved { friend_id: String },
    Error { message: String },
    Pong,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendInfo {
    pub id: String,
    pub display_name: String,
    pub elo: i32,
    pub online: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub email: String,
    pub display_name: Option<String>,
    pub elo: i32,
}
