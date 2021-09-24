use anyhow::{anyhow, bail, Context, Result};
use chrono::{Date, DateTime, Datelike, Duration, Local, Timelike, Weekday};
use clap::{App, Arg};
use fastrand::usize as random;
use git2::Repository;
use lazy_static::lazy_static;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};

mod font;

lazy_static! {
    static ref ONE_DAY: Duration = Duration::days(1);
    static ref TODAY: DateTime<Local> = Local::now();
    static ref FIFTY_TWO_WEEKS: Duration = Duration::days(1) * 7 * 52;
}

fn main() -> Result<()> {
    let app = App::new("GitHub Vanity")
        .arg(
            Arg::with_name("commits-lower")
                .takes_value(true)
                .short("l")
                .long("lower")
                .default_value("20"),
        )
        .arg(
            Arg::with_name("commits-upper")
                .takes_value(true)
                .short("u")
                .long("upper")
                .default_value("30"),
        )
        .arg(
            Arg::with_name("message")
                .takes_value(true)
                .short("m")
                .long("message")
                .required(true),
        )
        .arg(
            Arg::with_name("disable-invert")
                .takes_value(true)
                .short("i")
                .long("disable-invert")
                .takes_value(false),
        );

    let matches = app.get_matches();

    let commits_lower: usize = matches
        .value_of("commits-lower")
        .expect("commits-lower should be set")
        .parse()
        .context("Must be a positive number")?;
    let commits_upper: usize = matches
        .value_of("commits-upper")
        .expect("commits-upper should be set")
        .parse()
        .context("Must be a positive number")?;
    let disable_invert = match matches.value_of("disable-invert") {
        Some(_) => true,
        None => false,
    };

    let message = matches.value_of("message").expect("message should be set");

    if !message.is_ascii() {
        bail!("message must be ascii");
    }

    let repo = Repository::open(".").context("Could not open repo")?;

    let now = Local::now()
        .with_hour(12)
        .ok_or_else(|| anyhow!("Could not generalize time"))?;

    let mut date = get_last_sunday(now);
    debug_assert_eq!(date.weekday(), Weekday::Sun);
    date = date - *FIFTY_TWO_WEEKS;
    debug_assert_eq!(date.weekday(), Weekday::Sun);

    let mut date = MovingDate::new(date);

    let mut pixels = Vec::new();

    for _i in 0..7 {
        pixels.push(Pixel {
            date: date.next()?,
            fill: Fill::Background,
        });
    }

    for char in message.chars() {
        let bools = font::SYMBOLS
            .get(&char)
            .ok_or_else(|| anyhow!("char '{}' does not have a matching font char", char))?;

        for b in bools {
            pixels.push(Pixel {
                date: date.next()?,
                fill: if *b {
                    Fill::Character
                } else {
                    Fill::Background
                },
            })
        }
    }

    for pixel in pixels {
        let mut fill_this_date = match pixel.fill {
            Fill::Character => false,
            Fill::Background => true,
        };
        if disable_invert {
            fill_this_date = !fill_this_date
        }

        if fill_this_date {
            let commits_num = random(commits_lower..=commits_upper);

            for _i in 0..commits_num {
                commit_on_date(pixel.date)?
            }
        }
    }
    if !disable_invert {
        let mut next_date = date.next().unwrap_or(now);
        while next_date < now {
            for _i in 0..random(commits_lower..=commits_upper) {
                commit_on_date(next_date)?
            }
            next_date = date.next().unwrap_or(now);
        }
    }

    println!("Done");
    Ok(())
}

fn get_last_sunday(from: DateTime<Local>) -> DateTime<Local> {
    let mut last_sunday = from;

    while last_sunday.weekday() != Weekday::Sun {
        last_sunday = last_sunday - *ONE_DAY;
    }
    last_sunday
}

struct Pixel {
    date: DateTime<Local>,
    fill: Fill,
}

enum Fill {
    Character,
    Background,
}

struct MovingDate(DateTime<Local>);
impl MovingDate {
    fn next(&mut self) -> Result<DateTime<Local>> {
        let ret = self.0;
        if ret > *TODAY {
            bail!("Message was too long, extended past today")
        }
        self.0 = ret + *ONE_DAY;
        Ok(ret)
    }
    fn new(first: DateTime<Local>) -> MovingDate {
        MovingDate(first)
    }
}

fn commit_on_date(date: DateTime<Local>) -> Result<()> {
    static COUNT: AtomicUsize = AtomicUsize::new(0);
    let count = COUNT.fetch_add(1, Relaxed);
    if count % 200 == 0 {
        dbg!(count);
    }
    let mut c = Command::new("git");
    c.arg("commit")
        .arg("--allow-empty")
        .arg("--message")
        .arg("Done in vain")
        .arg(&format!("--date={}", date.format("%c")));

    if !c.output().context("Committing failed")?.status.success() {
        bail!("Committing failed");
    };
    Ok(())
}
