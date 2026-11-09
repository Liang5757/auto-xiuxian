use crate::task::Task;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use xiuxian_app_server_protocol::{ClickerState, Command, Snapshot};
use xiuxian_core::{clicker, native};
pub(crate) struct Engine {
    dispatch: Task,
    farm: Task,
    escape: Arc<AtomicBool>,
    clicker: clicker::Clicker,
    connected: bool,
    closing: bool,
}
impl Engine {
    pub(crate) fn new() -> Self {
        let busy = Arc::new(Mutex::new(()));
        let escape = Arc::new(AtomicBool::new(false));
        Self {
            dispatch: Task::new(false, busy.clone()),
            farm: Task::new(true, busy.clone()),
            clicker: clicker::Clicker::new(busy, escape.clone()),
            escape,
            connected: false,
            closing: false,
        }
    }
    pub(crate) fn command(&mut self, command: Command) -> Result<(), String> {
        if self.closing && !matches!(command, Command::Shutdown) {
            return Err("正在退出".into());
        }
        match command {
            Command::StartBoth => {
                if self.dispatch.stopping || self.farm.stopping {
                    return Err("请等待当前操作停止".into());
                }
                self.dispatch.start()?;
                self.farm.start()?;
            }
            Command::StopBoth => {
                self.dispatch.stop();
                self.farm.stop();
            }
            Command::ConfigurePolling { farming, interval } => {
                if !(1..=86400).contains(&interval) {
                    return Err("检测间隔应为 1～86400 s 的整数".into());
                }
                let task = if farming {
                    &mut self.farm
                } else {
                    &mut self.dispatch
                };
                task.interval.store(interval, Ordering::Relaxed);
            }
            Command::StartFarm => self.farm.start()?,
            Command::StartDispatch => self.dispatch.start()?,
            Command::StopFarm => self.farm.stop(),
            Command::StopDispatch => self.dispatch.stop(),
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
        if self.escape.swap(false, Ordering::Relaxed) {
            self.dispatch.stop();
            self.farm.stop();
        }
        self.dispatch.poll();
        self.farm.poll();
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
        self.dispatch.stop();
        self.farm.stop();
    }

    pub(crate) fn is_finished(&self) -> bool {
        self.closing && !self.dispatch.running && !self.farm.running
    }

    pub(crate) fn snapshot(&self) -> Snapshot {
        let settings = *self.clicker.settings.lock().unwrap();
        Snapshot {
            dispatch_interval: self.dispatch.interval.load(Ordering::Relaxed),
            farm: self.farm.snapshot(),
            dispatch_running: self.dispatch.running,
            dispatch_stopping: self.dispatch.stopping,
            dispatch_status: self.dispatch.status.clone(),
            map: self.dispatch.map.clone(),
            success: self.dispatch.success,
            failure: self.dispatch.failure,
            game_connected: self.connected,
            clicker: ClickerState {
                enabled: self.clicker.enabled.load(Ordering::Relaxed),
                count: self.clicker.count.load(Ordering::Relaxed),
                status: self.clicker.status.lock().unwrap().clone(),
                interval: settings.interval,
                right: settings.right,
                key: settings.key,
            },
            logs: self.dispatch.logs.iter().cloned().collect(),
            closing: self.closing,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn master_switch_stops_both_and_polling_intervals_are_independent() {
        let busy = Arc::new(Mutex::new(()));
        let guard = busy.lock().unwrap();
        let mut engine = Engine::new();
        engine.dispatch = Task::new(false, busy.clone());
        engine.farm = Task::new(true, busy.clone());
        engine
            .command(Command::ConfigurePolling {
                farming: false,
                interval: 5,
            })
            .unwrap();
        engine
            .command(Command::ConfigurePolling {
                farming: true,
                interval: 60,
            })
            .unwrap();
        assert_eq!(engine.snapshot().dispatch_interval, 5);
        assert_eq!(engine.snapshot().farm.interval, 60);
        for interval in [0, 86401] {
            assert!(
                engine
                    .command(Command::ConfigurePolling {
                        farming: true,
                        interval
                    })
                    .is_err()
            );
        }
        assert_eq!(engine.snapshot().farm.interval, 60);
        engine.command(Command::StartBoth).unwrap();
        engine.command(Command::StartBoth).unwrap();
        assert!(engine.snapshot().farm.running && engine.snapshot().dispatch_running);
        engine.command(Command::StopBoth).unwrap();
        assert!(engine.snapshot().farm.stopping && engine.snapshot().dispatch_stopping);
        drop(guard);
        engine.shutdown();
        for _ in 0..100 {
            engine.poll();
            if engine.is_finished() {
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        panic!("停止后工作线程未退出");
    }
}
