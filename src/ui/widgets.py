import tkinter as tk

BG, SIDE, PURPLE = "#f7f7fc", "#252631", "#7954d6"
INK, MUTED, LINE = "#242532", "#676a7b", "#dbdce6"
GREEN, RED = "#26865a", "#b53e50"
FONT = "Microsoft YaHei UI"


def label(parent, text="", size=11, color=INK, bold=False, **kw):
    return tk.Label(parent, text=text, font=(FONT, -round(size * 1.45), "bold" if bold else "normal"),
                    bg=parent.cget("bg"), fg=color, anchor="w", **kw)


def rule(parent):
    tk.Frame(parent, height=1, bg=LINE).pack(fill="x", pady=12)


class Switch(tk.Canvas):
    def __init__(self, parent, action):
        super().__init__(parent, width=50, height=28, bg=BG, highlightthickness=0,
                         takefocus=True, cursor="hand2")
        for sequence in ("<Button-1>", "<space>", "<Return>"):
            self.bind(sequence, lambda _: action())
        self.bind("<FocusIn>", lambda _: self.configure(highlightthickness=1))
        self.bind("<FocusOut>", lambda _: self.configure(highlightthickness=0))

    def show(self, on):
        self.delete("all")
        fill = PURPLE if on else "#a3a5b5"
        self.create_line(14, 14, 36, 14, width=24, capstyle="round", fill=fill)
        x = 36 if on else 14
        self.create_oval(x-9, 5, x+9, 23, fill="white", outline="")
