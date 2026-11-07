"""懒人修仙传轻变 Mod，后台窗口识别。Esc 停止。"""
import ctypes as C
from ctypes import wintypes as W
import sys
import time
from pathlib import Path

import numpy as np
from PIL import Image, ImageGrab

U = C.windll.user32
U.SetProcessDPIAware()
REF = Path(__file__).parent / "refs"
SIZE = (1612, 894)
templates = {p.stem: Image.open(p).convert("L") for p in REF.glob("*.png")}
hwnd = 0


def tick():
    if U.GetAsyncKeyState(0x1b) & 0x8000:
        raise KeyboardInterrupt


def pid(handle):
    result = W.DWORD()
    U.GetWindowThreadProcessId(handle, C.byref(result))
    return result.value


def rect():
    tick()
    if not U.IsWindow(hwnd) or U.IsIconic(hwnd):
        raise RuntimeError("请保持游戏窗口打开并取消最小化。")
    box = W.RECT()
    if C.windll.dwmapi.DwmGetWindowAttribute(hwnd, 9, C.byref(box), C.sizeof(box)):
        raise RuntimeError("无法定位游戏窗口。")
    return box.left, box.top, box.right, box.bottom


def client_box(handle):
    box = W.RECT()
    origin = W.POINT()
    if not U.GetClientRect(handle, C.byref(box)) or not U.ClientToScreen(handle, C.byref(origin)):
        raise RuntimeError("无法读取游戏窗口位置。")
    return origin.x, origin.y, box.right, box.bottom


def windows():
    result = []

    @C.WINFUNCTYPE(W.BOOL, W.HWND, W.LPARAM)
    def visit(handle, _):
        if U.IsWindowVisible(handle) and pid(handle) == pid(hwnd):
            result.append(handle)
        return True

    U.EnumWindows(visit, 0)
    return result


def frame():
    l, t, r, b = rect()
    canvas = Image.new("RGB", (r-l, b-t))
    for handle in reversed(windows()):
        x, y, w, h = client_box(handle)
        if w and h and x < r and y < b and x+w > l and y+h > t:
            layer = ImageGrab.grab(window=handle)
            canvas.paste(layer.resize((w, h)), (x-l, y-t))
    return canvas.resize(SIZE)


def click(x, y, *, main_window=False):
    l, t, r, b = rect()
    sx = round(l+x*(r-l)/SIZE[0])
    sy = round(t+y*(b-t)/SIZE[1])
    for handle in ([hwnd] if main_window else windows()):
        left, top, w, h = client_box(handle)
        if left <= sx < left+w and top <= sy < top+h:
            break
    else:
        raise RuntimeError("目标不在游戏窗口内。")
    while not main_window:
        left, top, w, h = client_box(handle)
        child = U.ChildWindowFromPointEx(handle, W.POINT(sx-left, sy-top), 7)
        if not child or child == handle:
            break
        handle = child
    # USER32 按接收线程 DPI 转换跨进程鼠标消息。
    lp = ((sy-top) << 16) | ((sx-left) & 0xffff)
    for msg, keys in ((0x200, 0), (0x201, 1), (0x202, 0)):
        if not U.PostMessageW(handle, msg, keys, lp):
            error = C.windll.kernel32.GetLastError()
            if msg == 0x202 and error == 1400 and not U.IsWindow(handle):
                break  # 地图窗口已在按下时关闭，由调用方核对目标场景。
            raise RuntimeError(f"后台点击被 Windows 拒绝（{error}）。请确认游戏与脚本权限一致。")
        time.sleep(.04)
    time.sleep(.35)


def matches(img, template, x, y, radius=6):
    ref = np.asarray(template, dtype=np.float32)
    ref -= ref.mean()
    norm = np.linalg.norm(ref)
    if norm < 1:
        return False
    w, h = template.size
    patch = np.asarray(img.crop((x-radius, y-radius, x+w+radius, y+h+radius)).convert("L"), dtype=np.float32)
    for dy in range(0, 2*radius+1):
        for dx in range(0, 2*radius+1):
            cell = patch[dy:dy+h, dx:dx+w]
            centered = cell-cell.mean()
            if (ref*centered).sum() > .78*norm*max(np.linalg.norm(centered), 1):
                return True
    return False


def has(img, key, x, y):
    return matches(img, templates[key], x, y)


def has_return_notice(img):
    # 战斗页右侧的通知框可同时显示多行。先筛蓝字，再匹配返回提示文字。
    panel = np.asarray(img.crop((1045, 209, 1257, 377)).convert("RGB"), dtype=np.int16)
    red, green, blue = panel.transpose(2, 0, 1)
    blue_text = (blue > red + 35) & (blue > green + 25)
    for offset in range(0, 150, 30):
        if blue_text[offset:offset+30].sum() >= 30:
            if any(has(img, "notice", x, 215+offset) for x in range(1080, 1142, 10)):
                return True
    return False


def wait_for(check, message):
    for _ in range(18):
        img = frame()
        if check(img):
            return img
        time.sleep(.18)
    raise RuntimeError(message)


def world():
    # v1.1.11 registers Alt+F with hotkey ID 0xA004.
    wm_hotkey = 0x0312
    alt_f = (ord("F") << 16) | 1
    if not U.PostMessageW(hwnd, wm_hotkey, 0xA004, alt_f):
        raise RuntimeError(f"地图热键发送失败（{C.windll.kernel32.GetLastError()}）。")
    return wait_for(lambda im: has(im, "world", 276, 751), "未识别到世界地图。")


def home_point(battle, report=print, cancelled=lambda: False):
    if has(battle, "origin", 675, 122):
        return 1270, 779
    report("请切到游戏，将鼠标移到原挂机地点的图标上，按 F8 记住位置。")
    while True:
        if cancelled():
            raise InterruptedError("地图标记已取消。")
        l, t, r, b = rect()
        if pid(U.GetForegroundWindow()) == pid(hwnd) and U.GetAsyncKeyState(0x77) & 0x8000:
            pos = W.POINT()
            U.GetCursorPos(C.byref(pos))
            x, y = (pos.x-l)*1612/(r-l), (pos.y-t)*894/(b-t)
            if 250 < x < 1360 and 35 < y < 830:
                return round(x), round(y)
        time.sleep(.08)


def icons(img):
    result = []
    for i in range(11):
        y = 201+63*i
        if y+48 > 870:
            break
        icon = img.crop((1286, y, 1330, y+45))
        if np.asarray(icon).mean() < 223:
            result.append((i, icon))
    return result


def confirm_dispatch(row):
    offset = (row-1)*98
    # 选宠已核对；派遣按钮由主窗口绘制，宠物浮窗可能盖在按钮上。
    click(991, 256+offset, main_window=True)
    wait_for(lambda im: has(im, "travel", 53, 212+offset), "未确认小队重新出发。")


def handle_row(row):
    d = (row-1)*98
    img = frame()
    if not (has(img, "safe", 53, 211+d) and has(img, "collect", 1137, 244+d)):
        return False
    if not has(img, "petpanel", 1344, 93):
        raise RuntimeError("请打开右侧宠物栏再运行。")
    before = icons(img)
    identity = img.crop((61, 243+d, 102, 281+d)).convert("L")
    click(1173, 258+d)
    img = wait_for(lambda im: has(im, "empty", 601, 243+d), "收取后未见空位。")
    after = icons(img)
    if len(after) != len(before)+1 or any(
        i != j or not matches(img, old.convert("L"), 1286, 201+j*63, 2)
        for (i, old), (j, _) in zip(before, after)
    ):
        raise RuntimeError("收取后宠物列表顺序或数量异常，请手动选宠。")
    slot = after[-1][0]
    click(1173, 258+d)
    wait_for(lambda im: has(im, "choose", 53, 211+d), "未打开选宠栏。")
    click(1390, 225+slot*63)
    wait_for(lambda im: matches(im, identity, 61, 243+d), "选中头像与刚返回的小队不符。")
    confirm_dispatch(row)
    return True


class GameNotFound(RuntimeError):
    pass


def find_game():
    found = []

    @C.WINFUNCTYPE(W.BOOL, W.HWND, W.LPARAM)
    def visit(handle, _):
        title = C.create_unicode_buffer(256)
        U.GetWindowTextW(handle, title, len(title))
        if U.IsWindowVisible(handle) and title.value.startswith("懒人修仙传 v"):
            found.append(handle)
        return True

    U.EnumWindows(visit, 0)
    if not found:
        raise GameNotFound("未检测到游戏主窗口。")
    if len(found) > 1:
        raise RuntimeError("检测到多个游戏主窗口，请只保留一个。")
    return found[0]


def quiet_event(kind, value):
    pass


def inspect_teams(homes, event=quiet_event, cancelled=lambda: False, *, initial=False):
    img = frame()
    battle = has(img, "current", 686, 96)
    map_name = "时空原点" if has(img, "origin", 675, 122) else "其他地图"
    event("map", map_name if battle else "待识别")
    if initial and not battle:
        raise RuntimeError("请切到原挂机地图的战斗页，再开启自动派遣。")
    if battle and has_return_notice(img):
        event("status", "正在派遣")
        if not has(img, "petpanel", 1344, 93):
            click(1430, 73)
            wait_for(lambda im: has(im, "petpanel", 1344, 93), "未能打开宠物栏。")
        name = img.crop((677, 122, 774, 146)).convert("L")
        world()
        home = next((pos for ref, pos in homes if matches(img, ref, 677, 122)), None)
        if home is None:
            def report(message):
                print(message, flush=True)
                event("log", message)

            home = home_point(img, report, cancelled)
            homes.append((name, home))
        click(363, 457)
        wait_for(lambda im: has(im, "expedition", 42, 58), "未进入蛮荒之地。")
        event("map", "蛮荒之地")
        event("log", "进入蛮荒之地")
        count = 0
        for i in range(3):
            try:
                sent = handle_row(i)
            except Exception:
                event("log", f"第 {i+1} 支小队派遣失败")
                raise
            if sent:
                count += 1
                event("log", f"第 {i+1} 支小队重新出发")
        world()
        click(*home)
        wait_for(lambda im: has(im, "current", 686, 96) and matches(im, name, 677, 122)
                 and not has(im, "world", 276, 751),
                 "回程后地图名称与出发前不一致，请检查。")
        event("map", map_name)
        event("log", f"已回到{map_name}" if map_name != "其他地图" else "已回到原地图")
        print(f"已回到原地图，本次重新派遣 {count} 支小队。", flush=True)
        if not count:
            raise RuntimeError("仍显示返回提示，但没有识别到可收取的小队。")
        return count
    if initial:
        event("log", "未发现小队返回提示，继续监听。")
    return 0


def set_minimized(value):
    U.ShowWindowAsync(hwnd, 7 if value else 4)
    for _ in range(40):
        if bool(U.IsIconic(hwnd)) == value:
            return
        time.sleep(.05)
    raise RuntimeError("窗口状态切换超时。")


def check_once(homes, event=quiet_event, cancelled=lambda: False, *, initial=False):
    try:
        minimized = bool(U.IsIconic(hwnd))
        try:
            if minimized:
                set_minimized(False)
                time.sleep(.4)
                if pid(U.GetForegroundWindow()) == pid(hwnd):
                    raise RuntimeError("还原后游戏获得了焦点，已停止。")
            count = inspect_teams(homes, event, cancelled, initial=initial)
        finally:
            if minimized and U.IsWindow(hwnd):
                set_minimized(True)
    except InterruptedError:
        raise
    except Exception:
        event("failure", "本轮操作失败")
        raise
    if count:
        event("success", f"本轮操作成功，已重新派遣 {count} 支小队。")



def main():
    global hwnd
    hwnd = find_game()
    if not U.PostMessageW(hwnd, 0x200, 0, 0):
        raise RuntimeError("Windows 拒绝后台鼠标消息。请以相同权限运行游戏和脚本。")
    print("每 30 秒检查一次。最小化时会短暂还原，检查后重新最小化。Esc 停止。", flush=True)
    homes = []
    initial = True
    while True:
        check_once(homes, initial=initial)
        initial = False
        if "--once" in sys.argv:
            return
        for _ in range(120):
            tick()
            time.sleep(.25)


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        print("已停止。")
    except Exception as exc:
        sys.exit(f"已停止：{exc}")
