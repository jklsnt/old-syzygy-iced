use anyhow::Result;
use chrono::{offset::TimeZone, Local};
use inquire::{CustomType, DateSelect, Select, Text};
use syzygy::*;
use uuid::*;

fn get_day_tasks(day: chrono::NaiveDate) -> Vec<String> {
    let mut tasks = Vec::new();
    let path = format!("{}/.cache/syzygy.json", std::env::var("HOME").unwrap());
    let p = std::path::Path::new(&path);
    let w: Workspace = serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap();
    for i in w.tasks.values() {
        if let Some(d) = i.date.current() {
            if d.naive_local().date() == day {
                tasks.push(i.title.clone());
            }
        }
    }
    tasks
}

pub fn create_task(w: &mut Workspace) -> Result<Uuid> {
    let title = Text::new("What's the task?")
        .with_placeholder("Task title...")
        .prompt()?;
    let rule = Select::new("What recur rule?", vec!["Blank", "Deadline", "Constant"]).prompt()?;
    let recur: Box<dyn Recur> = match rule {
        "Deadline" => {
            let date = DateSelect::new("When is it due?")
                .with_accessor(&|day| {
                    let mut tasks = Vec::new();
                    for i in w.tasks.values() {
                        if let Some(d) = i.date.current() {
                            if d.naive_local().date() == day {
                                tasks.push(i.title.clone());
                            }
                        }
                    }
                    tasks
                })
                .with_default(Local::today().naive_local())
                .with_week_start(chrono::Weekday::Mon)
                .prompt()?;
            let time = chrono::NaiveTime::from_hms(8, 0, 0);
            let datetime = chrono::NaiveDateTime::new(date, time);
            Deadline::new(Local.from_local_datetime(&datetime).unwrap())
        }
        "Blank" => Blank::new(),
        _ => todo!(),
    };
    let mut deps: Vec<Box<dyn Dependency>> = Vec::new();
    loop {
        let dep_types = vec!["Date", "RelativeDate", "Direct", "Children", "Parent"];
        let dep = Select::new("Any dependencies?", dep_types)
            .with_help_message("Esc to skip!")
            .prompt_skippable()?;
        if let Some(d) = dep {
            match d {
                "Date" => {
                    let date = DateSelect::new("What day?")
                        .with_default(Local::today().naive_local())
                        .with_week_start(chrono::Weekday::Mon)
                        .prompt()?;
                    let time = chrono::NaiveTime::from_hms(8, 0, 0);
                    let datetime = chrono::NaiveDateTime::new(date, time);
                    deps.push(Date::new(Local.from_local_datetime(&datetime).unwrap()));
                }
                "RelativeDate" => {
                    let days = CustomType::<i64>::new("How many days from now?").prompt()?;
                    deps.push(RelativeDate::new(chrono::Duration::days(days)));
                }
                "Direct" => {
                    let task = Select::new(
                        "Which task?",
                        w.tasks.values().map(|i| i.title.clone()).collect(),
                    )
                    .prompt()?;
                    for i in w.tasks.keys() {
                        if w.tasks[&i].title == task {
                            deps.push(Direct::new(*i));
                        }
                    }
                }
                "Parent" => deps.push(Parent::new()),
                "Children" => deps.push(Children::new()),
                _ => (),
            }
        } else {
            break;
        };
    }
    Ok(w.add_task(&title, recur, deps))
}
