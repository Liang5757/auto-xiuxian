use crate::{
    native::{self as win, Result},
    vision::{Frame, Refs, Template},
};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
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
    refs: Refs,
    homes: Vec<(Template, (i32, i32))>,
}
impl Session {
    pub fn new(stop: Arc<AtomicBool>, tx: Sender<Event>, busy: Arc<std::sync::Mutex<()>>) -> Self {
        Self {
            stop,
            tx,
            busy,
            refs: Refs::load(),
            homes: vec![],
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
        if !battle || !self.refs.returned(&f) {
            if initial {
                self.log("未发现返回提示，继续监听")
            }
            return Ok(0);
        }
        self.emit(Event::State("正在派遣".into()));
        let title = f.crop(677, 122, 97, 24);
        if !self.refs.has(&f, "petpanel", 1344, 93) {
            win::click(h, 1430, 73, false)?;
            self.wait_key(h, "petpanel", 1344, 93, "宠物栏未打开")?;
        }
        self.world(h)?;
        let home = if origin {
            (1270, 779)
        } else if let Some((_, p)) = self
            .homes
            .iter()
            .find(|(t, _)| t.matches(&f.gray, 677, 122, 6))
        {
            *p
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
        self.world(h)?;
        win::click(h, home.0, home.1, false)?;
        self.wait(
            h,
            |f| {
                self.refs.has(f, "current", 686, 96)
                    && title.matches(&f.gray, 677, 122, 6)
                    && !self.refs.has(f, "world", 276, 751)
            },
            "回程地图核对失败",
        )?;
        self.emit(Event::Map(name.into()));
        self.log("已回到原地图");
        if count == 0 {
            return Err("返回提示与队伍状态不一致".into());
        }
        Ok(count)
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
        let mut initial = true;
        let mut waiting = false;
        while !self.stop.load(Ordering::Relaxed) {
            let h = match win::game() {
                Ok(Some(h)) => h,
                Ok(None) => {
                    if !waiting {
                        self.emit(Event::State("等待游戏启动".into()));
                        waiting = true
                    }
                    self.pause(2);
                    continue;
                }
                Err(e) => {
                    self.log(e);
                    break;
                }
            };
            let lock = self.busy.clone();
            let result = {
                let _guard = lock.lock().unwrap();
                self.round(h, initial)
            };
            match result {
                Ok(0) => {}
                Ok(n) => self.emit(Event::Success(n)),
                Err(e) => {
                    if self.stop.load(Ordering::Relaxed) {
                        self.log(e)
                    } else {
                        self.emit(Event::Failure(e))
                    }
                    break;
                }
            }
            self.emit(Event::State("正在监听".into()));
            initial = false;
            waiting = false;
            self.pause(30)
        }
        self.emit(Event::Stopped)
    }
    fn pause(&self, secs: u32) {
        for _ in 0..secs * 10 {
            if self.stop.load(Ordering::Relaxed) {
                break;
            }
            std::thread::sleep(Duration::from_millis(100))
        }
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
