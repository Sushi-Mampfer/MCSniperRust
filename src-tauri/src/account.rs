use std::{collections::VecDeque, thread};

use chrono::{DateTime, Duration, Utc};
use regex::Regex;
use reqwest::{
    blocking::ClientBuilder,
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
};
use tauri::{http::HeaderMap, window::Color, Emitter};

use crate::{
    app_handle,
    log::{alert, log},
};

#[derive(PartialEq, Clone, Debug)]
enum AccType {
    CLAIMED,
    UNCLAIMED,
}

#[derive(Debug)]
pub struct Account {
    token: String,
    user: String,
    passwd: String,
    acc_type: AccType,
    time: DateTime<Utc>,
}

impl Account {
    pub fn new(user: String, passwd: String) -> Option<Self> {
        log(
            "INFO",
            Color::from((255, 255, 0)),
            format!("Authenticating {}.", user).as_str(),
        );
        let token = match Self::auth(&user, &passwd) {
            Some(tok) => tok,
            _ => {
                log(
                    "ERROR",
                    Color::from((255, 0, 0)),
                    format!("Failed to authenticate {}.", user).as_str(),
                );
                return None;
            }
        };
        let acc_type = match Self::check_type(token.clone()) {
            Some(acc_type) => acc_type,
            _ => {
                log(
                    "SUCCESS",
                    Color::from((255, 0, 0)),
                    format!("Failed to authenticate {}.", user).as_str(),
                );
                return None;
            }
        };
        log(
            "SUCCESS",
            Color::from((0, 255, 0)),
            format!("Successfully authenticated {}.", user).as_str(),
        );
        Some(Account {
            token,
            user,
            passwd,
            acc_type,
            time: Utc::now() + Duration::hours(23),
        })
    }

    pub fn claim(self, username: String) -> Option<()> {
        let mut map = HeaderMap::new();
        map.insert(AUTHORIZATION, self.get_token().parse().ok()?);
        let claimer = ClientBuilder::new().default_headers(map).build().ok()?;
        if self.get_type() == AccType::UNCLAIMED {
            let data = format!(
                r#"{{
              "profileName" : "{}"
            }}"#,
                username
            );
            let res = match claimer
                .post("https://api.minecraftservices.com/minecraft/profile")
                .header(AUTHORIZATION, format!("Bearer {}", self.token))
                .header(ACCEPT, "application/json")
                .body(data)
                .send()
            {
                Ok(res) => res,
                _ => {
                    log(
                        "ERROR",
                        Color::from((255, 0, 0)),
                        "Failed to send claim request!",
                    );
                    alert("Failed to send claim request!");
                    app_handle().emit("stop", true).unwrap();
                    return None;
                }
            };
            match res.status().as_u16() {
                200 => {
                    log(
                        "SUCCESS",
                        Color::from((0, 255, 0)),
                        format!("Sniped {}!", username).as_str(),
                    );
                    alert(format!("Sniped {}!", username).as_str());
                    return Some(());
                }
                400 => {
                    log(
                        "Error",
                        Color::from((255, 0, 0)),
                        format!(
                            "Claiming failed, probably to slow, error text was: {}!",
                            res.text().unwrap()
                        )
                        .as_str(),
                    );
                }
                429 => {
                    log(
                        "Error",
                        Color::from((255, 0, 0)),
                        "Claiming failed because of ratelimit!",
                    );
                }
                _ => {
                    log(
                        "Error",
                        Color::from((255, 0, 0)),
                        format!(
                            "Claiming failed with status code {} and error text: {}!",
                            res.status().as_str(),
                            res.text().unwrap()
                        )
                        .as_str(),
                    );
                }
            }
        } else {
            let res = match claimer
                .put(format!(
                    "https://api.minecraftservices.com/minecraft/profile/name/{}",
                    username
                ))
                .header(AUTHORIZATION, format!("Bearer {}", self.token))
                .send()
            {
                Ok(res) => res,
                _ => {
                    log(
                        "ERROR",
                        Color::from((255, 0, 0)),
                        "Failed to send claim request!",
                    );
                    alert("Failed to send claim request!");
                    app_handle().emit("stop", true).unwrap();
                    return None;
                }
            };
            match res.status().as_u16() {
                200 => {
                    log(
                        "SUCCESS",
                        Color::from((0, 255, 0)),
                        format!("Sniped {}!", username).as_str(),
                    );
                    alert(format!("Sniped {}!", username).as_str());
                    return Some(());
                }
                403 => {
                    log(
                        "Error",
                        Color::from((255, 0, 0)),
                        format!(
                            "Claiming failed, probably to slow, error text was: {}!",
                            res.text().unwrap()
                        )
                        .as_str(),
                    );
                }
                429 => {
                    log(
                        "Error",
                        Color::from((255, 0, 0)),
                        "Claiming failed because of ratelimit!",
                    );
                }
                _ => {
                    log(
                        "Error",
                        Color::from((255, 0, 0)),
                        format!(
                            "Claiming failed with status code {} and error text: {}!",
                            res.status().as_str(),
                            res.text().unwrap()
                        )
                        .as_str(),
                    );
                }
            }
        }
        alert(format!("Failed to snipe {}.", username).as_str());
        None
    }

    pub fn get_token(&self) -> String {
        self.token.clone()
    }

    fn get_type(&self) -> AccType {
        self.acc_type.clone()
    }

    fn check_type(token: String) -> Option<AccType> {
        let client = ClientBuilder::new().build().ok()?;
        let res = client
            .get("https://api.minecraftservices.com/minecraft/profile")
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .send()
            .ok()?;
        if res.status().as_u16() == 200 {
            return Some(AccType::CLAIMED);
        }
        let res = client
            .get("https://api.minecraftservices.com/entitlements/mcstores")
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .send()
            .ok()?;
        if res.status().as_u16() == 200 {
            return Some(AccType::UNCLAIMED);
        }
        None
    }

    pub fn reauth(self) -> Option<Self> {
        if self.time > Utc::now() {
            return Some(self);
        }
        log(
            "INFO",
            Color::from((255, 255, 0)),
            format!("Reauthenticating {}.", self.user).as_str(),
        );
        let token = match Self::auth(self.user.as_str(), self.passwd.as_str()) {
            Some(tok) => tok,
            _ => {
                log(
                    "ERROR",
                    Color::from((255, 0, 0)),
                    format!("Failed to reauthenticate {}.", self.user).as_str(),
                );
                return None;
            }
        };
        log(
            "SUCCESS",
            Color::from((255, 0, 0)),
            format!("Successfully reauthenticated {}!", self.user).as_str(),
        );
        return Some(Self {
            token,
            user: self.user,
            passwd: self.passwd,
            acc_type: self.acc_type,
            time: Utc::now() + Duration::hours(23),
        });
    }

    fn auth(user: &str, passwd: &str) -> Option<String> {
        let client = ClientBuilder::new().cookie_store(true).build().ok()?;
        let data = client.get("https://login.live.com/oauth20_authorize.srf?client_id=000000004C12AE6F&redirect_uri=https://login.live.com/oauth20_desktop.srf&scope=service::user.auth.xboxlive.com::MBI_SSL&display=touch&response_type=token&locale=en").send().ok()?.text().ok()?;
        let re1 = Regex::new("value=\"(.+?)\"").ok()?;
        let re2 = Regex::new("urlPost:'(.+?)'").ok()?;
        let sfttag = re1.captures(data.as_str())?.get(1)?.as_str();
        let posturl = re2.captures(data.as_str())?.get(1)?.as_str();
        let res = client
            .post(posturl)
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(format!(
                "login={}&loginfmt={}&passwd={}&PPFT={}",
                user, user, passwd, sfttag
            ))
            .send()
            .ok()?;
        let url = res.url().as_str();
        let re = Regex::new("access_token=(.+?)&").ok()?;
        let ms_token = re.captures(url)?.get(1)?.as_str();

        let data = format!(
            r#"{{
            "Properties": {{
                "AuthMethod": "RPS",
                "SiteName": "user.auth.xboxlive.com",
                "RpsTicket": "{}"
            }},
            "RelyingParty": "http://auth.xboxlive.com",
            "TokenType": "JWT"
            }}"#,
            ms_token
        );
        let res = client
            .post("https://user.auth.xboxlive.com/user/authenticate")
            .header(CONTENT_TYPE, "application/json")
            .header(ACCEPT, "application/json")
            .body(data)
            .send()
            .ok()?
            .text()
            .ok()?;
        let data = res.as_str();
        let re = Regex::new("\"Token\":\"(.+?)\"(?s)").ok()?;
        let live_token = re.captures(data)?.get(1)?.as_str();

        let data = format!(
            r#"{{
            "Properties": {{
                "SandboxId": "RETAIL",
                "UserTokens": [
                    "{}"
                ]
            }},
            "RelyingParty": "rp://api.minecraftservices.com/",
            "TokenType": "JWT"
        }}"#,
            live_token
        );
        let res = client
            .post("https://xsts.auth.xboxlive.com/xsts/authorize")
            .header(CONTENT_TYPE, "application/json")
            .header(ACCEPT, "application/json")
            .body(data)
            .send()
            .ok()?
            .text()
            .ok()?;
        let data = res.as_str();
        let re_tok = Regex::new("\"Token\":\"(.+?)\"(?s)").ok()?;
        let re_uhs = Regex::new("\"uhs\":\"(.+?)\"(?s)").ok()?;
        let xsts_token = re_tok.captures(data)?.get(1)?.as_str();
        let uhs = re_uhs.captures(data)?.get(1)?.as_str();

        let data = format!(
            r#"{{
           "identityToken" : "XBL3.0 x={};{}",
           "ensureLegacyEnabled" : true
        }}"#,
            uhs, xsts_token
        );
        let res = client
            .post("https://api.minecraftservices.com/authentication/login_with_xbox")
            .header(CONTENT_TYPE, "application/json")
            .body(data)
            .send()
            .ok()?
            .text()
            .ok()?;
        let data = res.as_str();
        let re = Regex::new("\"access_token\" : \"(.+?)\"(?s)").ok()?;
        Some(re.captures(data)?.get(1)?.as_str().to_owned())
    }
}

pub fn parse_list(accs: Vec<String>) -> Option<VecDeque<Account>> {
    let mut accounts = VecDeque::new();
    for acc in accs {
        let mut split = acc.split(':');
        let user = split.next()?.to_owned();
        let passwd = split.next()?.to_owned();
        if let Some(account) = Account::new(user, passwd) {
            accounts.push_back(account);
        }
        thread::sleep(std::time::Duration::from_secs(20));
    }
    Some(accounts)
}
