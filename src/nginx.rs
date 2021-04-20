use serde::Deserialize;

use crate::{config, error::Error};

#[derive(Deserialize, Debug)]
struct NginxRtmpStats {
    server: NginxRtmpServer,
}

#[derive(Deserialize, Debug)]
struct NginxRtmpServer {
    application: Vec<NginxRtmpApp>,
}

#[derive(Deserialize, Debug)]
struct NginxRtmpApp {
    name: String,
    live: NginxRtmpLive,
}

#[derive(Deserialize, Debug)]
struct NginxRtmpLive {
    stream: Option<Vec<NginxRtmpStream>>,
}

#[derive(Deserialize, Debug)]
struct NginxRtmpStream {
    name: String,
    bw_video: u32,
}

pub async fn get_rtmp_bitrate(config: &config::Rtmp) -> Result<Option<u32>, Error> {
    let res = reqwest::get(&config.url).await?;

    if res.status() != reqwest::StatusCode::OK {
        return Err(Error::RtmpDown("Can't connect to RTMP stats".to_owned()));
    }

    let text = res.text().await?;
    let parsed: NginxRtmpStats = quick_xml::de::from_str(&text)?;

    let filter: Option<NginxRtmpStream> = parsed
        .server
        .application
        .into_iter()
        .filter_map(|x| {
            if x.name == config.application {
                x.live.stream
            } else {
                None
            }
        })
        .flatten()
        .filter(|x| x.name == config.key)
        .collect::<Vec<NginxRtmpStream>>()
        .pop();

    Ok(match filter {
        Some(stream) => Some(stream.bw_video / 1024),
        None => None,
    })
}
