use crate::config;
use futures::StreamExt;
use irc_parser::Message;
use thiserror::Error;
use tokio::net::TcpStream;
use tokio::prelude::*;
use tokio::sync::mpsc;
use tokio_util::codec::{FramedRead, LinesCodec};

#[derive(Error, Debug)]
pub enum TwitchError {
    #[error("TCP error: {0}")]
    TcpError(#[from] tokio::io::Error),

    #[error("Channel send error: {0}")]
    ChannelSendError(#[from] mpsc::error::SendError<String>),

    #[error("Channel receive error: {0}")]
    ChannelReceiveError(#[from] mpsc::error::RecvError),
}

pub struct Twitch {
    pub config: config::Twitch,
    pub read: mpsc::Receiver<String>,
    pub write: mpsc::Sender<String>,
}

impl Twitch {
    pub async fn connect(twitch_config: config::Twitch) -> Result<Twitch, TwitchError> {
        let mut stream = TcpStream::connect("irc.chat.twitch.tv:6667").await?;

        stream
            .write(b"CAP REQ :twitch.tv/tags twitch.tv/commands\r\n")
            .await?;
        stream
            .write(format!("PASS {}\r\n", twitch_config.bot_oauth.to_lowercase()).as_bytes())
            .await?;
        stream
            .write(format!("NICK {}\r\n", twitch_config.bot_username.to_lowercase()).as_bytes())
            .await?;
        stream
            .write(format!("JOIN #{}\r\n", twitch_config.channel.to_lowercase()).as_bytes())
            .await?;

        let (r, mut w) = io::split(stream);

        let (ws, mut wr) = mpsc::channel(100);
        let (rs, rr) = mpsc::channel(100);

        // Clone for use in spawned tasks
        let mw = ws.clone();
        let mut keep_alive = ws.clone();

        let mut stream = FramedRead::new(r, LinesCodec::new());

        let tw = Twitch {
            config: twitch_config,
            read: rr,
            write: ws,
        };

        tokio::spawn(async move {
            while let Some(result) = stream.next().await {
                match result {
                    Ok(line) => {
                        let mut message_writer = mw.clone();
                        let mut message_sender = rs.clone();

                        tokio::spawn(async move {
                            if let Ok(msg) = Message::parse(&line) {
                                match msg.command {
                                    Some("PING") => {
                                        if let Some(arr) = &msg.params {
                                            if let Err(e) = message_writer
                                                .send(format!("PONG :{}\r\n", arr[0]))
                                                .await
                                            {
                                                println!(
                                                    "Got an error trying to send PONG message\n{}",
                                                    e
                                                );
                                            }
                                        }
                                    }
                                    Some("PRIVMSG") => {
                                        if let Some(params) = msg.params {
                                            message_sender
                                                .send(params[1].to_owned())
                                                .await
                                                .unwrap();
                                        }
                                    }
                                    _ => (),
                                }
                            }
                        });
                    }
                    Err(e) => println!("Got an error: {}", e),
                }
            }
        });

        tokio::spawn(async move {
            while let Some(msg) = wr.recv().await {
                //println!("trying to send {}", msg);
                w.write_all(msg.as_bytes()).await.unwrap();
            }
        });

        // Keepalive
        tokio::spawn(async move {
            loop {
                tokio::time::delay_for(std::time::Duration::from_secs(120)).await;
                //println!("Sending PING message");
                keep_alive
                    .send("PING :tmi.twitch.tv\r\n".to_string())
                    .await
                    .unwrap();
            }
        });

        Ok(tw)
    }

    pub async fn send_message(&mut self, channel: &str, message: &str) -> Result<(), TwitchError> {
        &self
            .write
            .send(format!("PRIVMSG #{} :{}\r\n", channel, message))
            .await?;
        Ok(())
    }
}
