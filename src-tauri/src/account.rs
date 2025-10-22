use std::{collections::VecDeque, thread};

use chrono::{DateTime, Duration, Utc};
use regex::Regex;
use reqwest::{
    blocking::ClientBuilder,
    header::{ACCEPT, AUTHORIZATION},
    Proxy,
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
    refresh_token: String,
    user: String,
    passwd: String,
    acc_type: AccType,
    time: DateTime<Utc>,
}

impl Account {
    pub fn new(user: String, passwd: String, proxy: Option<Proxy>) -> Option<Self> {
        log(
            "INFO",
            Color::from((255, 255, 0)),
            format!("Authenticating {}.", user).as_str(),
        );
        let (token, refresh_token) = match Self::auth(&user, &passwd, proxy.clone()) {
            Ok(tokens) => tokens,
            Err(msg) => {
                log(
                    "ERROR",
                    Color::from((255, 0, 0)),
                    &format!("Failed to authenticate {}: {}", user, msg),
                );
                return None;
            }
        };
        let acc_type = match Self::check_type(token.clone(), proxy) {
            Some(acc_type) => acc_type,
            _ => {
                log(
                    "ERROR",
                    Color::from((255, 0, 0)),
                    format!("Failed to get account type for {}.", user).as_str(),
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
            refresh_token,
            user,
            passwd,
            acc_type,
            time: Utc::now() + Duration::hours(23),
        })
    }

    pub fn claim(self, username: String, proxy: Option<Proxy>) -> Option<()> {
        let mut map = HeaderMap::new();
        map.insert(
            AUTHORIZATION,
            format!("Bearer {}", self.token).parse().unwrap(),
        );

        let claimer = if let Some(proxy) = proxy {
            ClientBuilder::new()
                .default_headers(map)
                .proxy(proxy)
                .build()
                .ok()?
        } else {
            ClientBuilder::new().default_headers(map).build().ok()?
        };
        if self.get_type() == AccType::UNCLAIMED {
            let data = format!(
                r#"{{
              "profileName" : "{}"
            }}"#,
                username
            );
            let res = match claimer
                .post("https://api.minecraftservices.com/minecraft/profile")
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

    fn check_type(token: String, proxy: Option<Proxy>) -> Option<AccType> {
        let client = if let Some(proxy) = proxy {
            ClientBuilder::new().proxy(proxy).build().ok()?
        } else {
            ClientBuilder::new().build().ok()?
        };

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

    pub fn opt_reauth(&mut self, proxy: Option<Proxy>) {
        if self.time > Utc::now() {
            return;
        }
        self.reauth(proxy);
    }

    pub fn reauth(&mut self, proxy: Option<Proxy>) -> Result<(), String> {
        let Ok(client) = (if let Some(proxy) = proxy {
            ClientBuilder::new().cookie_store(true).proxy(proxy).build()
        } else {
            ClientBuilder::new().cookie_store(true).build()
        }) else {
            return Err("Failed to initialize client for reauth!".to_string());
        };

        let body = format!("client_id=000000004C12AE6F&grant_type=refresh_token&refresh_token={}&scope=service::user.auth.xboxlive.com::MBI_SSL", encode(&self.refresh_token));

        let Ok(res) = client
            .post("https://login.live.com/oauth20_token.srf")
            .header("Content-Type", "application/x-www-form-urlencode")
            .body(body)
            .send()
        else {
            return Err("Failed to send request for new tokens!".to_string());
        };

        let Ok(body) = res.json::<Value>() else {
            return Err("Failed to parse request for new tokens!".to_string());
        };

        let Some(access_token) = body.get("access_token").and_then(|v| v.as_str()) else {
            return Err("No access_token in response!".to_string());
        };

        let Some(refresh_token) = body.get("refresh_token").and_then(|v| v.as_str()) else {
            return Err("No refresh_token in response!".to_string());
        };

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

        let Some(uhs) = body
            .get("DisplayClaims")
            .and_then(|v| v.get("xui"))
            .and_then(|v| v.get(0))
            .and_then(|v| v.get("uhs"))
            .and_then(|v| v.as_str())
        else {
            return Err("No uhs value found in xbox live auth body.".to_string());
        };

        let Some(token) = body.get("Token").and_then(|v| v.as_str()) else {
            return Err("No token found in xbox live auth body.".to_string());
        };

        let data = json!({
            "Properties": {
                "SandboxId": "RETAIL",
                "UserTokens": [
                    token
                ]
            },
            "RelyingParty": "rp://api.minecraftservices.com/",
            "TokenType": "JWT"
        });

        let Ok(res) = client
            .post("https://xsts.auth.xboxlive.com/xsts/authorize")
            .json(&data)
            .send()
        else {
            return Err("Failed to send request for xsts!".to_string());
        };

        let status = res.status().as_u16();

        let Ok(data) = res.json::<Value>() else {
            return Err("failed to parse request body for xsts!".to_string());
        };

        if status == 401 {
            let Some(error_code) = data.get("XErr").and_then(|v| v.as_i64()) else {
                return Err("Failed to parse error code for xsts!".to_string());
            };
            match error_code {
                2148916238 => return Err(
                    "Microsoft account belongs to someone under 18! add to family for this to work"
                        .to_string(),
                ),
                2148916233 => {
                    return Err("You have no xbox account! Sign up for one to continue.".to_string())
                }
                _ => {
                    let Some(error) = data.get("Message").and_then(|v| v.as_str()) else {
                        return Err("Failed to parse error for xsts!".to_string());
                    };
                    return Err(format!(
                        "Failed to got xsts token with error: {} {}",
                        error_code, error
                    ));
                }
            }
        }

        let Some(uhs_verify) = body
            .get("DisplayClaims")
            .and_then(|v| v.get("xui"))
            .and_then(|v| v.get(0))
            .and_then(|v| v.get("uhs"))
            .and_then(|v| v.as_str())
        else {
            return Err("No uhs value found in xbox live auth body.".to_string());
        };

        if uhs != uhs_verify {
            return Err("uhs tokens don't match!".to_string());
        }

        let Some(token) = body.get("Token").and_then(|v| v.as_str()) else {
            return Err("No token found in xbox live auth body.".to_string());
        };

        let body = json!({
            "identityToken" : format!("XBL3.0 x={};{}", uhs, token),
            "ensureLegacyEnabled" : true
        });

        let Ok(res) = client
            .post("https://api.minecraftservices.com/authentication/login_with_xbox")
            .json(&body)
            .send()
        else {
            return Err("Failed to send request for bearer!".to_string());
        };

        let Ok(data) = res.json::<Value>() else {
            return Err("Failed to parse response for bearer!".to_string());
        };

        let Some(bearer) = data.get("access_token").and_then(|v| v.as_str()) else {
            return Err("Failed to extract bearer!".to_string());
        };

        self.token = bearer.to_string();
        self.refresh_token = refresh_token.to_string();

        Ok(())
    }

    fn auth(user: &str, passwd: &str, proxy: Option<Proxy>) -> Result<(String, String), String> {
        let Ok(client) = (if let Some(proxy) = proxy {
            ClientBuilder::new().cookie_store(true).proxy(proxy).build()
        } else {
            ClientBuilder::new().cookie_store(true).build()
        }) else {
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
                return Err("Something is wrong with the redirect url.".to_string());
            };
            let Some(value) = pair.next() else {
                return Err("Something is wrong with the redirect url.".to_string());
            };
            if key == "access_token" {
                if let Ok(value) = decode(value) {
                    access_token = value.to_string()
                } else {
                    return Err("Something is wrong with the access_token.".to_string());
                };
            } else if key == "refresh_token" {
                if let Ok(value) = decode(value) {
                    refresh_token = value.to_string()
                } else {
                    return Err("Something is wrong with the refresh_token.".to_string());
                };
            }
        }

        if access_token.is_empty() {
            return Err("access_token is missing.".to_string());
        }

        if refresh_token.is_empty() {
            return Err("refresh_token is missing.".to_string());
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

        let Some(uhs) = body
            .get("DisplayClaims")
            .and_then(|v| v.get("xui"))
            .and_then(|v| v.get(0))
            .and_then(|v| v.get("uhs"))
            .and_then(|v| v.as_str())
        else {
            return Err("No uhs value found in xbox live auth body.".to_string());
        };

        let Some(token) = body.get("Token").and_then(|v| v.as_str()) else {
            return Err("No token found in xbox live auth body.".to_string());
        };

        let data = json!({
            "Properties": {
                "SandboxId": "RETAIL",
                "UserTokens": [
                    token
                ]
            },
            "RelyingParty": "rp://api.minecraftservices.com/",
            "TokenType": "JWT"
        });

        let Ok(res) = client
            .post("https://xsts.auth.xboxlive.com/xsts/authorize")
            .json(&data)
            .send()
        else {
            return Err("Failed to send request for xsts!".to_string());
        };

        let status = res.status().as_u16();

        let Ok(data) = res.json::<Value>() else {
            return Err("failed to parse request body for xsts!".to_string());
        };

        if status == 401 {
            let Some(error_code) = data.get("XErr").and_then(|v| v.as_i64()) else {
                return Err("Failed to parse error code for xsts!".to_string());
            };
            match error_code {
                2148916238 => return Err(
                    "Microsoft account belongs to someone under 18! add to family for this to work"
                        .to_string(),
                ),
                2148916233 => {
                    return Err("You have no xbox account! Sign up for one to continue.".to_string())
                }
                _ => {
                    let Some(error) = data.get("Message").and_then(|v| v.as_str()) else {
                        return Err("Failed to parse error for xsts!".to_string());
                    };
                    return Err(format!(
                        "Failed to got xsts token with error: {} {}",
                        error_code, error
                    ));
                }
            }
        }

        let Some(uhs_verify) = body
            .get("DisplayClaims")
            .and_then(|v| v.get("xui"))
            .and_then(|v| v.get(0))
            .and_then(|v| v.get("uhs"))
            .and_then(|v| v.as_str())
        else {
            return Err("No uhs value found in xbox live auth body.".to_string());
        };

        if uhs != uhs_verify {
            return Err("uhs tokens don't match!".to_string());
        }

        let Some(token) = body.get("Token").and_then(|v| v.as_str()) else {
            return Err("No token found in xbox live auth body.".to_string());
        };

        let body = json!({
            "identityToken" : format!("XBL3.0 x={};{}", uhs, token),
            "ensureLegacyEnabled" : true
        });

        let Ok(res) = client
            .post("https://api.minecraftservices.com/authentication/login_with_xbox")
            .json(&body)
            .send()
        else {
            return Err("Failed to send request for bearer!".to_string());
        };

        let Ok(data) = res.json::<Value>() else {
            return Err("Failed to parse response for bearer!".to_string());
        };

        let Some(bearer) = data.get("access_token").and_then(|v| v.as_str()) else {
            return Err("Failed to extract bearer!".to_string());
        };

        Ok((bearer.to_string(), refresh_token))
    }
}
