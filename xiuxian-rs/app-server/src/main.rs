mod engine;

use engine::Engine;
use serde::Serialize;
use std::{
    io::{self, BufRead, Write},
    sync::mpsc,
    time::{Duration, Instant},
};
use xiuxian_app_server_protocol::Request;
fn output(value: &impl Serialize) -> io::Result<()> {
    let stdout = io::stdout();
    let mut writer = stdout.lock();
    serde_json::to_writer(&mut writer, value)?;
    writer.write_all(b"\n")?;
    writer.flush()
}
fn main() -> io::Result<()> {
    unsafe {
        windows_sys::Win32::UI::HiDpi::SetThreadDpiAwarenessContext(-4);
    }
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        for line in io::stdin().lock().lines() {
            match line {
                Ok(line) => {
                    if tx.send(Some(line)).is_err() {
                        return;
                    }
                }
                Err(_) => break,
            }
        }
        let _ = tx.send(None);
    });
    let mut engine = Engine::new();
    let mut previous = None;
    let mut output_open = true;
    let mut scan = Instant::now();
    loop {
        match rx.recv_timeout(Duration::from_millis(50)) {
            Ok(Some(line)) => match serde_json::from_str::<Request>(&line) {
                Ok(request) => {
                    let result = engine.command(request.command);
                    let _ = output(
                        &serde_json::json!({"type":"response","id":request.id,"ok":result.is_ok(),"error":result.err()}),
                    );
                }
                Err(error) => {
                    let _ = output(
                        &serde_json::json!({"type":"protocol_error","error":error.to_string()}),
                    );
                }
            },
            Ok(None) | Err(mpsc::RecvTimeoutError::Disconnected) => {
                engine.shutdown();
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
        engine.poll();
        if Instant::now() >= scan {
            engine.scan_game();
            scan = Instant::now() + Duration::from_secs(2);
        }
        let snapshot = engine.snapshot();
        if output_open && previous.as_ref() != Some(&snapshot) {
            if output(&serde_json::json!({"type":"state","state":snapshot})).is_err() {
                output_open = false;
                engine.shutdown();
            }
            previous = Some(snapshot);
        }
        if engine.is_finished() {
            break;
        }
    }
    Ok(())
}
