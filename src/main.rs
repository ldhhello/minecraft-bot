mod event;
mod file_watcher;


use std::env;
use std::time::Duration;
use lazy_static::lazy_static;

use serenity::all::{ChannelId, Ready};
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::prelude::*;

use tokio::sync::mpsc;
use tokio::sync::RwLock;

use crate::event::Event;
use crate::file_watcher::file_watcher;

struct Handler;

lazy_static! {
    static ref EVENT_SENDER: RwLock<Option<mpsc::Sender<Event>>> = RwLock::new(None);
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let sender = EVENT_SENDER.read().await;
        let Some(sender) = sender.clone() else {
            eprintln!("Error: sender does not exist");
            return;
        };
        
        sender.send(Event::Chat{
            channel_id: msg.channel_id, 
            str: msg.content,
        }).await.unwrap_or(());
    }
    async fn ready(&self, ctx: Context, _: Ready) {
        let (sender, mut receiver) = mpsc::channel::<Event>(1024);
        *EVENT_SENDER.write().await = Some(sender);

        let admin_channel_id = ChannelId::new(1462043292937093122u64);
        if let Err(why) = admin_channel_id.say(&ctx.http, "봇 켜짐").await {
            println!("Error!!!");
        }

        loop {
            let Some(event) = receiver.recv().await else {
                break;
            };

            match event {
                Event::Chat { channel_id, str } => {
                    if str == "엄준식" {
                        let send_msg = format!("엄준식은 살아있다!");
                        if let Err(why) = channel_id.say(&ctx.http, send_msg).await {
                            println!("Error sending message: {why:?}");
                        }
                    }
                }
            }
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
    let handle1 = tokio::spawn(async move {
        if let Err(why) = client.start().await {
            println!("Client error: {why:?}");
        }
    });
    
    let handle2 = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(3)).await;

        let sender = EVENT_SENDER.read().await;
        let Some(sender) = sender.clone() else {
            eprintln!("Error: sender does not exist");
            return;
        };

        if let Err(e) = file_watcher(sender).await {
            println!("Watcher error: {e:?}");
        }
    });

    let _ = tokio::join!(handle1, handle2);

    Ok(())
}