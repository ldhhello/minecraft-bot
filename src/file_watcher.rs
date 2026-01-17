use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio::sync::mpsc;
use crate::event::Event;
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
                            if sp[4] == "connected:" {
                                println!("Connect");
                                let nickname = sp[5];
                                let nickname = &nickname[0..nickname.len()-1];
                                let send_msg = format!("{}님이 접속했습니다.", nickname);
                                sender.send(Event::SendChat { 
                                    channel_id: minecraft_channel_id.into(), 
                                    str: send_msg,
                                }).await?;
                            }
                            else if sp[4] == "disconnected:" {
                                println!("Disconnect");
                                let nickname = sp[5];
                                let nickname = &nickname[0..nickname.len()-1];
                                let send_msg = format!("{}님이 퇴장했습니다.", nickname);
                                sender.send(Event::SendChat { 
                                    channel_id: minecraft_channel_id.into(), 
                                    str: send_msg,
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

    Ok(())
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