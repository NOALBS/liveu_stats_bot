use twitch_irc::{login, TCPTransport, TwitchIRCClient};

use crate::{config, liveu};

#[derive(Debug, Clone)]
pub struct Monitor {
    pub client: TwitchIRCClient<TCPTransport, login::StaticLoginCredentials>,
    pub config: config::Config,
    pub liveu: liveu::Liveu,
    pub boss_id: String,
}

impl Monitor {
    pub fn run(&self) {
        let modems = self.clone();
        tokio::spawn(async move { modems.monitor_modems().await });

        let battery = self.clone();
        tokio::spawn(async move { battery.monitor_battery().await });
    }

    pub async fn monitor_modems(&self) {
        let mut current_modems = Vec::new();
        let mut ignore = false;

        for interface in self
            .liveu
            .get_unit_custom_names(&self.boss_id, self.config.custom_port_names.clone())
            .await
            .unwrap()
        {
            current_modems.push(interface.port);
        }

        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

            if !self.liveu.is_streaming(&self.boss_id).await {
                ignore = true;
                continue;
            }

            let mut current = Vec::new();
            let mut new_modems = Vec::new();

            for interface in self
                .liveu
                .get_unit_custom_names(&self.boss_id, self.config.custom_port_names.clone())
                .await
                .unwrap()
            {
                // we got a new interface
                if !current_modems.contains(&interface.port) {
                    // println!("New modem {}", interface.port);
                    new_modems.push(interface.port.to_owned());
                    current_modems.push(interface.port.to_owned());
                }

                current.push(interface.port);
            }

            // check diff between current and prev
            let mut removed_modems = Vec::new();
            for modem in current_modems.iter() {
                if !current.contains(&modem) {
                    // println!("Removed modem {}", modem);
                    removed_modems.push(modem.to_owned());
                }
            }

            for rem in removed_modems.iter() {
                let index = current_modems.iter().position(|m| m == rem).unwrap();
                current_modems.swap_remove(index);
            }

            let message = Self::generate_modems_message(new_modems, removed_modems);

            if !ignore && !message.is_empty() {
                let _ = self
                    .client
                    .say(
                        self.config.twitch.channel.to_owned(),
                        "LiveU: ".to_string() + &message,
                    )
                    .await;
            }

            if ignore {
                ignore = false;
            }
        }
    }

    fn generate_modems_message(new_modems: Vec<String>, removed_modems: Vec<String>) -> String {
        let mut message = String::new();

        if !new_modems.is_empty() {
            let a = if new_modems.len() > 1 { "are" } else { "is" };

            message += &format!("{} {} now connected.", new_modems.join(", "), a);
        }

        if !removed_modems.is_empty() {
            let a = if removed_modems.len() > 1 {
                "have"
            } else {
                "has"
            };

            message += &format!("{} {} disconnected.", removed_modems.join(", "), a);
        }

        message
    }

    pub async fn monitor_battery(&self) {
        let mut prev = liveu::Battery {
            connected: false,
            percentage: 255,
            run_time_to_empty: 0,
            discharging: false,
            charging: false,
        };

        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

            if !self.liveu.is_streaming(&self.boss_id).await {
                continue;
            }

            let battery = if let Ok(battery) = self.liveu.get_battery(&self.boss_id).await {
                if prev.percentage == 255 {
                    prev = battery.clone();
                }

                battery
            } else {
                continue;
            };

            if !battery.charging && battery.discharging && !prev.discharging {
                let _ = self
                    .client
                    .say(
                        self.config.twitch.channel.to_owned(),
                        "LiveU: RIP PowerBank / Cable Disconnected".to_string(),
                    )
                    .await;
            }

            if battery.charging && !battery.discharging && !prev.charging {
                let _ = self
                    .client
                    .say(
                        self.config.twitch.channel.to_owned(),
                        "LiveU: Now charging".to_string(),
                    )
                    .await;
            }

            if battery.percentage < 100
                && !battery.charging
                && !battery.discharging
                && (prev.charging || prev.discharging)
            {
                let _ = self
                    .client
                    .say(
                        self.config.twitch.channel.to_owned(),
                        "LiveU: Too hot to charge".to_string(),
                    )
                    .await;
            }

            if battery.percentage == 100
                && !battery.charging
                && !battery.discharging
                && prev.charging
                && !prev.discharging
            {
                let _ = self
                    .client
                    .say(
                        self.config.twitch.channel.to_owned(),
                        "LiveU: Fully charged".to_string(),
                    )
                    .await;
            }

            for percentage in &self.config.liveu.monitor.battery_notification {
                self.battery_percentage_message(*percentage, &battery, &prev)
                    .await;
            }

            prev = battery;
        }
    }

    pub async fn battery_percentage_message(
        &self,
        percentage: u8,
        current: &liveu::Battery,
        prev: &liveu::Battery,
    ) {
        if current.percentage == percentage && prev.percentage > percentage {
            let message = format!(
                "LiveU: Internal battery is at {}% and is {} charging.",
                percentage,
                if current.charging { "" } else { "not" }
            );

            let _ = self
                .client
                .say(self.config.twitch.channel.to_owned(), message)
                .await;
        }
    }
}
