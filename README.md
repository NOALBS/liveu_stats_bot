# LIVEU STATS BOT

A chat bot that makes it easier to see the current status of the battery and modems.

## How do i run this?

Just download the latest binary from [releases](https://github.com/715209/liveu_stats_bot/releases) and execute it.

## Config

This config will be automatically generated upon running the binary and saved as `config.json`.

```JSON
{
    "liveu": {
        "email": "YOUR LIVEU EMAIL",
        "password": "YOUR LIVEU PASSWORD",
        "id": null,
        "monitor": {
            "battery": true,
            "batteryNotification": [
                99,
                50,
                10,
                5,
                1
            ],
            "modems": true
        }
    },
    "twitch": {
        "botUsername": "TWITCH BOT USERNAME",
        "botOauth": "TWITCH BOT OAUTH",
        "channel": "YOUR TWITCH CHANNEL",
        "adminUsers": ["b3ck", "travelwithgus"],
        "modOnly": true
    },
    "commands": {
        "cooldown": 5,
        "stats": ["!lustats", "!liveustats", "!lus"],
        "battery": ["!battery", "!liveubattery", "!lub"],
        "start": "!lustart",
        "stop": "!lustop",
        "restart": "!lurestart"
    },
    "rtmp": {
        "url": "http://localhost/stat",
        "application": "publish",
        "key": "live"
    },
    "customPortNames": {
        "ethernet": "ETH",
        "wifi": "WiFi",
        "usb1": "USB1",
        "usb2": "USB2",
        "sim1": "SIM1",
        "sim2": "SIM2"
    }
}
```

### Optional config settings

You can remove these settings from the config if you don't want them or replace them with `null`.

| Name            | Description                                                                         |
| --------------- | ----------------------------------------------------------------------------------- |
| id              | When using mutliple units you can set a default unit by using the bossid            |
| adminUsers      | A list of twitch usernames e.g. `["715209", "b3ck"]`                                |
| rtmp            | If you are using nginx you can also show the bitrate when using the `stats` command |
| customPortNames | Customize the port names                                                            |

## Chat Commands

After running the app successfully you can use the following default commands in your chat:

| Name    | Default command | Description                                    |
| ------- | --------------- | ---------------------------------------------- |
| stats   | !lus            | Shows the current connected modems and bitrate |
| battery | !lub            | Shows the current battery charge percentage    |
| start   | !lustart        | Starts the stream (not the unit)               |
| stop    | !lustop         | Stops the stream                               |
| restart | !lurestart      | Restarts the stream                            |

You can add, delete or change the commands to whatever you want in `config.json` under the `commands` section.

The start, stop and restart commands are only available to the channel owner or adminUsers.

## Give specific users access to all commands

Add the twitch username in adminUsers like this: `["715209", "b3ck"]`.

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
