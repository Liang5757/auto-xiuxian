use crate::native;
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

#[derive(Clone, Copy)]
pub struct Settings {
    pub interval: u64,
    pub right: bool,
    pub key: i32,
}
pub struct Clicker {
    pub enabled: Arc<AtomicBool>,
    pub count: Arc<AtomicU64>,
    pub status: Arc<Mutex<String>>,
    pub settings: Arc<Mutex<Settings>>,
    shutdown: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}
impl Clicker {
    pub fn new(busy: Arc<Mutex<()>>, dispatch_stop: Arc<AtomicBool>) -> Self {
        let mut this = Self {
            enabled: Arc::new(AtomicBool::new(false)),
            count: Arc::new(AtomicU64::new(0)),
            status: Arc::new(Mutex::new("已停止".into())),
            settings: Arc::new(Mutex::new(Settings {
                interval: 100,
                right: false,
                key: 0x75,
            })),
            shutdown: Arc::new(AtomicBool::new(false)),
            thread: None,
        };
        let enabled = this.enabled.clone();
        let count = this.count.clone();
        let status = this.status.clone();
        let settings = this.settings.clone();
        let shutdown = this.shutdown.clone();
        this.thread = Some(thread::spawn(move || {
            unsafe {
                windows_sys::Win32::UI::HiDpi::SetThreadDpiAwarenessContext(-4);
            }
            let mut held = false;
            let mut next = Instant::now();
            let mut scan = Instant::now();
            let mut game = None;
            while !shutdown.load(Ordering::Relaxed) {
                let config = *settings.lock().unwrap();
                let pressed = native::key(config.key);
                if pressed && !held {
                    let on = !enabled.fetch_xor(true, Ordering::Relaxed);
                    *status.lock().unwrap() = if on {
                        "等待鼠标指向游戏"
                    } else {
                        "已停止"
                    }
                    .into();
                }
                held = pressed;
                if native::key(0x1b) {
                    dispatch_stop.store(true, Ordering::Relaxed);
                }
                let now = Instant::now();
                if now >= scan {
                    game = native::game().ok().flatten();
                    scan = now + Duration::from_secs(2);
                }
                if enabled.load(Ordering::Relaxed) && now >= next {
                    next = now + Duration::from_millis(config.interval);
                    let message = if let Ok(_guard) = busy.try_lock() {
                        match game.map(|h| native::repeat_click(h, config.right)) {
                            Some(Ok(true)) => {
                                count.fetch_add(1, Ordering::Relaxed);
                                "正在连点".into()
                            }
                            Some(Err(error)) => {
                                enabled.store(false, Ordering::Relaxed);
                                error
                            }
                            _ => "等待鼠标指向游戏".into(),
                        }
                    } else {
                        "派遣检查中".into()
                    };
                    *status.lock().unwrap() = message;
                }
                thread::sleep(Duration::from_millis(10));
            }
        }));
        this
    }
}
impl Clicker {
    pub fn stop(&self) {
        self.enabled.store(false, Ordering::Relaxed);
        self.shutdown.store(true, Ordering::Relaxed);
    }
}
impl Drop for Clicker {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn continues_without_ui_updates_and_stops() {
        let busy = Arc::new(Mutex::new(()));
        let _guard = busy.lock().unwrap();
        let clicker = Clicker::new(busy.clone(), Arc::new(AtomicBool::new(false)));
        clicker.settings.lock().unwrap().key = 0;
        clicker.enabled.store(true, Ordering::Relaxed);
        let until = Instant::now() + Duration::from_secs(2);
        while *clicker.status.lock().unwrap() != "派遣检查中" && Instant::now() < until {
            thread::sleep(Duration::from_millis(10));
        }
        assert_eq!(*clicker.status.lock().unwrap(), "派遣检查中");
        assert_eq!(clicker.count.load(Ordering::Relaxed), 0);
        clicker.stop();
        assert!(!clicker.enabled.load(Ordering::Relaxed));
        drop(clicker);
    }
}
