use image::{
    RgbImage,
    imageops::{self, FilterType},
};
use std::{
    mem::{size_of, zeroed},
    thread::sleep,
    time::Duration,
};
use windows_sys::Win32::{
    Foundation::*,
    Graphics::{Dwm::*, Gdi::*},
    UI::HiDpi::{GetWindowDpiAwarenessContext, SetThreadDpiAwarenessContext},
    UI::{Input::KeyboardAndMouse::GetAsyncKeyState, WindowsAndMessaging::*},
};
pub type Result<T> = std::result::Result<T, String>;
pub fn delay(ms: u64) {
    sleep(Duration::from_millis(ms))
}
pub fn key(v: i32) -> bool {
    unsafe { GetAsyncKeyState(v) < 0 }
}
pub fn pid(h: HWND) -> u32 {
    let mut p = 0;
    unsafe { GetWindowThreadProcessId(h, &mut p) };
    p
}
fn windows() -> Vec<HWND> {
    unsafe extern "system" fn visit(h: HWND, p: LPARAM) -> BOOL {
        unsafe {
            if IsWindowVisible(h) != 0 {
                (*(p as *mut Vec<HWND>)).push(h)
            }
        };
        1
    }
    let mut list = Vec::new();
    unsafe { EnumWindows(Some(visit), &mut list as *mut _ as isize) };
    list
}
pub fn game() -> Result<Option<HWND>> {
    let mut found = None;
    for h in windows() {
        let mut buf = [0; 128];
        let n = unsafe { GetWindowTextW(h, buf.as_mut_ptr(), 128) };
        if String::from_utf16_lossy(&buf[..n as usize]).starts_with("懒人修仙传 v") {
            if found.is_some() {
                return Err("请只保留一个游戏窗口".into());
            }
            found = Some(h)
        }
    }
    Ok(found)
}
pub fn foreground(h: HWND) -> bool {
    unsafe { pid(GetForegroundWindow()) == pid(h) }
}
pub fn minimized(h: HWND) -> bool {
    unsafe { IsIconic(h) != 0 }
}
pub fn minimize(h: HWND, value: bool) -> Result<()> {
    unsafe {
        ShowWindowAsync(
            h,
            if value {
                SW_SHOWMINNOACTIVE
            } else {
                SW_SHOWNOACTIVATE
            },
        )
    };
    for _ in 0..40 {
        if minimized(h) == value {
            return Ok(());
        }
        delay(50)
    }
    Err("窗口状态切换超时".into())
}
fn bounds(h: HWND) -> Result<RECT> {
    unsafe {
        let mut r: RECT = zeroed();
        if IsWindow(h) == 0
            || minimized(h)
            || DwmGetWindowAttribute(h, 9, &mut r as *mut _ as _, 16) < 0
            || r.right <= r.left
            || r.bottom <= r.top
        {
            return Err("无法定位游戏窗口".into());
        }
        Ok(r)
    }
}
fn client(h: HWND) -> Result<(i32, i32, i32, i32)> {
    unsafe {
        let mut r = zeroed::<RECT>();
        let mut p = POINT { x: 0, y: 0 };
        if GetClientRect(h, &mut r) == 0 || ClientToScreen(h, &mut p) == 0 {
            return Err("窗口已关闭".into());
        }
        Ok((p.x, p.y, r.right, r.bottom))
    }
}
fn owned(h: HWND) -> Vec<HWND> {
    let p = pid(h);
    windows().into_iter().filter(|w| pid(*w) == p).collect()
}
fn capture(h: HWND, w: i32, height: i32) -> Result<RgbImage> {
    unsafe {
        let dc = GetDC(h);
        if dc == 0 {
            return Err("无法取得游戏窗口画面".into());
        }
        let old_dpi = SetThreadDpiAwarenessContext(GetWindowDpiAwarenessContext(h));
        let mut logical = zeroed::<RECT>();
        let measured = GetClientRect(h, &mut logical);
        SetThreadDpiAwarenessContext(old_dpi);
        let mem = CreateCompatibleDC(dc);
        SetStretchBltMode(mem, HALFTONE);
        let mut info = zeroed::<BITMAPINFO>();
        info.bmiHeader.biSize = size_of::<BITMAPINFOHEADER>() as u32;
        info.bmiHeader.biWidth = w;
        info.bmiHeader.biHeight = -height;
        info.bmiHeader.biPlanes = 1;
        info.bmiHeader.biBitCount = 32;
        let mut bits = std::ptr::null_mut();
        let bmp = CreateDIBSection(dc, &info, DIB_RGB_COLORS, &mut bits, 0, 0);
        let old = SelectObject(mem, bmp);
        let result = if bmp != 0
            && mem != 0
            && !bits.is_null()
            && measured != 0
            && StretchBlt(
                mem,
                0,
                0,
                w,
                height,
                dc,
                0,
                0,
                logical.right,
                logical.bottom,
                SRCCOPY,
            ) != 0
        {
            GdiFlush();
            let bytes =
                std::slice::from_raw_parts(bits as *const u8, w as usize * height as usize * 4);
            let mut rgb = Vec::with_capacity(bytes.len() / 4 * 3);
            for p in bytes.chunks_exact(4) {
                rgb.extend_from_slice(&[p[2], p[1], p[0]])
            }
            RgbImage::from_raw(w as u32, height as u32, rgb).ok_or("截图尺寸异常".to_string())
        } else {
            Err("游戏后台截图失败".into())
        };
        SelectObject(mem, old);
        DeleteObject(bmp);
        DeleteDC(mem);
        ReleaseDC(h, dc);
        result
    }
}
pub fn frame(h: HWND) -> Result<RgbImage> {
    let r = bounds(h)?;
    let mut im = RgbImage::new((r.right - r.left) as u32, (r.bottom - r.top) as u32);
    for win in owned(h).into_iter().rev() {
        let (x, y, w, b) = client(win)?;
        if w > 0 && b > 0 && x < r.right && y < r.bottom && x + w > r.left && y + b > r.top {
            imageops::overlay(
                &mut im,
                &capture(win, w, b)?,
                (x - r.left) as i64,
                (y - r.top) as i64,
            )
        }
    }
    Ok(imageops::resize(&im, 1612, 894, FilterType::Triangle))
}
pub fn post(h: HWND, m: u32, k: usize, l: isize) -> Result<()> {
    if unsafe { PostMessageW(h, m, k, l) } == 0 {
        Err(format!("窗口消息失败：{}", std::io::Error::last_os_error()))
    } else {
        Ok(())
    }
}
fn pack(x: i32, y: i32) -> isize {
    (((y as u32 & 65535) << 16) | (x as u32 & 65535)) as isize
}
pub fn click(h: HWND, x: i32, y: i32, main: bool) -> Result<()> {
    let r = bounds(h)?;
    let sx = r.left + x * (r.right - r.left) / 1612;
    let sy = r.top + y * (r.bottom - r.top) / 894;
    for mut win in if main { vec![h] } else { owned(h) } {
        let (l, t, w, b) = client(win)?;
        if !(l..l + w).contains(&sx) || !(t..t + b).contains(&sy) {
            continue;
        }
        let lp = loop {
            let (l, t, _, _) = client(win)?;
            let p = POINT {
                x: sx - l,
                y: sy - t,
            };
            let child = if main {
                0
            } else {
                unsafe { ChildWindowFromPointEx(win, p, 7) }
            };
            if child == 0 || child == win {
                break pack(p.x, p.y);
            }
            win = child
        };
        for (m, k) in [(WM_MOUSEMOVE, 0), (WM_LBUTTONDOWN, 1), (WM_LBUTTONUP, 0)] {
            if let Err(e) = post(win, m, k, lp) {
                if m == WM_LBUTTONUP && unsafe { IsWindow(win) } == 0 {
                    break;
                }
                return Err(e);
            }
            delay(40)
        }
        delay(350);
        return Ok(());
    }
    Err("点击超出游戏窗口".into())
}
pub fn cursor_map(h: HWND) -> Result<(i32, i32)> {
    let r = bounds(h)?;
    let mut p = POINT { x: 0, y: 0 };
    unsafe { GetCursorPos(&mut p) };
    Ok((
        (p.x - r.left) * 1612 / (r.right - r.left),
        (p.y - r.top) * 894 / (r.bottom - r.top),
    ))
}
pub fn repeat_click(h: HWND, right: bool) -> Result<bool> {
    unsafe {
        let mut p = POINT { x: 0, y: 0 };
        GetCursorPos(&mut p);
        let w = WindowFromPoint(p);
        if w == 0 || pid(w) != pid(h) || ScreenToClient(w, &mut p) == 0 {
            return Ok(false);
        }
        let lp = pack(p.x, p.y);
        let (down, up, k) = if right {
            (WM_RBUTTONDOWN, WM_RBUTTONUP, 2)
        } else {
            (WM_LBUTTONDOWN, WM_LBUTTONUP, 1)
        };
        post(w, down, k, lp)?;
        post(w, up, 0, lp)?;
        Ok(true)
    }
}

/// Locate the game-owned native popup and address items by their displayed text.
fn popup_item(h: HWND, label: &str) -> Result<(HWND, HMENU, u32, usize, bool)> {
    for _ in 0..15 {
        for popup in owned(h) {
            let mut class = [0; 32];
            unsafe {
                let len = GetClassNameW(popup, class.as_mut_ptr(), class.len() as i32);
                if String::from_utf16_lossy(&class[..len as usize]) != "#32768" {
                    continue;
                }
                let mut menu = 0;
                // MN_GETHMENU returns the HMENU of this popup window.
                if SendMessageTimeoutW(popup, 0x01e1, 0, 0, SMTO_ABORTIFHUNG, 250, &mut menu) == 0 {
                    continue;
                }
                let menu = menu as HMENU;
                for pos in 0..GetMenuItemCount(menu) {
                    let mut title = [0; 96];
                    let n = GetMenuStringW(menu, pos as u32, title.as_mut_ptr(), 96, MF_BYPOSITION);
                    if !String::from_utf16_lossy(&title[..n as usize]).contains(label) {
                        continue;
                    }
                    let preceding = (0..pos)
                        .filter(|i| {
                            GetMenuState(menu, *i as u32, MF_BYPOSITION) & MF_SEPARATOR == 0
                        })
                        .count();
                    let enabled = GetMenuState(menu, pos as u32, MF_BYPOSITION)
                        & (MF_DISABLED | MF_GRAYED)
                        == 0;
                    return Ok((popup, menu, pos as u32, preceding, enabled));
                }
            }
        }
        delay(100);
    }
    Err(format!("未找到游戏菜单项：{label}"))
}

fn select_menu_item(
    popup: HWND,
    menu: HMENU,
    position: u32,
    preceding: usize,
    key: usize,
) -> Result<()> {
    // Menu navigation includes disabled entries but skips separators.
    post(popup, WM_KEYDOWN, 0x24, 1)?;
    for _ in 0..preceding {
        post(popup, WM_KEYDOWN, 0x28, 1)?;
    }
    for _ in 0..10 {
        let state = unsafe { GetMenuState(menu, position, MF_BYPOSITION) };
        if state != u32::MAX && state & MF_HILITE != 0 && state & (MF_DISABLED | MF_GRAYED) == 0 {
            return post(popup, WM_KEYDOWN, key, 1);
        }
        delay(50);
    }
    Err("未选中指定菜单项，已取消操作".into())
}

pub fn garden_menu(h: HWND, plant: bool) -> Result<bool> {
    let action = || -> Result<bool> {
        if plant {
            let (popup, menu, position, preceding, enabled) = popup_item(h, "中种下")?;
            if !enabled {
                return Err("没有可播种的空坑".into());
            }
            select_menu_item(popup, menu, position, preceding, 0x27)?;
        }
        let label = if plant {
            "种植高级灵谷"
        } else {
            "收获成熟作物"
        };
        let (popup, menu, position, preceding, enabled) = popup_item(h, label)?;
        if !enabled {
            if plant {
                return Err("高级灵谷不可用".into());
            }
            return Ok(false);
        }
        select_menu_item(popup, menu, position, preceding, 0x0d)?;
        delay(350);
        Ok(true)
    };
    let result = action();
    if !matches!(result, Ok(true)) {
        post(h, WM_CANCELMODE, 0, 0)?;
        delay(150);
    }
    result
}

#[cfg(test)]
mod live {
    #[test]
    #[ignore = "requires the game and a local screenshot destination"]
    fn capture_battle() {
        use super::*;
        unsafe {
            windows_sys::Win32::UI::HiDpi::SetThreadDpiAwarenessContext(-4);
        }
        let h = game().unwrap().unwrap();
        let mini = minimized(h);
        if mini {
            minimize(h, false).unwrap();
            delay(400);
        }
        let capture = frame(h);
        if mini {
            minimize(h, true).unwrap();
        }
        let im = capture.unwrap();
        im.save(std::env::temp_dir().join("xiuxian-live-capture.png"))
            .unwrap();
        let screen = crate::vision::Frame::new(im);
        assert!(crate::vision::Refs::load().has(&screen, "current", 686, 96));
    }
}
