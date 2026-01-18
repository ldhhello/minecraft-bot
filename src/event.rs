use serenity::all::ChannelId;

pub enum Event {
    Chat{channel_id: ChannelId, str: String},
    SendChat{channel_id: ChannelId, str: String},
    PlayerConnected{channel_id: ChannelId, nickname: String},
    PlayerDisconnected{channel_id: ChannelId, nickname: String},
}