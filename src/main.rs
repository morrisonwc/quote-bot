use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready, channel::ReactionType},
    prelude::*,
};
use sqlx::mysql::{MySqlPool, MySqlPoolOptions};
use dotenv::dotenv;
use rand::seq::SliceRandom;
use std::env;
use regex::Regex;
use lazy_static::lazy_static;

struct DbPool;

impl TypeMapKey for DbPool {
    type Value = MySqlPool;
}

lazy_static! {
    static ref ADD_QUOTE_PATTERN: Regex = Regex::new(r#"!addquote\s+"([^"]+)"\s*-\s*(.*)"#).unwrap();
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }

        if msg.content.starts_with("!quote") {
            let pool = {
                let data = ctx.data.read().await;
                data.get::<DbPool>().unwrap().clone()
            };

            let parts: Vec<&str> = msg.content.splitn(2, ' ').collect();
            
            let quote_to_send: Option<String> = if parts.len() > 1 {
                let name = parts[1].trim();
                let quotes = sqlx::query!("SELECT text, author FROM quotes WHERE author LIKE ?", format!("%{}%", name))
                    .fetch_all(&pool)
                    .await
                    .expect("Failed to fetch quotes");

                if let Some(quote) = quotes.choose(&mut rand::thread_rng()) {
                    Some(format!("\"{}\" - {}", quote.text.clone(), quote.author.clone()))
                } else {
                    Some(format!("No quotes found for \"{}\".", name))
                }
            } else {
                let quotes = sqlx::query!("SELECT text, author FROM quotes")
                    .fetch_all(&pool)
                    .await
                    .expect("Failed to fetch quotes");
                
                if let Some(quote) = quotes.choose(&mut rand::thread_rng()) {
                    Some(format!("\"{}\" - {}", quote.text.clone(), quote.author.clone()))
                } else {
                    Some("No quotes found in the database.".to_string())
                }
            };
            
            if let Some(quote_text) = quote_to_send {
                if let Err(why) = msg.channel_id.say(&ctx.http, quote_text).await {
                    println!("Error sending message: {:?}", why);
                }
            }
        } else if msg.content.starts_with("!addquote") {
            if let Some(captures) = ADD_QUOTE_PATTERN.captures(&msg.content) {
                let quote_text = captures.get(1).map(|m| m.as_str()).unwrap_or("");
                let author = captures.get(2).map(|m| m.as_str()).unwrap_or("");

                let pool = {
                    let data = ctx.data.read().await;
                    data.get::<DbPool>().unwrap().clone()
                };

                let result = sqlx::query!("INSERT INTO quotes (text, author) VALUES (?, ?)", quote_text, author)
                    .execute(&pool)
                    .await;

                if let Err(e) = result {
                    println!("Failed to insert quote: {:?}", e);
                    let _ = msg.channel_id.say(&ctx.http, "Failed to add quote.").await;
                } else {
                    if let Err(e) = msg.react(&ctx.http, ReactionType::Unicode("👍".to_string())).await {
                        println!("Failed to add reaction: {:?}", e);
                    }
                }
            } else {
                let _ = msg.channel_id.say(&ctx.http, "Invalid format. Use: `!addquote \"quote text here\" - Name`").await;
            }
        } else if msg.content.starts_with("!help") {
            let parts: Vec<&str> = msg.content.splitn(2, ' ').collect();

            let help_message = match parts.get(1) {
                Some(&"!quote") => Some(
                    "**!quote**: Retrieves a quote from the database.\n\
                     `!quote` - Gets a random quote.\n\
                     `!quote <Name>` - Gets a quote by a specific person."
                ),
                Some(&"!addquote") => Some(
                    "**!addquote**: Adds a new quote to the database.\n\
                     `!addquote \"quote text here\" - <Name>`"
                ),
                Some(_) => Some(
                    "Command not recognized. Try `!help` or `!help !quote` or `!help !addquote`."
                ),
                None => Some(
                    "I can retrieve or add quotes. Here are my available commands:\n\
                     `!quote` - Retrieves a quote from the database.\n\
                     `!addquote` - Adds a new quote to the database.\n\
                     For more details on a command, use `!help <command>` (e.g., `!help !quote`)."
                ),
            };

            if let Some(message_text) = help_message {
                if let Err(e) = msg.channel_id.say(&ctx.http, message_text).await {
                    println!("Failed to send help message: {:?}", e);
                }
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let pool = {
            let data = ctx.data.read().await;
            data.get::<DbPool>().unwrap().clone()
        };

        let _ = sqlx::query("CREATE TABLE IF NOT EXISTS quotes (id INT NOT NULL AUTO_INCREMENT, text TEXT NOT NULL, author TEXT NOT NULL, PRIMARY KEY (id))")
            .execute(&pool)
            .await;
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    
    let token = env::var("DISCORD_TOKEN").expect("Expected a DISCORD_TOKEN in the environment");
    let database_url = env::var("DATABASE_URL").expect("Expected a DATABASE_URL in the environment");

    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create database connection pool");

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILDS;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<DbPool>(pool);
    }

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
