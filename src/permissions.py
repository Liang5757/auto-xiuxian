"""只读查询 Windows 进程的 TokenElevation。"""
import ctypes as C
import os
from ctypes import wintypes as W

kernel = C.WinDLL("kernel32", use_last_error=True)
security = C.WinDLL("advapi32", use_last_error=True)
kernel.OpenProcess.argtypes = (W.DWORD, W.BOOL, W.DWORD)
kernel.OpenProcess.restype = W.HANDLE
kernel.CloseHandle.argtypes = (W.HANDLE,)
security.OpenProcessToken.argtypes = (W.HANDLE, W.DWORD, C.POINTER(W.HANDLE))
security.GetTokenInformation.argtypes = (
    W.HANDLE, C.c_int, W.LPVOID, W.DWORD, C.POINTER(W.DWORD)
)


def is_elevated(pid):
    process = kernel.OpenProcess(0x1000, False, pid)  # PROCESS_QUERY_LIMITED_INFORMATION
    if not process:
        raise C.WinError(C.get_last_error())
    token = W.HANDLE()
    try:
        if not security.OpenProcessToken(process, 0x0008, C.byref(token)):  # TOKEN_QUERY
            raise C.WinError(C.get_last_error())
        value, size = W.DWORD(), W.DWORD()
        if not security.GetTokenInformation(token, 20, C.byref(value), C.sizeof(value), C.byref(size)):
            raise C.WinError(C.get_last_error())
        return bool(value.value)
    finally:
        if token.value:
            kernel.CloseHandle(token)
        kernel.CloseHandle(process)


def permission_status(game_pid):
    def describe(pid):
        return "管理员权限" if is_elevated(pid) else "普通权限"

    return f"游戏（PID {game_pid}）：{describe(game_pid)}；当前程序（PID {os.getpid()}）：{describe(os.getpid())}"


if __name__ == "__main__":
    import dispatch
    try:
        print(permission_status(dispatch.pid(dispatch.find_game())))
    except (OSError, RuntimeError) as exc:
        raise SystemExit(str(exc))
