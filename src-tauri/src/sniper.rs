use std::{
    collections::VecDeque,
    sync::mpsc::{channel, TryRecvError},
    thread::{self, sleep},
    time::Duration,
};

use reqwest::{blocking::ClientBuilder, header::HeaderMap, Proxy};

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
pub fn start(accounts: Vec<String>, claim: String, name: String, proxies: Vec<String>) -> bool {
    thread::spawn(move || snipe(name, accounts, claim, proxies));
    set_thread_status(true);
    true
}

fn snipe(name: String, accounts: Vec<String>, claim: String, proxies: Vec<String>) {
    thread::sleep(Duration::from_secs(1));

    let mut proxy_list = VecDeque::new();

    for p in proxies {
        let Ok(proxy) = Proxy::all(p) else {
            log(
                "ERROR",
                Color::from((225, 0, 0)),
                &format!("{} is not a valid proxy.", p),
            );
            continue;
        };
        proxy_list.push_back(Some(proxy));
    }
    proxy_list.push_back(None);

    log(
        "SUCCESS",
        Color::from((0, 255, 0)),
        &format!("Added {} proxies.", proxy_list.len()),
    );

    let accounts = accounts.chunks((accounts.len() / proxies.len()).max(1));

    let threads = Vec::new();

    for (i, accs) in accounts.enumerate() {
        let proxy = proxy_list[i];
        threads.push(thread::spawn(|| {
            let mut out = Vec::new();
            for (i, acc) in accs.iter().enumerate() {
                if i != 0 {
                    sleep(Duration::from_secs(21));
                }

                let split = acc.split(":");
                let Some(user) = split.next() else {
                    log(
                        "ERROR",
                        Color::from((255, 0, 0)),
                        &format!("{} isn't a valid account!", i),
                    );
                    continue;
                };
                let Some(pass) = split.next() else {
                    log(
                        "ERROR",
                        Color::from((255, 0, 0)),
                        &format!("{} isn't a valid account!", i),
                    );
                    continue;
                };
                if let Some(acc) = Account::new(user.to_owned(), pass.to_owned(), proxy) {
                    out.push(acc);
                };
            }
            if proxy.is_none() {
                sleep(Duration::from_secs(21));
            }
            out
        }));
    }

    let mut accounts = VecDeque::new();

    for i in threads {
        match i.join() {
            Ok(accs) => accounts.append(&mut VecDeque::from(accs)),
            Err(_) => log(
                "ERROR",
                Color::from((255, 0, 0)),
                "A thread to sign in accounts failed.",
            ),
        }
    }
    let mut accounts_num = accounts.len();

    if accounts_num == 0 {
        log("ERROR", Color::from((255, 0, 0)), "No working accounts!");
        alert("No working accounts!");
        app_handle().emit("stop", true).unwrap();
        return;
    }

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
    let claim = match Account::new(user.to_string(), pass.to_string(), None) {
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
    let (tx_death, rx_death) = channel::<()>();
    let (tx_acc, rx_acc) = channel::<Account>();

    loop {
        if !get_thread_status() {
            return;
        }
        loop {
            match rx_death.try_recv() {
                Ok(_) => accounts_num -= 1,
                Err(TryRecvError::Empty) => break,
                _ => {
                    log(
                        "ERROR",
                        Color::from((255, 0, 0)),
                        "Account death receiver died!",
                    );
                    alert("Account death receiver died!");
                    app_handle().emit("stop", true).unwrap();
                    return;
                }
            }
        }

        if accounts_num == 0 {
            log("ERROR", Color::from((255, 0, 0)), "No working accounts!");
            alert("No working accounts!");
            app_handle().emit("stop", true).unwrap();
            return;
        }

        loop {
            match rx_acc.try_recv() {
                Ok(acc) => accounts.push_back(acc),
                Err(TryRecvError::Empty) => break,
                _ => {
                    log("ERROR", Color::from((255, 0, 0)), "Account receiver died!");
                    alert("Account receiver died!");
                    app_handle().emit("stop", true).unwrap();
                    return;
                }
            }
        }

        let account = match accounts.pop_front() {
            Some(token) => token,
            _ => {
                log("ERROR", Color::from((255, 0, 0)), "Couldn't pop account!");
                alert("Couldn't pop account!");
                app_handle().emit("stop", true).unwrap();
                return;
            }
        };

        thread::spawn(|| {});
        sleep(dur);
        /* let account = match account.reauth() {
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
        accounts.push_back(account); */
    }
}

fn calculate_delay(accounts: u32, proxies: u32) -> u32 {}
fn calculate_ratelimit(accounts: u32, proxies: u32) -> f32 {
    let account_ratelimit = 10.0 / accounts as f32;
    account_ratelimit.max(3.0 / proxies as f32)
}
