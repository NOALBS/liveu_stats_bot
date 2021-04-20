use crate::{
    config,
    error::Error,
    liveu::{self, Liveu},
    nginx,
};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use twitch_irc::{
    login::StaticLoginCredentials, message, ClientConfig, TCPTransport, TwitchIRCClient,
};

pub struct Twitch {
    client: TwitchIRCClient<TCPTransport, StaticLoginCredentials>,
    liveu: Liveu,
    liveu_boss_id: String,
    config: config::Config,
    timeout: Arc<AtomicBool>,
}

impl Twitch {
    pub fn run(
        config: config::Config,
        liveu: Liveu,
        liveu_boss_id: String,
    ) -> tokio::task::JoinHandle<()> {
        let twitch_credentials = StaticLoginCredentials::new(
            config.twitch.bot_username.to_lowercase(),
            Some(config.twitch.bot_oauth.to_owned()),
        );
        let twitch_config = ClientConfig::new_simple(twitch_credentials);
        let (mut incoming_messages, client) =
            TwitchIRCClient::<TCPTransport, StaticLoginCredentials>::new(twitch_config);

        client.join(config.twitch.channel.to_lowercase());

        tokio::spawn(async move {
            let t = Self {
                client,
                liveu,
                liveu_boss_id,
                config,
                timeout: Arc::new(AtomicBool::new(false)),
            };

            while let Some(message) = incoming_messages.recv().await {
                t.handle_chat(message).await;
            }
        })
    }

    async fn handle_chat(&self, message: message::ServerMessage) {
        let timeout = self.timeout.clone();
        if timeout.load(Ordering::Acquire) {
            return;
        }

        match message {
            message::ServerMessage::Notice(msg) => {
                if msg.message_text == "Login authentication failed" {
                    panic!("Twitch authentication failed");
                }
            }
            message::ServerMessage::Privmsg(msg) => {
                let command = msg
                    .message_text
                    .split_ascii_whitespace()
                    .next()
                    .unwrap_or("")
                    .to_string();

                if self.config.twitch.commands.contains(&command) {
                    let cooldown = self.config.twitch.command_cooldown;

                    tokio::spawn(async move {
                        timeout.store(true, Ordering::Release);
                        tokio::time::sleep(tokio::time::Duration::from_secs(cooldown as u64)).await;
                        timeout.store(false, Ordering::Release);
                    });

                    if let Ok(lu_msg) = self.generate_liveu_message().await {
                        let _ = self.client.say(msg.channel_login, lu_msg).await;
                    }
                }
            }
            _ => {}
        };
    }

    async fn generate_liveu_message(&self) -> Result<String, Error> {
        let interfaces: Vec<liveu::Interface> = self
            .liveu
            .get_unit_custom_names(&self.liveu_boss_id)
            .await?;

        if interfaces.is_empty() {
            return Ok("LiveU Offline :(".to_string());
        }

        let mut message = String::new();
        let mut total_bitrate = 0;

        for interface in interfaces.iter() {
            message = message + &format!("{}: {} Kbps, ", interface.port, interface.uplink_kbps);
            total_bitrate += interface.uplink_kbps;
        }

        if total_bitrate == 0 {
            return Ok("LiveU Online and Ready".to_string());
        }

        message += &format!("Total LRT: {} Kbps", total_bitrate);

        if let Some(rtmp) = &self.config.rtmp {
            if let Ok(Some(bitrate)) = nginx::get_rtmp_bitrate(&rtmp).await {
                message += &format!(", RTMP: {} Kbps", bitrate);
            };
        }

        Ok(message)
    }
}
