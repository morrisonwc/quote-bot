use serenity::{
  async_trait,
  model::{channel::Message, gateway::Ready},
  prelude::*,
};
use sqlx::mysql::{MySqlPool, MySqlPoolOptions};
use dotenv::dotenv;
use rand::seq::SliceRandom;
use std::env;

struct DbPool;

impl TypeMapKey for DbPool {
  type Value = MySqlPool;
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
        // Handle !quote Name
        let name = parts[1].trim();
        let quotes = sqlx::query!("SELECT text, author FROM quotes WHERE author LIKE ?"("%{}%", name))
            .fetch_all(&pool)
            .await
            .expect("Failed to fetch quotes");

        if let Some(quote) = quotes.choose(&mut rand::thread_rng()) {
          Some(format!("\"{}\" - {}", quote.text, quote.author))
        } else {
          Some(format!("No quotes found for \"{}\".", name))
        }
      } else {
        // Handle !quote (random)
        let quotes = sqlx::query!("SELECT text, author FROM quotes")
            .fetch_all(&pool)
            .await
            .expect("Failed to fetch quotes");
        if let Some(quote) = quotes.choose(&mut rand::thread_rng()) {
          Some(format!("\"{}\" - {}", quote.text, quote.author))
        } else {
          Some("No quotes found in the database.".to_string())
        }
      };

      if let Some(quote_text) = quote_to_send {
         if let Err(why) = msg.channel_id.say(&ctx.http, quote_text).await {
             println!("Error sending message: {:?}", why);
         }
      }
    }
  }
  
