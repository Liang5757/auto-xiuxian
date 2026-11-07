import ctypes as C
from ctypes import wintypes as W

import dispatch as game


def click_at_cursor(game_handle, game_lock, button):
    p = W.POINT()
    game.U.GetCursorPos(C.byref(p))
    target = game.U.WindowFromPoint(p)
    if (not game_handle or not game.U.IsWindow(target)
            or game.pid(target) != game.pid(game_handle)):
        return False, "等待鼠标指向游戏"
    if not game_lock.acquire(False):
        return False, "派遣检查中"
    try:
        if not game.U.ScreenToClient(target,C.byref(p)):
            return False, "等待鼠标指向游戏"
        lp = ((p.y & 65535)<<16) | (p.x & 65535)
        down, up, key = (0x201,0x202,1) if button=="左键" else (0x204,0x205,2)
        if game.U.PostMessageW(target,down,key,lp) and game.U.PostMessageW(target,up,0,lp):
            return True, "正在连点"
        else:
            raise RuntimeError("点击失败，请检查游戏和助手的权限")
    finally:
        game_lock.release()
