import ctypes as C

import dispatch as game
from permissions import permission_status


def run_dispatch(stop, game_lock, events):
    def emit(kind, value):
        events.put((kind, value))
    try:
        waiting = False
        while not stop.is_set():
            try:
                game.hwnd = game.find_game()
                break
            except game.GameNotFound:
                if not waiting:
                    emit("status", "等待游戏启动")
                    emit("log", "等待游戏主窗口，检测到后将检查小队返回提示。")
                    waiting = True
                stop.wait(2)
        if stop.is_set():
            return
        if not game.U.PostMessageW(game.hwnd, 0x200, 0, 0):
            error = C.windll.kernel32.GetLastError()
            if error == 5:
                status = permission_status(game.pid(game.hwnd))
                raise RuntimeError(f"后台输入被拒绝。{status}")
            raise RuntimeError(f"后台输入失败，Windows 错误 {error}。")
        homes = []
        initial = True
        while not stop.is_set():
            with game_lock:
                game.check_once(homes, emit, stop.is_set, initial=initial)
            initial = False
            emit("status", "正在监听")
            if stop.wait(30):
                break
    except (Exception, KeyboardInterrupt) as exc:
        emit("log", str(exc) or "已中断")
    finally:
        emit("done", "已停止")
