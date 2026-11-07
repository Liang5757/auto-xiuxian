import tkinter as tk
from .widgets import BG, INK, MUTED, LINE, FONT, label, rule


class DispatchPage:
    def __init__(self, parent):
        metrics = tk.Frame(parent, bg=BG)
        metrics.pack(fill="x")
        self.values = []
        for i, name in enumerate(("成功", "失败", "当前地图")):
            metrics.columnconfigure(i, weight=1, uniform="stat")
            card = tk.Frame(metrics, bg=BG, padx=12)
            card.grid(column=i, row=0, sticky="ew")
            if i:
                tk.Frame(metrics, width=1, bg=LINE).grid(column=i,row=0,sticky="nsw")
            label(card, name, 10, MUTED).pack(fill="x")
            v = label(card, size=17, bold=True)
            v.pack(fill="x")
            self.values.append(v)
        rule(parent)
        label(parent, "最近活动", 14, bold=True).pack(fill="x", pady=(0,8))
        self.history = tk.Listbox(parent, bg="#fcfcff", fg=INK, bd=0, highlightthickness=1,
                                  highlightbackground=LINE, font=(FONT, -16), activestyle="none")
        self.history.pack(fill="both", expand=True)

    def update(self, counts, map_name, logs):
        for v, value in zip(self.values,(counts["success"], counts["failure"], map_name)):
            v.config(text=value)
        self.history.delete(0, "end")
        for entry in logs or ["启动自动派遣后，此处显示运行记录。"]:
            self.history.insert("end", "  "+entry)
