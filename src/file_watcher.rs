use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use time::{Date, Month, Time, UtcDateTime};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio::sync::mpsc;
use crate::event::Event;
use std::collections::HashMap;
use std::{path::Path};
use std::env;

pub async fn file_watcher(sender: mpsc::Sender<Event>) -> Result<(), Box<dyn std::error::Error>> {
    let (tx, mut rx) = mpsc::channel(1024);
    let mut watcher: RecommendedWatcher =
    Watcher::new(move |res| {
        let _ = tx.blocking_send(res);
    }, notify::Config::default())?;

    let path = &env::var("WATCH_FILE")?;
    let path = Path::new(&path);
    watcher.watch(path, RecursiveMode::NonRecursive)?;

    println!("Watching...");

    let mut last_connected_time = HashMap::new();

    let mut last_file_size: u64 = {
        let mut file = File::open(path).await?;
        file.seek(std::io::SeekFrom::End(0)).await?
    };
    let minecraft_channel_id = u64::from_str_radix(
        &env::var("MINECRAFT_CHANNEL_ID")?, 
        10
    )?;

    
    loop {
        let Some(res) = rx.recv().await else {
            return Err("Watch failed".into());
        };
        match res {
            Ok(event) => {
                match event.kind {
                    EventKind::Modify(_) |
                    EventKind::Create(_) |
                    EventKind::Remove(_) => {
                        println!("File changed");

                        let (vec, offset) = read_file(path, last_file_size).await?;
                        println!("{:?}", vec);

                        for s in vec {
                            let sp = s.split(' ').collect::<Vec<_>>();
                            println!("{:?}", sp);
                            if sp.len() < 5 {
                                continue;
                            }

                            let unixtime = get_timestamp(sp[0], sp[1]).unwrap_or(0);
                            println!("시간: {:?}", UtcDateTime::from_unix_timestamp(unixtime as i64));

                            if sp[4] == "connected:" {
                                println!("Connect");

                                let nickname = sp[5];
                                let nickname = &nickname[0..nickname.len()-1];
                                let nickname = String::from(nickname);

                                last_connected_time.insert(nickname.clone(), unixtime);
                                sender.send(Event::PlayerConnected { 
                                    channel_id: minecraft_channel_id.into(), 
                                    nickname,
                                }).await?;
                            }
                            else if sp[4] == "disconnected:" {
                                println!("Disconnect");
                                let nickname = sp[5];
                                let nickname = &nickname[0..nickname.len()-1];
                                let nickname = String::from(nickname);
                                let mut played_time = 0;

                                if let Some(&timestamp) = last_connected_time.get(&nickname) {
                                    played_time = unixtime - timestamp;
                                }
                                sender.send(Event::PlayerDisconnected { 
                                    channel_id: minecraft_channel_id.into(), 
                                    nickname,
                                    played_time
                                }).await?;
                            }
                        }

                        last_file_size = offset;
                    }
                    _ => {}
                }
            }
            Err(e) => eprintln!("Watch error: {:?}", e)
        }
    }
}

// date: "[2026-01-17" 꼴의 문자열, time: "12:04:27:437" 꼴의 문자열에서
// u64 timestamp 값을 뽑아온다.
// 입력으로 들어오는 문자열은 UTC+00으로 가정한다.
pub fn get_timestamp(date: &str, time: &str) -> Result<u64, Box<dyn std::error::Error>> {
    let date = &date[1..];
    let date = date.split('-').collect::<Vec<_>>();

    let time = time.split(':').collect::<Vec<_>>();

    let year = i32::from_str_radix(date[0], 10)?;
    let month = u8::from_str_radix(date[1], 10)?;
    let day = u8::from_str_radix(date[2], 10)?;
    let hour = u8::from_str_radix(time[0], 10)?;
    let minute = u8::from_str_radix(time[1], 10)?;
    let second = u8::from_str_radix(time[2], 10)?;

    let date = Date::from_calendar_date(
        year, 
        Month::January.nth_next(month-1), 
        day)?
    .with_hms(hour, minute, second)?;

    return Ok(date.as_utc().unix_timestamp() as u64);
}

async fn read_file(path: &Path, offset: u64) -> Result<(Vec<String>, u64), Box<dyn std::error::Error>> {
    let mut file = File::open(path).await?;

    file.seek(std::io::SeekFrom::Start(offset)).await?;

    let mut vec: Vec<String> = vec![];
    let mut str_buf = vec![];
    let mut buf = [0u8; 1024];
    loop {
        let sz = file.read(&mut buf).await?;

        if sz == 0 {
            if str_buf.len() > 0 {
                vec.push(String::from_utf8_lossy(&str_buf).into());
            }
            break;
        }

        for i in 0..sz {
            if buf[i] == b'\n' {
                if str_buf.len() > 0 {
                    vec.push(String::from_utf8_lossy(&str_buf).into());
                }

                str_buf.clear();
            }
            else {
                str_buf.push(buf[i]);
            }
        }
    }

    let now_offset = file.seek(std::io::SeekFrom::Current(0)).await?;

    return Ok((vec, now_offset));
}

enum LogEvent {
    Connected(u64, String),
    Disconnected(u64, String),
}

// 추후 저 위쪽에 중복되는 코드를 이쪽으로 합칠 예정
fn parse_file(vec: &Vec<String>) -> Vec<LogEvent> {
    let mut res = vec![];

    for s in vec {
        let sp = s.split(' ').collect::<Vec<_>>();
        println!("{:?}", sp);
        if sp.len() < 5 {
            continue;
        }

        let unixtime = get_timestamp(sp[0], sp[1]).unwrap_or(0);
        println!("시간: {:?}", UtcDateTime::from_unix_timestamp(unixtime as i64));

        if sp[4] == "connected:" {
            println!("Connect");
            let nickname = sp[5];
            let nickname = &nickname[0..nickname.len()-1];
            let nickname = String::from(nickname);
            //let send_msg = format!("{}님이 접속했습니다.", nickname);
            res.push(LogEvent::Connected(
                unixtime,
                nickname,
            ));
        }
        else if sp[4] == "disconnected:" {
            println!("Disconnect");
            let nickname = sp[5];
            let nickname = &nickname[0..nickname.len()-1];
            let nickname = String::from(nickname);
            res.push(LogEvent::Disconnected(
                unixtime,
                nickname,
            ));
        }
    }
    return res;
}