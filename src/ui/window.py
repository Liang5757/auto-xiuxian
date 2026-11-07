from datetime import datetime
import queue
import threading
import time
import tkinter as tk

import dispatch as game
from services.dispatch_worker import run_dispatch
from services.clicker import click_at_cursor
from .clicker_page import ClickerPage
from .dispatch_page import DispatchPage
from .widgets import BG, SIDE, GREEN, MUTED, FONT, Switch, label, rule


class App:
    def __init__(self, root):
        self.root = root
        root.title("修仙助手")
        root.geometry("940x525")
        root.minsize(800, 460)
        root.configure(bg=BG)
        self.events = queue.Queue()
        self.stop = threading.Event()
        self.game_lock = threading.Lock()
        self.worker = None
        self.running = self.clicking = self.closing = False
        self.hot_down = False
        self.page = "自动派遣"
        self.status = "已停止"
        self.map = "待识别"
        self.counts = {"success": 0, "failure": 0}
        self.logs = []
        self.clicks = 0
        self.click_status = "已停止"
        self.interval = tk.StringVar(value="100")
        self.button = tk.StringVar(value="左键")
        self.key = tk.StringVar(value="F6")
        self.next_click = self.next_scan = 0
        self.game_handle = 0
        self.sidebar = tk.Frame(root, width=185, bg=SIDE)
        self.sidebar.pack(side="left", fill="y")
        self.sidebar.pack_propagate(False)
        brand = tk.Frame(self.sidebar, bg=SIDE)
        brand.pack(fill="x", padx=16, pady=(24, 29))
        logo = tk.Canvas(brand, width=30, height=30, bg=SIDE, highlightthickness=0)
        logo.pack(side="left", padx=(0, 8))
        logo.create_polygon(2,8,15,1,28,8,15,16, fill="#c2b2ff")
        logo.create_polygon(2,8,15,16,15,30,2,22, fill="#9a80ef")
        logo.create_polygon(15,16,28,8,28,22,15,30, fill="#7558c8")
        label(brand, "修仙助手", 16, "#ffffff", True).pack(side="left")
        self.nav = {}
        for title, symbol in (("自动派遣", "➤"), ("连点器", "⌖")):
            b = tk.Button(self.sidebar, text=f"{symbol}    {title}", command=lambda t=title:self.select(t),
                          anchor="w", padx=13, font=(FONT, -17), fg="white", bd=0,
                          activebackground="#5b497b", activeforeground="white", cursor="hand2")
            b.pack(fill="x", padx=9, pady=3, ipady=7)
            self.nav[title] = b
        self.connection = label(self.sidebar, "●  正在检测游戏", 10, "#cad0de")
        self.connection.pack(side="bottom", fill="x", padx=18, pady=17)
        self.content = tk.Frame(root, bg=BG, padx=24, pady=18)
        self.content.pack(side="left", fill="both", expand=True)
        self.select(self.page)
        root.protocol("WM_DELETE_WINDOW", self.close)
        self.timer = root.after(20, self.poll)

    def select(self, title):
        self.page = title
        for name, b in self.nav.items():
            b.configure(bg="#645080" if name == title else SIDE)
        for widget in self.content.winfo_children():
            widget.destroy()
        header = tk.Frame(self.content, bg=BG)
        header.pack(fill="x")
        label(header, title, 21, bold=True).pack(side="left")
        self.switch = Switch(header, self.toggle)
        self.switch.pack(side="right")
        self.switch_text = label(header)
        self.switch_text.pack(side="right", padx=8)
        rule(self.content)
        self.state_line = label(self.content)
        self.state_line.pack(fill="x")
        rule(self.content)
        if title == "自动派遣":
            self.view = DispatchPage(self.content)
        else:
            self.view = ClickerPage(self.content, self.interval, self.button, self.key)
        self.refresh()

    def refresh(self):
        dispatch = self.page == "自动派遣"
        on = self.running if dispatch else self.clicking
        self.switch.show(on)
        self.switch_text.config(text="已开启" if on else "已关闭")
        state = self.status if dispatch else self.click_status
        self.state_line.config(text="●  "+state+("     │     每 30 秒检查" if dispatch else ""),
                               fg=GREEN if on else MUTED)
        if dispatch:
            self.view.update(self.counts, self.map, self.logs)
        else:
            self.view.update(self.clicking, self.clicks)

    def log(self, message):
        self.logs.insert(0, datetime.now().strftime("%H:%M:%S")+"     "+message)
        del self.logs[100:]

    def toggle(self):
        if self.page == "连点器":
            self.toggle_clicks()
        elif self.running:
            self.stop.set()
            self.status = "正在停止"
            self.refresh()
        else:
            self.running = True
            self.stop.clear()
            self.status = "正在监听"
            self.log("已开启自动派遣，立即检查小队返回提示")
            self.worker = threading.Thread(target=run_dispatch, args=(self.stop, self.game_lock, self.events), daemon=True)
            self.worker.start()
            self.refresh()

    def toggle_clicks(self):
        if self.clicking:
            self.clicking = False
            self.click_status = "已停止"
        else:
            try:
                self.delay = int(self.interval.get()) / 1000
                if not .03 <= self.delay <= 60:
                    raise ValueError
                self.game_handle = game.find_game()
            except (ValueError, RuntimeError) as exc:
                self.click_status = str(exc) if isinstance(exc,RuntimeError) else "点击间隔应为 30–60000 毫秒"
                self.refresh()
                return
            self.clicking = True
            self.click_status = "等待鼠标指向游戏"
            self.next_click = 0
        self.refresh()

    def poll(self):
        now = time.monotonic()
        dirty = False
        while not self.events.empty():
            kind, value = self.events.get_nowait()
            if kind in self.counts:
                self.counts[kind] += 1
                self.log(value)
            elif kind == "map":
                self.map = value
            elif kind == "log":
                self.log(value)
            elif kind == "done":
                self.running = False
                self.status = value
                self.log(value)
            elif kind == "status" and not self.stop.is_set():
                self.status = value
            dirty = True
        if self.closing and not self.running:
            self.root.destroy()
            return
        pressed = bool(game.U.GetAsyncKeyState(0x70+int(self.key.get()[1:])-1)&0x8000)
        if pressed and not self.hot_down and not self.closing:
            self.toggle_clicks()
        self.hot_down = pressed
        if now >= self.next_scan:
            try:
                self.game_handle = game.find_game()
            except RuntimeError:
                self.game_handle = 0
            self.connection.config(text="●  游戏已连接" if self.game_handle else "○  等待游戏启动",
                                   fg="#9fdfc7" if self.game_handle else "#b9bbc8")
            self.next_scan = now+2
        if self.clicking and now >= self.next_click:
            try:
                sent, self.click_status = click_at_cursor(self.game_handle, self.game_lock, self.button.get())
                self.clicks += int(sent)
            except RuntimeError as exc:
                self.click_status = str(exc)
                self.clicking = False
            self.next_click = now+self.delay
            dirty = self.page == "连点器" or dirty
        if dirty:
            self.refresh()
        self.timer = self.root.after(20,self.poll)

    def close(self):
        self.closing = True
        self.clicking = False
        self.stop.set()
        if self.running:
            self.status = "完成当前操作后关闭"
            self.refresh()
        else:
            self.root.destroy()
