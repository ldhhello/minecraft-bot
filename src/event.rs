use serenity::all::ChannelId;

pub enum Event {
    Chat{channel_id: ChannelId, str: String},
}