use std::{thread, time::Duration};

use reqwest::{blocking::ClientBuilder, header::HeaderMap};

use tauri::{window::Color, Emitter, Manager};

use crate::{
    account::Account,
    app_handle, get_thread_status,
    log::{alert, log},
    set_thread_status,
};

#[tauri::command]
pub fn stop() {
    set_thread_status(false);
    log("STOPPED", Color::from((255, 0, 0)), "Sniper stopped!");
    alert("Sniper stopped!");
    app_handle().emit("stop", true).unwrap();
}

#[tauri::command]
pub fn start(accounts: Vec<String>, claim: String, name: String) -> bool {
    thread::spawn(move || snipe_slow(name, accounts, claim));
    set_thread_status(true);
    true
}

fn snipe_slow(name: String, accounts: Vec<String>, claim: String) {
    thread::sleep(Duration::from_secs(1));
    let mut accounts = match Account::parse_list(accounts) {
        Some(accs) => accs,
        _ => {
            log(
                "ERROR",
                Color::from((255, 0, 0)),
                "Failed to authenticate accounts!",
            );
            alert("Failed to authenticate accounts!");
            app_handle().emit("stop", true).unwrap();
            return;
        }
    };
    let claim = claim.split(":").collect::<Vec<&str>>();
    let user = match claim.get(0) {
        Some(user) => user,
        _ => {
            log(
                "ERROR",
                Color::from((255, 0, 0)),
                "Failed to parse claim account!",
            );
            alert("Failed to parse claim account!");
            app_handle().emit("stop", true).unwrap();
            return;
        }
    };
    let pass = match claim.get(1) {
        Some(pass) => pass,
        _ => {
            log(
                "ERROR",
                Color::from((255, 0, 0)),
                "Failed to parse claim account!",
            );
            alert("Failed to parse claim account!");
            app_handle().emit("stop", true).unwrap();
            return;
        }
    };
    let claim = match Account::new(user.to_string(), pass.to_string()) {
        Some(acc) => acc,
        _ => {
            log(
                "ERROR",
                Color::from((255, 0, 0)),
                "Failed to authenticate claim account!",
            );
            alert("Failed to authenticate claim account!");
            app_handle().emit("stop", true).unwrap();
            return;
        }
    };
    let window = match app_handle().get_webview_window("main") {
        Some(w) => w,
        _ => {
            log("ERROR", Color::from((255, 0, 0)), "Failed to get window!");
            alert("Failed to get window!");
            app_handle().emit("stop", true).unwrap();
            return;
        }
    };
    match window.set_title(format!("MCSniperRust - {}", name).as_str()) {
        Ok(_) => {}
        _ => {
            log(
                "ERROR",
                Color::from((255, 0, 0)),
                "Failed to change window title!",
            );
            alert("Failed to change window title!");
            app_handle().emit("stop", true).unwrap();
            return;
        }
    };
    if accounts.len() < 4 {
        log(
            "ERROR",
            Color::from((255, 0, 0)),
            "Slow needs at least 4 working accounts!",
        );
        alert("Slow needs at least 4 working accounts!");
        app_handle().emit("stop", true).unwrap();
        return;
    }
    thread::sleep(Duration::from_secs(1));
    let url = format!(
        "https://api.minecraftservices.com/minecraft/profile/name/{}/available",
        name,
    );

    log(
        "STARTED",
        Color::from((0, 255, 0)),
        "Sniper started successfully!",
    );
    alert("Sniper started successfully!");

    log(
        name.to_uppercase().as_str(),
        Color::from((0, 255, 0)),
        format!("Sniping {}!", name).as_str(),
    );
    loop {
        let account = match accounts.pop_front() {
            Some(token) => token,
            _ => {
                log("ERROR", Color::from((255, 0, 0)), "Couldn't pop account!");
                alert("Couldn't pop account!");
                app_handle().emit("stop", true).unwrap();
                return;
            }
        };
        let account = match account.reauth() {
            Some(acc) => acc,
            _ => {
                if accounts.len() < 4 {
                    log(
                        "ERROR",
                        Color::from((255, 0, 0)),
                        "Not enought working accounts!",
                    );
                    alert("Not enought working accounts!");
                    app_handle().emit("stop", true).unwrap();
                    return;
                } else {
                    continue;
                }
            }
        };
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {}", account.get_token()).parse().unwrap(),
        );
        let client = match ClientBuilder::new().default_headers(headers).build() {
            Ok(client) => client,
            _ => {
                log("ERROR", Color::from((255, 0, 0)), "Couldn't create client!");
                alert("Couldn't create client!");
                app_handle().emit("stop", true).unwrap();
                return;
            }
        };

        for i in 1..31 {
            if !get_thread_status() {
                return;
            }
            let response = match client.get(&url).send() {
                Ok(res) => res,
                _ => {
                    log("ERROR", Color::from((255, 0, 0)), "Failed to make request!");
                    alert("Failed to make request!");
                    app_handle().emit("stop", true).unwrap();
                    return;
                }
            };
            if response.status().is_success() {
                let text = match response.text() {
                    Ok(text) => text,
                    _ => {
                        log("ERROR", Color::from((255, 0, 0)), "Error reading response!");
                        alert("Error reading response!");
                        app_handle().emit("stop", true).unwrap();
                        return;
                    }
                };
                if text.contains("DUPLICATE") {
                    log(
                        "DUPLICATE",
                        Color::from((255, 255, 0)),
                        format!("Request {}/30 was successfull!", i).as_str(),
                    );
                } else if text.contains("AVAILABLE") {
                    log(
                        "AVAILABLE",
                        Color::from((0, 255, 0)),
                        format!("Request {}/30 was successfull. Claiming now!", i).as_str(),
                    );
                    claim.claim(name);
                    app_handle().emit("stop", true).unwrap();
                    return;
                } else {
                    log(
                        "Not Allowed",
                        Color::from((255, 0, 0)),
                        format!("Request {}/30 was successfull!", i).as_str(),
                    );
                    return;
                }
            } else if response.status().as_u16() == 429 {
                log(
                    "ERROR",
                    Color::from((255, 0, 0)),
                    format!("Request {}/30 got ratelimited, sleeping 200 seconds!", i).as_str(),
                );
                thread::sleep(Duration::from_secs(200));
            } else if response.status().as_u16() == 503 {
                log(
                    "ERROR",
                    Color::from((255, 0, 0)),
                    format!("Request {}/30 failed because Mojang is down!", i).as_str(),
                );
            } else {
                log(
                    "ERROR",
                    Color::from((255, 0, 0)),
                    format!(
                        "Request {}/30 failed with status code {}!",
                        i,
                        response.status()
                    )
                    .as_str(),
                );
                return;
            }
            thread::sleep(Duration::from_secs(3));
        }
        accounts.push_back(account);
    }
}
