use chrono::prelude::*;
use rodio::Source;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::Cursor;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;
use structopt::StructOpt;
#[derive(StructOpt, Debug, Clone)]
struct Cli {
    work_minutes: u32,
    break_minutes: u32,
    task_name: Option<String>,
}

impl Cli {
    fn get_work_minutes(&self) -> u32 {
        return self.work_minutes * 60;
    }
    fn get_break_minutes(&self) -> u32 {
        return self.break_minutes * 60;
    }
}

enum SoundType {
    Start,
    Done,
}

const START_SOUND: &'static [u8] = include_bytes!("../sound/start.ogg");
const DONE_SOUND: &'static [u8] = include_bytes!("../sound/done.ogg");

fn play_sound(t: SoundType) {
    let device = rodio::default_output_device().unwrap();
    let start = Cursor::new(START_SOUND);
    let end = Cursor::new(DONE_SOUND);
    let source = rodio::Decoder::new(match t {
        SoundType::Start => start,
        SoundType::Done => end,
    })
    .unwrap();
    rodio::play_raw(&device, source.convert_samples());
}

fn write_to_file(start_time: DateTime<Local>, end_time: DateTime<Local>) {
    let mut home = dirs::home_dir().unwrap();
    home.push("pom.txt");
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .read(true)
        .open(home)
        .expect("Unable to read file!");
    let formatted_start = format!("{}", start_time.format("%H:%M:%S"));
    let formatted_end = format!("{}", end_time.format("%H:%M:%S, %a %Y-%m-%d"));
    let combined = format!("{} -- {}", formatted_start, formatted_end);
    file.write_all(combined.as_bytes())
        .expect("Could not write to file");
    file.write_all(b"\n").expect("could not write to file");
}
fn create_start() -> impl Send + Fn() -> () {
    let start_time = Local::now();
    move || {
        play_sound(SoundType::Done);
        write_to_file(start_time, Local::now());
    }
}
fn main() {
    let args = Cli::from_args();
    let mut set_close_handler = false;
    let end = Arc::new(Mutex::new(create_start()));
    loop {
        let data = Arc::clone(&end);
        {
            //update timestamp
            play_sound(SoundType::Start);
            let mut updated = data.lock().unwrap();
            *updated = create_start();
        }
        let dup = create_start();

        if set_close_handler == false {
            set_close_handler = true;
            //can only be called once
            ctrlc::set_handler(move || {
                let end = data.lock().unwrap();
                end();
                std::process::exit(0);
            })
            .expect("Error setting Ctrl-C handler");
        }
        println!("Work now!");
        sleep(Duration::from_secs(args.get_work_minutes().into()));
        dup();
        println!("Break time!");
        sleep(Duration::from_secs(args.get_break_minutes().into()));
    }
}
