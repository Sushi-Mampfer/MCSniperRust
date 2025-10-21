use std::{
    collections::{HashMap, VecDeque},
    thread,
};

use chrono::{DateTime, Duration, Utc};
use regex::Regex;
use reqwest::{
    blocking::ClientBuilder,
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    redirect,
};
use serde_json::{json, Value};
use tauri::{http::HeaderMap, window::Color, Emitter};
use urlencoding::{decode, encode};

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

    fn auth(user: &str, passwd: &str) -> Result<String, String> {
        let Ok(client) = ClientBuilder::new().cookie_store(true).build() else {
            return Err("Failed to initialize client for auth!".to_string());
        };
        let Ok(res) = client.get("https://login.live.com/oauth20_authorize.srf?client_id=000000004C12AE6F&redirect_uri=https://login.live.com/oauth20_desktop.srf&scope=service::user.auth.xboxlive.com::MBI_SSL&display=touch&response_type=token&locale=en").send() else {
            return Err("Failed to send initial request for auth!".to_string())        };

        let val_regex = Regex::new("value=\"(.+?)\"").unwrap();
        let url_post_regex = Regex::new("urlPost:'(.+?)'").unwrap();

        let Ok(res) = res.text() else {
            return Err("Failed to extract text from initial request for auth!".to_string());
        };

        let value = if let Some(caps) = val_regex.captures(&res) {
            caps[1].to_owned()
        } else {
            return Err("Failed to find value in initial request for auth!".to_string());
        };

        let url_post = if let Some(caps) = url_post_regex.captures(&res) {
            caps[1].to_owned()
        } else {
            return Err("Failed to find url in initial request for auth!".to_string());
        };

        let data = format!(
            "login={}&loginfmt={}&passwd={}&PPFT={}",
            encode(user),
            encode(user),
            encode(passwd),
            encode(&value)
        );

        let Ok(res) = client
            .post(&url_post)
            .body(data)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .send()
        else {
            return Err("Failed to send second request for auth!".to_string());
        };

        let binding = res.url().clone();
        let url = binding.as_str();

        if url == url_post {
            return Err("invalid credentials, no redirect".to_string());
        };

        let Ok(body) = res.text() else {
            return Err("Failed to get text from secondary request!".to_string());
        };

        if body.contains("Sign in to") {
            return Err("Invalid credentials, sign in to".to_string());
        };

        if body.contains("Help us protect your account") {
            return Err("2fa is enabled, which is not supported now".to_string());
        };

        if !url.contains("access_token") {
            return Err("Invalid credentials, no access_token in redirect".to_string());
        };

        let Some(params) = url.split("#").nth(1) else {
            return Err("Failed to parse redirect url".to_string());
        };

        let mut access_token = "".to_string();
        let mut refresh_token = "".to_string();

        for i in params.split("&") {
            let mut pair = i.split("=");
            let Some(key) = pair.next() else {
                return Err(format!("Something is wrong with the redirect url: {}", url));
            };
            let Some(value) = pair.next() else {
                return Err(format!("Something is wrong with the redirect url: {}", url));
            };
            if key == "access_token" {
                if let Ok(value) = decode(value) {
                    access_token = value.to_string()
                } else {
                    return Err(format!("Something is wrong with the access_token: {}", url));
                };
            } else if key == "refresh_token" {
                if let Ok(value) = decode(value) {
                    refresh_token = value.to_string()
                } else {
                    return Err(format!(
                        "Something is wrong with the refresh_token: {}",
                        url
                    ));
                };
            }
        }

        if access_token.is_empty() {
            return Err(format!("access_token is missing: {}", url));
        }

        if refresh_token.is_empty() {
            return Err(format!("refresh_token is missing: {}", url));
        }

        let body = json!({
            "Properties": {
                "Authmethod": "RPS",
                "Sitename": "user.auth.xboxlive.com",
                "Rpsticket": access_token,
            },
            "Relyingparty": "http://auth.xboxlive.com",
            "Tokentype": "JWT",
        });

        let mut headers = HeaderMap::new();
        headers.append("Content-Type", "application/json".parse().unwrap());
        headers.append("Accept", "application/json".parse().unwrap());
        headers.append("x-xbl-contract-version", "1".parse().unwrap());

        let Ok(res) = client
            .post("https://user.auth.xboxlive.com/user/authenticate")
            .headers(headers)
            .json(&body)
            .send()
        else {
            return Err("Failed to send request for xbox live!".to_string());
        };

        let Ok(body) = res.json::<Value>() else {
            return Err("Failed to parse request body for xbox live!".to_string());
        };

        todo!()
    }

    pub fn parse_list(accs: Vec<String>) -> Option<VecDeque<Account>> {
        let mut accounts = VecDeque::new();
        for (i, acc) in accs.iter().enumerate() {
            if i != 0 {
                thread::sleep(std::time::Duration::from_secs(21));
            }
            let mut split = acc.split(':');
            let user = split.next()?.to_owned();
            let passwd = split.next()?.to_owned();
            /* log(
                "INFO",
                Color::from((255, 255, 0)),
                format!("Signing in to {}.",),
            ); */
            if let Some(account) = Account::new(user, passwd) {
                accounts.push_back(account);
            }
        }
        Some(accounts)
    }
}
