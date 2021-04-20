# LIVEU STATS BOT

Adds a chat command to check the current modems and bitrate on the liveu.

## How do i run this?

Just download the latest binary from [releases](https://github.com/715209/liveu_stats_bot/releases) and execute it.

## Config

Edit `config.json` to your own settings.

```JSON
{
    "liveu": {
        "email": "YOUR LIVEU EMAIL",
        "password": "YOUR LIVEU PASSWORD"
    },
    "twitch": {
        "botUsername": "TWITCH BOT USERNAME",
        "botOauth": "TWITCH BOT OAUTH",
        "twitchChannel": "YOUR TWITCH CHANNEL",
        "commands": ["!lustats", "!liveustats", "!lus"]
    },
    "nginx": {
        "stats": "http://localhost/stat",
        "application": "publish",
        "key": "live"
    }
}
```

The nginx part is optional

## Chat Commands
  
After running the app successfully you can use the follwing default commands in your chat to get stats:  
- !lustats
- !liveustats
- !lus  
  
  
You can also add, delete or change the commands to whatever you want just as long its in an array format in config.json on line 10;  
  
  
Example:
`        "commands": ["!ls", "!modemstats", "!connstats", "!lus", "mystats", "!whatever"],`  
  
  
## Possible Chat Command Results:  
  
If your LiveU is offline you'll see this in chat:
> ChatBot: LiveU Offline :(  
  
If your LiveU is online and ready you'll see this in chat:
> ChatBot: LiveU Online and Ready  
  
If your LiveU is online, streaming but not using NGINX you'll see this in chat:
> ChatBot: WiFi: 2453 Kbps, USB1: 2548 Kbps, USB2: 2328 Kbps, Ethernet: 2285 Kbps, Total LRT: 7000 Kbps
  
If your LiveU is online, streaming and you're using NGINX you'll see this in chat:
> ChatBot: WiFi: 2453 Kbps, USB1: 2548 Kbps, USB2: 2328 Kbps, Ethernet: 2285 Kbps, Total LRT: 7000 Kbps, RTMP: 6000 Kbps


`Please note: if one of your connections is offline it will NOT show up at all in the stats.`

## Credits:
[Cinnabarcorp (travelingwithgus)](https://twitch.tv/travelwithgus): Initial Idea, Feedback, Use Case, and Q&A Testing.

[B3ck](https://twitch.tv/b3ck): Feedback and Q&A Testing.
