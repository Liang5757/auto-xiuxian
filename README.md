# 修仙助手

Windows 桌面助手。双击根目录的 `修仙助手.exe` 打开界面。自动派遣与连点器启动时均关闭。发行版请求管理员权限，以便给管理员模式的游戏发送窗口消息。

开启自动派遣后立即检查，之后每 30 秒检查一次。识别到小队从蛮荒返回的蓝色提示后，助手进入蛮荒，重新派遣并返回原挂机地图。最小化的游戏会短暂还原并在检查后重新最小化。首次在其他挂机地图触发派遣时，根据日志提示用 F8 标记回程位置。每轮操作计数一次，空轮询不计数。

## 构建

安装 Windows MSVC 版 Rust 工具链，在仓库根目录运行：

```powershell
cargo test --locked --release
cargo build --locked --release --features admin
Copy-Item target/release/xiuxian.exe 修仙助手.exe
```

模板已嵌入可执行文件。运行时只需该 EXE。界面在 `src/main.rs`，派遣流程在 `src/session.rs`，Win32 窗口操作和图像识别分别在 `src/native.rs`、`src/vision.rs`。
