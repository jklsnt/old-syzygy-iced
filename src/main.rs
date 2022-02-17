use anyhow::Result;
use chrono::{offset::TimeZone, Local};
use inquire::{CustomType, DateSelect, Select, Text};
use std::path::Path;
use structopt::StructOpt;
use syzygy::*;

pub mod core;
pub use crate::core::create_task;

pub mod upcoming;
pub use crate::upcoming::*;

#[derive(StructOpt)]
struct Opt {
    #[structopt(subcommand)] // Note that we mark a field as a subcommand
    cmd: Command,
}

#[derive(StructOpt)]
enum Command {
    Projects,
    Inbox,
    Upcoming,
    Add,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    let mut w: Workspace;
    let path = format!("{}/.cache/syzygy.json", std::env::var("HOME")?);
    let p = Path::new(&path);
    dbg!(p, p.exists());
    if p.exists() {
        w = serde_json::from_str(&std::fs::read_to_string(p)?)?
    } else {
        w = Workspace::new();
        std::fs::File::create(p)?;
    };
    match opt.cmd {
        Command::Projects => {}
        Command::Inbox => {}
        Command::Upcoming => {}
        Command::Add => {
            create_task(&mut w);
        }
    }
    std::fs::write(p, serde_json::to_string(&w)?)?;
    Ok(())
}
