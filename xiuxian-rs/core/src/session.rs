use crate::{
    native::{self as win, Result},
    vision::{Frame, Refs, Template},
};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc::Sender,
    },
    time::Duration,
};
pub enum Event {
    State(String),
    Map(String),
    Log(String),
    Success(usize),
    Failure(String),
    Stopped,
}
pub struct Session {
    pub stop: Arc<AtomicBool>,
    pub tx: Sender<Event>,
    pub busy: Arc<std::sync::Mutex<()>>,
    pub farming: bool,
    pub interval: Arc<AtomicU64>,
    needs_planting: bool,
    refs: Refs,
    homes: Vec<(Template, (i32, i32))>,
    return_map: Option<ReturnMap>,
}
struct ReturnMap {
    title: Template,
    point: (i32, i32),
    name: String,
}
impl Session {
    pub fn new(stop: Arc<AtomicBool>, tx: Sender<Event>, busy: Arc<std::sync::Mutex<()>>) -> Self {
        Self {
            stop,
            tx,
            busy,
            farming: false,
            interval: Arc::new(AtomicU64::new(30)),
            needs_planting: false,
            refs: Refs::load(),
            homes: vec![],
            return_map: None,
        }
    }
    fn emit(&self, e: Event) {
        let _ = self.tx.send(e);
    }
    fn log(&self, s: impl Into<String>) {
        self.emit(Event::Log(s.into()))
    }
    fn frame(&self, h: isize) -> Result<Frame> {
        Ok(Frame::new(win::frame(h)?))
    }
    fn wait(&self, h: isize, ok: impl Fn(&Frame) -> bool, msg: &str) -> Result<Frame> {
        for _ in 0..18 {
            let f = self.frame(h)?;
            if ok(&f) {
                return Ok(f);
            }
            win::delay(180)
        }
        Err(msg.into())
    }
    fn wait_key(&self, h: isize, key: &str, x: i32, y: i32, msg: &str) -> Result<Frame> {
        self.wait(h, |f| self.refs.has(f, key, x, y), msg)
    }
    fn world(&self, h: isize) -> Result<()> {
        win::post(h, 0x312, 0xa004, (70 << 16) | 1)?;
        self.wait_key(h, "world", 276, 751, "未识别到世界地图")?;
        Ok(())
    }
    fn row(&self, h: isize, i: i32) -> Result<bool> {
        let d = (i - 1) * 98;
        let f = self.frame(h)?;
        if !self.refs.has(&f, "safe", 53, 211 + d) || !self.refs.has(&f, "collect", 1137, 244 + d) {
            return Ok(false);
        }
        if !self.refs.has(&f, "petpanel", 1344, 93) {
            return Err("宠物栏未打开".into());
        }
        let before = f.slots();
        let pet = f.crop(61, (243 + d) as u32, 41, 38);
        win::click(h, 1173, 258 + d, false)?;
        let f = self.wait_key(h, "empty", 601, 243 + d, "收取后未见空位")?;
        let after = f.slots();
        if after.len() != before.len() + 1
            || before.iter().zip(&after).any(|((i, t), (j, _))| {
                i != j || !t.matches(&f.gray, 1286, 201 + *j as i32 * 63, 2)
            })
        {
            return Err("收取后宠物列表异常，请手动选宠".into());
        }
        let slot = after.last().unwrap().0 as i32;
        win::click(h, 1173, 258 + d, false)?;
        self.wait_key(h, "choose", 53, 211 + d, "未打开选宠栏")?;
        win::click(h, 1390, 225 + slot * 63, false)?;
        self.wait(h, |f| pet.matches(&f.gray, 61, 243 + d, 6), "选中头像不符")?;
        win::click(h, 991, 256 + d, true)?;
        self.wait_key(h, "travel", 53, 212 + d, "未确认小队出发")?;
        Ok(true)
    }
    fn inspect(&mut self, h: isize, initial: bool) -> Result<usize> {
        let f = self.frame(h)?;
        let battle = self.refs.has(&f, "current", 686, 96);
        let origin = self.refs.has(&f, "origin", 675, 122);
        let name = if !battle {
            "待识别"
        } else if origin {
            "时空原点"
        } else {
            "其他地图"
        };
        self.emit(Event::Map(name.into()));
        if initial && !battle {
            return Err("请在原挂机地图的战斗页启动".into());
        }
        let ready = if self.farming {
            self.needs_planting
                || (0..5).any(|i| self.refs.has(&f, "ripe_notice", 1098, 215 + i * 30))
        } else {
            self.refs.returned(&f)
        };
        if !battle || !ready {
            if initial {
                self.log("未发现任务提示，继续监听")
            }
            return Ok(0);
        }
        self.emit(Event::State(
            if self.farming {
                "正在收种"
            } else {
                "正在派遣"
            }
            .into(),
        ));
        if !self.farming && !self.refs.has(&f, "petpanel", 1344, 93) {
            win::click(h, 1430, 73, false)?;
            self.wait_key(h, "petpanel", 1344, 93, "宠物栏未打开")?;
        }
        let known_home = if origin {
            Some((1270, 779))
        } else {
            self.homes
                .iter()
                .find(|(t, _)| t.matches(&f.gray, 677, 122, 6))
                .map(|(_, point)| *point)
        };
        if let Some(point) = known_home {
            self.return_map = Some(ReturnMap {
                title: f.crop(677, 122, 97, 24),
                point,
                name: name.into(),
            });
        }
        self.world(h)?;
        let home = if let Some(point) = known_home {
            point
        } else {
            self.log("请切到游戏，鼠标指向原挂机地图图标，按 F8");
            let p = loop {
                if self.stop.load(Ordering::Relaxed) {
                    return Err("已取消地图标记".into());
                }
                if win::key(0x77) && win::foreground(h) {
                    let (x, y) = win::cursor_map(h)?;
                    if (251..1360).contains(&x) && (36..830).contains(&y) {
                        break (x, y);
                    }
                }
                win::delay(80)
            };
            self.homes.push((f.crop(677, 122, 97, 24), p));
            p
        };
        if self.return_map.is_none() {
            self.return_map = Some(ReturnMap {
                title: f.crop(677, 122, 97, 24),
                point: home,
                name: name.into(),
            });
        }
        if self.farming {
            let result = self.farm(h);
            self.restore_home(h)?;
            return result;
        }
        win::click(h, 363, 457, false)?;
        self.wait_key(h, "expedition", 42, 58, "未进入蛮荒之地")?;
        self.emit(Event::Map("蛮荒之地".into()));
        self.log("进入蛮荒之地");
        let mut count = 0;
        for i in 0..3 {
            if self.row(h, i)? {
                count += 1;
                self.log(format!("第 {} 支小队重新出发", i + 1))
            }
        }
        self.restore_home(h)?;
        if count == 0 {
            return Err("返回提示与队伍状态不一致".into());
        }
        Ok(count)
    }
    fn farm(&mut self, h: isize) -> Result<usize> {
        win::click(h, 729, 774, false)?;
        self.wait_key(h, "cave", 40, 60, "未进入洞府聚灵阵")?;
        win::click(h, 1140, 654, false)?;
        self.wait_key(h, "garden", 40, 60, "未进入灵植园")?;
        self.emit(Event::Map("灵植园".into()));
        let empty = |f: &Frame| -> Vec<_> {
            (0..9)
                .filter(|i| {
                    self.refs
                        .has(f, "plot_empty", 175 + i % 3 * 345, 145 + i / 3 * 98)
                })
                .collect()
        };
        self.wait_key(h, "quick", 784, 433, "未找到快捷种植按钮")?;
        win::click(h, 850, 447, false)?;
        let harvested = win::garden_menu(h, false)?;
        if !harvested && empty(&self.frame(h)?).is_empty() {
            self.needs_planting = false;
            return Ok(0);
        }
        let frame = self.wait(h, |f| !empty(f).is_empty(), "未检测到可播种的空坑")?;
        let plots = empty(&frame);
        self.needs_planting = true;
        win::click(h, 850, 447, false)?;
        win::garden_menu(h, true)?;
        self.wait(
            h,
            |f| {
                plots.iter().all(|i| {
                    self.refs
                        .has(f, "growing", 111 + i % 3 * 345, 152 + i / 3 * 98)
                })
            },
            "未确认全部空坑已种下作物",
        )?;
        self.needs_planting = false;
        self.log(if harvested {
            "已收获成熟作物并补种高级灵谷"
        } else {
            "已为空坑补种高级灵谷"
        });
        Ok(plots.len())
    }
    fn restore_home(&mut self, h: isize) -> Result<()> {
        let Some(home) = self.return_map.as_ref() else {
            return Ok(());
        };
        let at_home = |f: &Frame| {
            self.refs.has(f, "current", 686, 96)
                && home.title.matches(&f.gray, 677, 122, 6)
                && !self.refs.has(f, "world", 276, 751)
        };
        let frame = self.frame(h)?;
        if !at_home(&frame) {
            if !self.refs.has(&frame, "world", 276, 751) {
                self.world(h)?;
            }
            win::click(h, home.point.0, home.point.1, false)?;
            self.wait(h, at_home, "回程地图核对失败")?;
        }
        let home = self.return_map.take().unwrap();
        self.emit(Event::Map(home.name));
        self.log("已回到原地图");
        Ok(())
    }
    fn round(&mut self, h: isize, initial: bool) -> Result<usize> {
        let mini = win::minimized(h);
        let res = (|| {
            if mini {
                win::minimize(h, false)?;
                win::delay(400);
                if win::foreground(h) {
                    return Err("还原后游戏获得焦点".into());
                }
            }
            self.restore_home(h)?;
            self.inspect(h, initial)
        })();
        if mini {
            win::minimize(h, true)?
        }
        res
    }
    pub fn run(mut self) {
        unsafe {
            windows_sys::Win32::UI::HiDpi::SetThreadDpiAwarenessContext(-4);
        }
        self.run_loop(Self::check, Self::pause);
    }
    fn check(&mut self, initial: bool) -> Result<Option<usize>> {
        let Some(h) = win::game()? else {
            return Ok(None);
        };
        let lock = self.busy.clone();
        let _guard = loop {
            if self.stop.load(Ordering::Relaxed) {
                return Ok(Some(0));
            }
            match lock.try_lock() {
                Ok(guard) => break guard,
                Err(std::sync::TryLockError::WouldBlock) => win::delay(100),
                Err(std::sync::TryLockError::Poisoned(_)) => return Err("游戏操作锁异常".into()),
            }
        };
        self.round(h, initial).map(Some)
    }
    fn run_loop(
        &mut self,
        mut check: impl FnMut(&mut Self, bool) -> Result<Option<usize>>,
        mut pause: impl FnMut(&Self, u32),
    ) {
        let mut initial = true;
        let mut waiting = false;
        while !self.stop.load(Ordering::Relaxed) {
            let result = check(self, initial);
            match result {
                Ok(None) => {
                    if !waiting {
                        self.emit(Event::State("等待游戏启动".into()));
                        waiting = true
                    }
                    pause(self, 2);
                    continue;
                }
                Ok(Some(count)) => {
                    if count > 0 {
                        self.emit(Event::Success(count));
                    }
                    initial = false;
                    self.emit(Event::State("正在监听".into()));
                }
                Err(e) => {
                    if self.stop.load(Ordering::Relaxed) {
                        self.log(e);
                        break;
                    }
                    self.emit(Event::Failure(e));
                    self.emit(Event::State("本轮失败，等待重试".into()));
                }
            }
            if self.stop.load(Ordering::Relaxed) {
                break;
            }
            waiting = false;
            pause(self, 0) // Use the configurable polling interval.
        }
        self.emit(Event::Stopped)
    }
    fn pause(&self, secs: u32) {
        let mut elapsed = 0;
        loop {
            let limit = if secs == 0 {
                self.interval.load(Ordering::Relaxed)
            } else {
                secs as u64
            };
            if elapsed >= limit * 10 || self.stop.load(Ordering::Relaxed) {
                break;
            }
            std::thread::sleep(Duration::from_millis(100));
            elapsed += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failures_keep_polling_and_later_rounds_can_succeed() {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut job = Session::new(Default::default(), tx, Default::default());
        let mut checks = 0;
        let mut pauses = Vec::new();
        job.run_loop(
            |job, initial| {
                assert_eq!(initial, checks < 3);
                let result = match checks {
                    0 | 1 => Err("模拟本轮失败".into()),
                    2 => Ok(Some(2)),
                    3 => Ok(None),
                    4 => Ok(Some(0)),
                    5 => {
                        job.stop.store(true, Ordering::Relaxed);
                        Err("用户取消".into())
                    }
                    _ => panic!("取消后仍在检查"),
                };
                checks += 1;
                result
            },
            |_, seconds| pauses.push(seconds),
        );
        let events: Vec<_> = rx.try_iter().collect();
        assert_eq!(checks, 6);
        assert_eq!(pauses, [0, 0, 0, 2, 0]);
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e, Event::Failure(_)))
                .count(),
            2
        );
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e, Event::Success(2)))
                .count(),
            1
        );
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e, Event::Stopped))
                .count(),
            1
        );
    }

    #[test]
    fn stop_during_retry_wait_prevents_another_round() {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut job = Session::new(Default::default(), tx, Default::default());
        let mut checks = 0;
        job.run_loop(
            |_, _| {
                checks += 1;
                Err("模拟失败".into())
            },
            |job, seconds| {
                assert_eq!(seconds, 0);
                job.stop.store(true, Ordering::Relaxed);
            },
        );
        assert_eq!(checks, 1);
        let events: Vec<_> = rx.try_iter().collect();
        assert!(matches!(events.last(), Some(Event::Stopped)));
    }
}

#[cfg(test)]
mod live {
    use super::*;

    #[test]
    #[ignore = "operates the live game; requires the origin battle map"]
    fn inspect_origin_once() {
        unsafe {
            windows_sys::Win32::UI::HiDpi::SetThreadDpiAwarenessContext(-4);
        }
        let (tx, rx) = std::sync::mpsc::channel();
        let mut job = Session::new(Default::default(), tx, Default::default());
        let h = win::game().unwrap().unwrap();
        let mini = win::minimized(h);
        let result = (|| -> Result<usize> {
            if mini {
                win::minimize(h, false)?;
                win::delay(400);
            }
            let f = job.frame(h)?;
            if !job.refs.has(&f, "origin", 675, 122) {
                return Err("实机测试需要时空原点".into());
            }
            job.inspect(h, true)
        })();
        if mini {
            win::minimize(h, true).unwrap();
        }
        for e in rx.try_iter() {
            if let Event::Log(s) = e {
                println!("{s}");
            }
        }
        println!("round: {result:?}");
        result.unwrap();
    }
}
