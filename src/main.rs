use std::env;

use serenity::all::{ChannelId, Ready};
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::prelude::*;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error sending message: {why:?}");
            }
        }
        else if msg.content == "!channel_id" {
            let send_msg = format!("채널 ID : {}", msg.channel_id);
            if let Err(why) = msg.channel_id.say(&ctx.http, send_msg).await {
                println!("Error sending message: {why:?}");
            }
        }
        else if msg.content == "엄준식" {
            println!("채널 ID : {}", msg.channel_id);

            let send_msg = format!("엄준식은 살아있다");
            if let Err(why) = msg.channel_id.say(&ctx.http, send_msg).await {
                println!("Error sending message: {why:?}");
            }
        }
    }
    async fn ready(&self, ctx: Context, _: Ready) {
        let admin_channel_id = ChannelId::new(1462043292937093122u64);
        if let Err(why) = admin_channel_id.say(&ctx.http, "봇 켜짐").await {
            println!("Error!!!");
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv()?;

    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Create a new instance of the Client, logging in as a bot.
    let mut client =
        Client::builder(&token, intents).event_handler(Handler).await.expect("Err creating client");

    // Start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }

    Ok(())
}