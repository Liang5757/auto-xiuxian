use std::{
    collections::VecDeque,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc,
    },
};
use xiuxian_app_server_protocol::{LogEntry, TaskState};
use xiuxian_core::session::{Event, Session};
pub(crate) struct Task {
    stop: Arc<AtomicBool>,
    pub interval: Arc<AtomicU64>,
    busy: Arc<Mutex<()>>,
    tx: mpsc::Sender<Event>,
    rx: mpsc::Receiver<Event>,
    farming: bool,
    pub running: bool,
    pub stopping: bool,
    pub status: String,
    pub map: String,
    pub success: u64,
    pub failure: u64,
    pub logs: VecDeque<LogEntry>,
    log_id: u64,
}
impl Task {
    pub fn new(farming: bool, busy: Arc<Mutex<()>>) -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            stop: Default::default(),
            interval: Arc::new(AtomicU64::new(30)),
            busy,
            tx,
            rx,
            farming,
            running: false,
            stopping: false,
            status: "已停止".into(),
            map: "待识别".into(),
            success: 0,
            failure: 0,
            logs: VecDeque::new(),
            log_id: 0,
        }
    }
    pub fn start(&mut self) -> Result<(), String> {
        if self.stopping {
            return Err("请等待本轮操作停止".into());
        }
        if self.running {
            return Ok(());
        }
        self.stop.store(false, Ordering::Relaxed);
        self.running = true;
        self.status = "正在监听".into();
        self.log(
            if self.farming {
                "立即检查作物成熟提示"
            } else {
                "立即检查小队返回提示"
            }
            .into(),
        );
        let mut job = Session::new(self.stop.clone(), self.tx.clone(), self.busy.clone());
        job.farming = self.farming;
        job.interval = self.interval.clone();
        std::thread::spawn(move || job.run());
        Ok(())
    }
    pub fn stop(&mut self) {
        if self.running {
            self.stopping = true;
            self.status = "正在停止".into();
            self.stop.store(true, Ordering::Relaxed);
        }
    }
    pub fn snapshot(&self) -> TaskState {
        TaskState {
            interval: self.interval.load(Ordering::Relaxed),
            running: self.running,
            stopping: self.stopping,
            status: self.status.clone(),
            map: self.map.clone(),
            success: self.success,
            failure: self.failure,
            logs: self.logs.iter().cloned().collect(),
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
    pub(crate) fn poll(&mut self) {
        while let Ok(event) = self.rx.try_recv() {
            match event {
                Event::State(s) => {
                    if !self.stopping {
                        self.status = s;
                    }
                }
                Event::Map(s) => self.map = s,
                Event::Log(s) => self.log(s),
                Event::Success(n) => {
                    self.success += 1;
                    self.log(if self.farming {
                        format!("本轮成功，补种 {n} 个空坑")
                    } else {
                        format!("本轮成功，重新派遣 {n} 支小队")
                    });
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
}
