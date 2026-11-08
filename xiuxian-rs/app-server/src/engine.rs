use std::{
    collections::VecDeque,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
};
use xiuxian_app_server_protocol::{ClickerState, Command, LogEntry, Snapshot};
use xiuxian_core::{
    clicker, native,
    session::{Event, Session},
};
pub(crate) struct Engine {
    stop: Arc<AtomicBool>,
    busy: Arc<Mutex<()>>,
    tx: mpsc::Sender<Event>,
    rx: mpsc::Receiver<Event>,
    clicker: clicker::Clicker,
    running: bool,
    stopping: bool,
    closing: bool,
    status: String,
    map: String,
    success: u64,
    failure: u64,
    connected: bool,
    logs: VecDeque<LogEntry>,
    log_id: u64,
}
impl Engine {
    pub(crate) fn new() -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let busy = Arc::new(Mutex::new(()));
        let (tx, rx) = mpsc::channel();
        Self {
            clicker: clicker::Clicker::new(busy.clone(), stop.clone()),
            stop,
            busy,
            tx,
            rx,
            running: false,
            stopping: false,
            closing: false,
            status: "已停止".into(),
            map: "待识别".into(),
            success: 0,
            failure: 0,
            connected: false,
            logs: VecDeque::new(),
            log_id: 0,
        }
    }
    fn log(&mut self, message: String) {
        let mut t = unsafe { std::mem::zeroed() };
        unsafe { windows_sys::Win32::System::SystemInformation::GetLocalTime(&mut t) };
        self.log_id += 1;
        self.logs.push_front(LogEntry {
            id: self.log_id,
            time: format!("{:02}:{:02}:{:02}", t.wHour, t.wMinute, t.wSecond),
            message,
        });
        self.logs.truncate(100);
    }
    pub(crate) fn command(&mut self, command: Command) -> Result<(), String> {
        if self.closing && !matches!(command, Command::Shutdown) {
            return Err("正在退出".into());
        }
        match command {
            Command::StartDispatch => {
                if self.stopping {
                    return Err("请等待本轮操作停止".into());
                }
                if !self.running {
                    self.stop.store(false, Ordering::Relaxed);
                    self.running = true;
                    self.status = "正在监听".into();
                    self.log("立即检查小队返回提示".into());
                    let job = Session::new(self.stop.clone(), self.tx.clone(), self.busy.clone());
                    std::thread::spawn(move || job.run());
                }
            }
            Command::StopDispatch => {
                if self.running {
                    self.stopping = true;
                    self.status = "正在停止".into();
                    self.stop.store(true, Ordering::Relaxed);
                }
            }
            Command::SetClicker { enabled } => {
                self.clicker.enabled.store(enabled, Ordering::Relaxed);
                *self.clicker.status.lock().unwrap() = if enabled {
                    "等待鼠标指向游戏"
                } else {
                    "已停止"
                }
                .into();
            }
            Command::ConfigureClicker {
                interval,
                right,
                key,
            } => {
                if !(30..=60000).contains(&interval) || ![0x75, 0x76, 0x78].contains(&key) {
                    return Err("点击间隔应为 30～60000 毫秒，热键应为 F6、F7 或 F9".into());
                }
                if self.clicker.enabled.load(Ordering::Relaxed) {
                    return Err("请先停止连点再修改设置".into());
                }
                *self.clicker.settings.lock().unwrap() = clicker::Settings {
                    interval,
                    right,
                    key,
                };
            }
            Command::Shutdown => self.shutdown(),
        }
        Ok(())
    }
    pub(crate) fn poll(&mut self) {
        while let Ok(event) = self.rx.try_recv() {
            match event {
                Event::State(s) => {
                    if !self.stopping && !self.closing {
                        self.status = s;
                    }
                }
                Event::Map(s) => self.map = s,
                Event::Log(s) => self.log(s),
                Event::Success(n) => {
                    self.success += 1;
                    self.log(format!("本轮成功，重新派遣 {n} 支小队"));
                }
                Event::Failure(s) => {
                    self.failure += 1;
                    self.log(s);
                }
                Event::Stopped => {
                    self.running = false;
                    self.stopping = false;
                    self.status = "已停止".into();
                }
            }
        }
    }
    pub(crate) fn scan_game(&mut self) {
        self.connected = native::game().ok().flatten().is_some();
    }

    pub(crate) fn shutdown(&mut self) {
        if self.closing {
            return;
        }
        self.closing = true;
        self.clicker.stop();
        self.stop.store(true, Ordering::Relaxed);
        self.status = "完成当前操作后退出".into();
    }

    pub(crate) fn is_finished(&self) -> bool {
        self.closing && !self.running
    }

    pub(crate) fn snapshot(&self) -> Snapshot {
        let settings = *self.clicker.settings.lock().unwrap();
        Snapshot {
            dispatch_running: self.running,
            dispatch_stopping: self.stopping,
            dispatch_status: self.status.clone(),
            map: self.map.clone(),
            success: self.success,
            failure: self.failure,
            game_connected: self.connected,
            clicker: ClickerState {
                enabled: self.clicker.enabled.load(Ordering::Relaxed),
                count: self.clicker.count.load(Ordering::Relaxed),
                status: self.clicker.status.lock().unwrap().clone(),
                interval: settings.interval,
                right: settings.right,
                key: settings.key,
            },
            logs: self.logs.iter().cloned().collect(),
            closing: self.closing,
        }
    }
}
