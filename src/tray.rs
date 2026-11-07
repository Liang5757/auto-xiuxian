use eframe::egui;
use std::{
    mem::{size_of, zeroed},
    ptr::null,
    sync::mpsc::{self, Receiver, Sender},
    thread::{self, JoinHandle},
    time::Duration,
};
use windows_sys::Win32::{
    Foundation::*,
    System::LibraryLoader::GetModuleHandleW,
    UI::{Shell::*, WindowsAndMessaging::*},
};
const CALLBACK: u32 = WM_APP + 1;
const SHOW: usize = 1;
const EXIT: usize = 2;
pub enum Event {
    Show,
    Exit,
}
struct State {
    tx: Sender<Event>,
    ctx: egui::Context,
    taskbar_created: u32,
}
pub struct Tray {
    pub events: Receiver<Event>,
    hwnd: HWND,
    thread: Option<JoinHandle<()>>,
}
fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
}
unsafe fn data(hwnd: HWND) -> NOTIFYICONDATAW {
    unsafe {
        let mut data: NOTIFYICONDATAW = zeroed();
        data.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
        data.hWnd = hwnd;
        data.uID = 1;
        data.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
        data.uCallbackMessage = CALLBACK;
        data.hIcon = LoadIconW(GetModuleHandleW(null()), std::ptr::without_provenance(1));
        let tip = wide("修仙助手 · 双击打开，右键退出");
        data.szTip[..tip.len()].copy_from_slice(&tip);
        data
    }
}
fn restore(ctx: &egui::Context) {
    // A hidden Windows window may not receive repaint messages until shown.
    unsafe extern "system" fn show(hwnd: HWND, _: LPARAM) -> BOOL {
        unsafe {
            let mut pid = 0;
            GetWindowThreadProcessId(hwnd, &mut pid);
            if pid == std::process::id()
                && GetWindowLongW(hwnd, GWL_EXSTYLE) & WS_EX_TOOLWINDOW as i32 == 0
            {
                let mut title = [0u16; 32];
                let len = GetWindowTextW(hwnd, title.as_mut_ptr(), 32);
                if String::from_utf16_lossy(&title[..len as usize]) == "修仙助手" {
                    ShowWindow(hwnd, SW_RESTORE);
                    SetForegroundWindow(hwnd);
                    return 0;
                }
            }
            1
        }
    }
    unsafe {
        EnumWindows(Some(show), 0);
    }
    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
    ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
    ctx.request_repaint();
}
unsafe extern "system" fn procedure(hwnd: HWND, message: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    unsafe {
        if message == WM_NCCREATE {
            let create = &*(l as *const CREATESTRUCTW);
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, create.lpCreateParams as isize);
        }
        let state = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut State;
        if !state.is_null() {
            let state = &*state;
            if message == state.taskbar_created {
                Shell_NotifyIconW(NIM_ADD, &data(hwnd));
                return 0;
            }
            if message == CALLBACK {
                match l as u32 {
                    WM_LBUTTONDBLCLK => {
                        let _ = state.tx.send(Event::Show);
                        restore(&state.ctx);
                    }
                    WM_RBUTTONUP => {
                        let menu = CreatePopupMenu();
                        if menu != 0 {
                            AppendMenuW(menu, MF_STRING, SHOW, wide("显示窗口").as_ptr());
                            AppendMenuW(menu, MF_SEPARATOR, 0, null());
                            AppendMenuW(menu, MF_STRING, EXIT, wide("退出").as_ptr());
                            let mut point = zeroed::<POINT>();
                            GetCursorPos(&mut point);
                            SetForegroundWindow(hwnd);
                            let selected = TrackPopupMenu(
                                menu,
                                TPM_RETURNCMD | TPM_RIGHTBUTTON,
                                point.x,
                                point.y,
                                0,
                                hwnd,
                                null(),
                            );
                            DestroyMenu(menu);
                            PostMessageW(hwnd, WM_NULL, 0, 0);
                            if selected == SHOW as i32 || selected == EXIT as i32 {
                                let _ = state.tx.send(if selected == SHOW as i32 {
                                    Event::Show
                                } else {
                                    Event::Exit
                                });
                                restore(&state.ctx);
                            }
                        }
                    }
                    _ => {}
                }
                return 0;
            }
        }
        match message {
            WM_CLOSE => {
                DestroyWindow(hwnd);
                0
            }
            WM_DESTROY => {
                Shell_NotifyIconW(NIM_DELETE, &data(hwnd));
                PostQuitMessage(0);
                0
            }
            _ => DefWindowProcW(hwnd, message, w, l),
        }
    }
}
pub fn show_existing() -> bool {
    unsafe {
        let hwnd = FindWindowW(
            wide("XiuxianTrayWindow").as_ptr(),
            wide("修仙助手托盘").as_ptr(),
        );
        hwnd != 0 && PostMessageW(hwnd, CALLBACK, 0, WM_LBUTTONDBLCLK as isize) != 0
    }
}

impl Tray {
    pub fn new(ctx: egui::Context) -> Result<Self, String> {
        let (tx, events) = mpsc::channel();
        let (ready_tx, ready_rx) = mpsc::channel::<Result<HWND, String>>();
        let thread = thread::spawn(move || unsafe {
            let instance = GetModuleHandleW(null());
            let class = wide("XiuxianTrayWindow");
            let wc = WNDCLASSW {
                lpfnWndProc: Some(procedure),
                hInstance: instance,
                lpszClassName: class.as_ptr(),
                ..zeroed()
            };
            if RegisterClassW(&wc) == 0 {
                let _ = ready_tx.send(Err("托盘窗口注册失败".into()));
                return;
            }
            let state = Box::new(State {
                tx,
                ctx,
                taskbar_created: RegisterWindowMessageW(wide("TaskbarCreated").as_ptr()),
            });
            let hwnd = CreateWindowExW(
                WS_EX_TOOLWINDOW,
                class.as_ptr(),
                wide("修仙助手托盘").as_ptr(),
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                instance,
                &*state as *const State as _,
            );
            if hwnd == 0 || Shell_NotifyIconW(NIM_ADD, &data(hwnd)) == 0 {
                if hwnd != 0 {
                    DestroyWindow(hwnd);
                }
                let _ = ready_tx.send(Err("系统托盘创建失败".into()));
            } else if ready_tx.send(Ok(hwnd)).is_ok() {
                let mut message = zeroed::<MSG>();
                while GetMessageW(&mut message, 0, 0, 0) > 0 {
                    TranslateMessage(&message);
                    DispatchMessageW(&message);
                }
            } else {
                DestroyWindow(hwnd);
            }
            UnregisterClassW(class.as_ptr(), instance);
        });
        let hwnd = ready_rx
            .recv_timeout(Duration::from_secs(3))
            .map_err(|_| "系统托盘初始化超时".to_string())??;
        Ok(Self {
            events,
            hwnd,
            thread: Some(thread),
        })
    }
}
impl Drop for Tray {
    fn drop(&mut self) {
        unsafe {
            PostMessageW(self.hwnd, WM_CLOSE, 0, 0);
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires interactive Windows desktop"]
    fn tray_lifecycle() {
        let tray = Tray::new(egui::Context::default()).unwrap();
        unsafe {
            assert_ne!(
                PostMessageW(tray.hwnd, CALLBACK, 0, WM_LBUTTONDBLCLK as isize),
                0
            );
        }
        assert!(matches!(
            tray.events.recv_timeout(Duration::from_secs(2)),
            Ok(Event::Show)
        ));
        let hwnd = tray.hwnd;
        drop(tray);
        assert_eq!(unsafe { IsWindow(hwnd) }, 0);
    }
}
