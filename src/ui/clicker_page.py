import tkinter as tk
from tkinter import ttk
from .widgets import BG, MUTED, label, rule


class ClickerPage:
    def __init__(self, parent, interval, button, key):
        self.click_fields = []
        for text, var, choices in (("点击间隔（毫秒）", interval, None),
                                   ("鼠标按键", button, ("左键","右键")),
                                   ("启停热键", key, ("F6","F7","F9"))):
            row = tk.Frame(parent, bg=BG)
            row.pack(fill="x", pady=6)
            label(row, text, width=17).pack(side="left")
            entry = (ttk.Combobox(row, textvariable=var, values=choices, state="readonly", width=11)
                     if choices else ttk.Entry(row, textvariable=var, width=13))
            entry.pack(side="left")
            self.click_fields.append(entry)
        rule(parent)
        self.click_value = label(parent, size=14, bold=True)
        self.click_value.pack(fill="x")
        label(parent, "鼠标指向游戏窗口时连点。自动派遣检查期间暂停连点。", 10, MUTED).pack(fill="x", pady=8)

    def update(self, clicking, clicks):
        for field in self.click_fields:
            state = "readonly" if isinstance(field, ttk.Combobox) else "normal"
            field.configure(state="disabled" if clicking else state)
        self.click_value.config(text=f"已点击   {clicks} 次")
