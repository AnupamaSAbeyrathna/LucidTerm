use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::io::{Read, Write};
use std::thread;
use anyhow::Result;

fn main() -> Result<()> {
    let pty_system = native_pty_system();
    let pair = pty_system.openpty(PtySize {
        rows: 24,
        cols: 80,
        pixel_width: 0,
        pixel_height: 0,
    })?;

    // Spawn a shell on the slave side
    let cmd = CommandBuilder::new("/bin/bash"); // or "sh"
    let mut child = pair.slave.spawn_command(cmd)?;

    // We only need to operate with master here
    let mut reader = pair.master.try_clone_reader()?;
    let mut writer = pair.master.take_writer()?;

    // Reader thread: read from PTY and print to our stdout (later feed to parsgit branch
er)
    let rhandle = thread::spawn(move || {
        let mut buf = [0u8; 4096];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    print!("{}", String::from_utf8_lossy(&buf[..n]));
                }
                Err(e) => {
                    eprintln!("pty read error: {e:?}");
                    break;
                }
            }
        }
    });

    // Main thread: forward local stdin to the PTY
    let mut stdin = std::io::stdin();
    let mut ibuf = [0u8; 1024];
    loop {
        let n = stdin.read(&mut ibuf)?;
        if n == 0 { break; }
        writer.write_all(&ibuf[..n])?;
        writer.flush()?;
    }

    let _ = rhandle.join();
    let _ = child.wait()?;
    Ok(())
}
