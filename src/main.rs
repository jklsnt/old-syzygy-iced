use anyhow::Result;
use chrono::{Local, offset::TimeZone};
use std::path::Path;
use structopt::StructOpt;
use syzygy::*;

#[derive(StructOpt)]
struct Opt {
    #[structopt(subcommand)] // Note that we mark a field as a subcommand
    cmd: Command,
}

#[derive(StructOpt)]
enum Command {
    Projects,
    Inbox,
    Add,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    let mut w: Workspace;
    let path = format!("{}/.cache/syzygy.json", std::env::var("HOME").unwrap());
    let p = Path::new(&path);
    if p.exists() {
        w = serde_json::from_str(&std::fs::read_to_string(p)?)?
    } else {
        w = Workspace::new();
	std::fs::File::create(p)?;
    };
    match opt.cmd {
        Command::Projects => {}
        Command::Inbox => {}
        Command::Add => {
            let title = inquire::Text::new("What's the task?")
                .with_placeholder("Task title...")
                .prompt()
                .unwrap();
            let rule =
                inquire::Select::new("What recur rule?", vec!["Blank", "Deadline", "Constant"])
                    .prompt()
                    .unwrap();
            let recur: Box<dyn Recur> = match rule {
                "Deadline" => {
                    let date = inquire::DateSelect::new("When is it due?")
			.with_default(Local::today().naive_local())                        
                        .with_week_start(chrono::Weekday::Mon)
                        .prompt()
                        .unwrap();
                    let time = chrono::NaiveTime::from_hms(8, 0, 0);
                    let datetime = chrono::NaiveDateTime::new(date, time);
                    Deadline::new(Local.from_local_datetime(&datetime).unwrap())
                }
                "Blank" => Blank::new(),
                _ => todo!(),
            };
            let mut deps: Vec<Box<dyn Dependency>> = Vec::new();
            loop {
                let dep = inquire::Select::new(
                    "Any dependencies?",
                    vec!["Date", "RelativeDate", "Direct", "Children", "Parent"],
                )
                .with_help_message("Esc to skip!")
                .prompt_skippable()
                .unwrap();
                if let Some(d) = dep {
                    match d {
                        "Date" => {
                            let date = inquire::DateSelect::new("What day?")
                                .with_default(Local::today().naive_local())
                                .with_week_start(chrono::Weekday::Mon)
                                .prompt()
                                .unwrap();
                            let time = chrono::NaiveTime::from_hms(8, 0, 0);
                            let datetime = chrono::NaiveDateTime::new(date, time);
                            deps.push(Date::new(Local.from_local_datetime(&datetime).unwrap()));
                        },
			"RelativeDate" => {
			    let days = inquire::CustomType::<i64>::new("How many days from now?").prompt().unwrap();
			    deps.push(RelativeDate::new(chrono::Duration::days(days)));
			},
			"Direct" => {
			    let task = inquire::Select::new(
				"Which task?",
				w.tasks.values().map(|i| i.title.clone()).collect(),
			    ).prompt().unwrap();			    
			    for i in w.tasks.keys() {
				if w.tasks[&i].title == task {
				    deps.push(Direct::new(*i));
				}
			    }
			},
                        "Parent" => deps.push(Parent::new()),
                        "Children" => deps.push(Children::new()),
                        _ => (),
                    }
                } else {
                    break;
                };		
            }
	    w.add_task(&title, recur, deps);
        }
    }
    std::fs::write(p, serde_json::to_string(&w).unwrap())?;    
    Ok(())
}
