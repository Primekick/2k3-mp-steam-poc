use std::ffi::c_char;
use std::io::BufRead;
use std::sync::{Arc, Mutex};
use std::thread;

use ansi_term::enable_ansi_support;
use anyhow::Result;
use async_trait::async_trait;
use colored::Colorize;
use lazy_static::lazy_static;
use steamworks::{AppId, Client, SingleClient, UserStatsReceived};
use time::Duration;

use crate::cmd::Cmd;
use crate::ffi::{maniacs, open_console};

mod ffi;
mod cmd;

toy_arms::internal::create_entrypoint!(axeman_setup);

const VAR_IN: i32 = 100;
const STRVAR_IN: i32 = 100;

const LOGO: &'static str = r#"
  /$$$$$$  /$$   /$$ /$$$$$$$$ /$$      /$$  /$$$$$$  /$$   /$$
 /$$__  $$| $$  / $$| $$_____/| $$$    /$$$ /$$__  $$| $$$ | $$
| $$  \ $$|  $$/ $$/| $$      | $$$$  /$$$$| $$  \ $$| $$$$| $$
| $$$$$$$$ \  $$$$/ | $$$$$   | $$ $$/$$ $$| $$$$$$$$| $$ $$ $$
| $$__  $$  >$$  $$ | $$__/   | $$  $$$| $$| $$__  $$| $$  $$$$
| $$  | $$ /$$/\  $$| $$      | $$\  $ | $$| $$  | $$| $$\  $$$
| $$  | $$| $$  \ $$| $$$$$$$$| $$ \/  | $$| $$  | $$| $$ \  $$
|__/  |__/|__/  |__/|________/|__/     |__/|__/  |__/|__/  \__/
"#;

pub struct Steamworks {
    client: Client,
    single: SingleClient,
}

impl Steamworks {
    pub fn new() -> Self {
        let (client, single) = Client::init_app(AppId(1781490)).expect("Couldn't initialize Steamworks API");
        Self { client, single }
    }
}

lazy_static! {
    pub static ref SW: Mutex<Steamworks> = Mutex::new(Steamworks::new());
}

#[no_mangle]
extern "C" fn process_command(cstr: *const c_char) {
    let cmd = rust_string!(cstr).parse::<Cmd>();
    if let Ok(cmd) = cmd {
        match cmd {
            Cmd::TriggerAchievement(ack_id) => {
                let sw = SW.lock().unwrap();
                let user_stats = sw.client.user_stats();
                user_stats.request_current_stats();
                let ack_name = ack_id.into_value();
                match user_stats.achievement(&ack_name).set() {
                    Ok(_) => {
                        sw.client.user_stats().store_stats().expect("Couldn't store achievements");
                    }
                    Err(_) => {
                        println!("Failed to set achievement");
                    }
                };
            }
            Cmd::InitSteam => {
                let sw = SW.lock().unwrap();
                let user_stats = sw.client.user_stats();
                user_stats.request_current_stats();
                if let Some(achievements) = user_stats.get_achievement_names() {
                    println!("{}", "Detected achievements:".bold().bright_yellow());
                    for ack in achievements {
                        println!("{}", ack.italic());
                    }
                    match user_stats.reset_all_stats(true) {
                        Ok(_) => {
                            sw.client.user_stats().store_stats().expect("Couldn't store achievements");
                        }
                        Err(_) => {
                            println!("Failed to set achievement");
                        }
                    }
                }
            }
            Cmd::RunCallbacks => {
                let sw = SW.lock().unwrap();
                sw.single.run_callbacks();
            }
        }
    }
}

fn axeman_setup() {
    open_console();
    enable_ansi_support().unwrap();
    colored::control::set_virtual_terminal(true).unwrap();
    println!("{}", LOGO.bright_blue());
    println!("{}", "Fetching achievements...".bold().bright_yellow());
    let sw = SW.lock().unwrap();
    let _ = sw.client.register_callback(|val: UserStatsReceived| {
        if val.result.is_ok() {
           println!("{}", "[UserStatsReceived callback]".italic());
        }
    });
}