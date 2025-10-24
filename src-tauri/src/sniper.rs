use std::{
    collections::VecDeque,
    sync::mpsc::{channel, TryRecvError},
    thread::{self, sleep},
    time::Duration,
};

use reqwest::Proxy;

use tauri::{window::Color, Emitter, Manager};

use crate::{
    account::Account,
    app_handle, get_ratelimit, get_thread_status,
    log::{alert, log},
    set_ratelimit, set_thread_status,
};

#[tauri::command]
pub fn stop() {
    set_thread_status(false);
    log("STOPPED", Color::from((255, 0, 0)), "Sniper stopped!");
    alert("Sniper stopped!");
    app_handle().emit("stop", true).unwrap();
}

#[tauri::command]
pub fn start(claim: String, accounts: Vec<String>, proxies: Vec<String>, name: String) -> bool {
    thread::spawn(move || snipe(name, accounts, claim, proxies));
    set_thread_status(true);
    true
}

fn snipe(name: String, accounts: Vec<String>, claim: String, proxies: Vec<String>) {
    thread::sleep(Duration::from_secs(1));

    let mut proxy_list = VecDeque::new();

    for p in proxies {
        let Ok(proxy) = Proxy::all(p.clone()) else {
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

    let accounts = accounts
        .chunks((accounts.len() / proxy_list.len()).max(1))
        .map(|accs| accs.to_vec());

    let mut threads = Vec::new();

    for (i, accs) in accounts.enumerate() {
        let proxy = proxy_list[i].clone();
        threads.push(thread::spawn(move || {
            let mut out = Vec::new();
            for (i, acc) in accs.iter().enumerate() {
                if i != 0 {
                    sleep(Duration::from_secs(21));
                }

                let mut split = acc.split(":");
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
                if let Some(acc) = Account::new(user.to_owned(), pass.to_owned(), proxy.clone()) {
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
    let mut accounts_num = accounts.len() as u32;

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
    let mut claim = match Account::new(user.to_string(), pass.to_string(), None) {
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
    match claim.check_change_eligibility() {
        Some(b) => {
            if !b {
                log(
                    "ERROR",
                    Color::from((255, 0, 0)),
                    "Claimer can't namechange!",
                );
                alert("Claimer can't namechange!");
                app_handle().emit("stop", true).unwrap();
                return;
            }
        }
        None => {
            log(
                "ERROR",
                Color::from((255, 0, 0)),
                "Failed to check if claimer can change name!",
            );
            alert("Failed to check if claimer can change name!");
            app_handle().emit("stop", true).unwrap();
            return;
        }
    }
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

    let mut rl = calculate_ratelimit(accounts_num, proxy_list.len() as u32);

    loop {
        if !get_thread_status() {
            return;
        }
        if get_ratelimit() {
            sleep(Duration::from_secs(300));
            set_ratelimit(false);
        }
        loop {
            match rx_death.try_recv() {
                Ok(_) => {
                    accounts_num -= 1;
                    rl = calculate_ratelimit(accounts_num, proxy_list.len() as u32);
                }
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

        let mut account = match accounts.pop_front() {
            Some(acc) => acc,
            _ => {
                log("ERROR", Color::from((255, 0, 0)), "Couldn't pop account!");
                alert("Couldn't pop account!");
                app_handle().emit("stop", true).unwrap();
                return;
            }
        };
        let proxy = match proxy_list.pop_front() {
            Some(p) => p,
            _ => {
                log("ERROR", Color::from((255, 0, 0)), "Couldn't pop proxy!");
                alert("Couldn't pop proxy!");
                app_handle().emit("stop", true).unwrap();
                return;
            }
        };

        let tx_acc_pass = tx_acc.clone();
        let tx_death_pass = tx_death.clone();
        let proxy_pass = proxy.clone();
        let name_pass = name.clone();
        let claim_pass = claim.clone();

        thread::spawn(move || {
            match account.check(name_pass.clone(), proxy_pass.clone()) {
                Ok(available) => {
                    if available {
                        claim_pass.claim(name_pass, proxy_pass.clone());
                        app_handle().emit("stop", true).unwrap();
                        return;
                    } else {
                        log(
                            "INFO",
                            Color::from((255, 255, 0)),
                            &format!("{} not available yet, continuing", name_pass),
                        );
                    }
                }
                Err(err) => log("ERROR", Color::from((255, 0, 0)), &err),
            }
            match account.opt_reauth(proxy_pass) {
                Some(_) => match tx_acc_pass.send(account) {
                    Ok(_) => (),
                    Err(_) => log(
                        "ERROR",
                        Color::from((255, 0, 0)),
                        "Failed to send account back after use!",
                    ),
                },
                None => match tx_death_pass.send(()) {
                    Ok(_) => (),
                    Err(_) => log(
                        "ERROR",
                        Color::from((255, 0, 0)),
                        "Failed to send account death info!",
                    ),
                },
            }
        });

        proxy_list.push_back(proxy);
        if claim.opt_reauth(None).is_none() {
            log(
                "ERROR",
                Color::from((255, 0, 0)),
                "Failed to reauth claimer!",
            );
            alert("Failed to reauth claimer!");
            app_handle().emit("stop", true).unwrap();
            return;
        }
        sleep(Duration::from_secs_f32(rl));
    }
}

fn calculate_ratelimit(accounts: u32, proxies: u32) -> f32 {
    let account_ratelimit = 10.0 / accounts as f32;
    let rl = account_ratelimit.max(3.0 / proxies as f32);
    log(
        "INFO",
        Color::from((255, 255, 0)),
        &format!("Calculated new ratelimit to be {}", rl),
    );
    rl
}
