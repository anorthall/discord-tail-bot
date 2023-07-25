use secrecy::ExposeSecret;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serenity::async_trait;
use serenity::model::gateway::{Activity, Ready};
use serenity::model::id::GuildId;
use serenity::prelude::*;

use discord_tail_bot::configuration::{get_configuration, Settings};
use discord_tail_bot::logchecker::do_log_check;

use env_logger::Env;

struct Handler {
    is_loop_running: AtomicBool,
    settings: Arc<Settings>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>) {
        log::info!("Cache built successfully!");
        ctx.set_activity(Activity::watching("caves.app")).await;

        let ctx = Arc::new(ctx);
        let last_seek = Arc::new(Mutex::new(0_u64));
        let settings = Arc::clone(&self.settings);

        if !self.is_loop_running.load(Ordering::Relaxed) {
            let ctx1 = Arc::clone(&ctx);

            tokio::spawn(async move {
                loop {
                    do_log_check(
                        Arc::clone(&settings),
                        Arc::clone(&ctx1),
                        Arc::clone(&last_seek),
                    )
                    .await;
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            });

            self.is_loop_running.swap(true, Ordering::Relaxed);
        }
    }

    async fn ready(&self, _ctx: Context, ready: Ready) {
        log::info!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    let env = Env::default().filter_or("LOG_LEVEL", "warn");
    env_logger::init_from_env(env);
    log::info!("Starting up...");

    let settings = get_configuration().expect("Failed to read configuration.");

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::GUILDS
        | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(&settings.discord_token.expose_secret(), intents)
        .event_handler(Handler {
            is_loop_running: AtomicBool::new(false),
            settings: Arc::new(settings),
        })
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        log::error!("Client error: {:?}", why);
    }
}
