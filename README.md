# 修仙助手

Windows 桌面助手。Electron + React + TypeScript 负责界面，Rust 负责截图识别、自动派遣、连点和全局热键。

## 目录

~~~text
auto-xiuxian/
├─ desktop/                    Electron 与 React 桌面工程
│  ├─ electron/                主进程、preload、Rust 进程通信
│  ├─ src/
│  │  ├─ components/           React 组件及就近 SCSS
│  │  ├─ types.ts              前端通信类型
│  │  └─ useEngineState.ts     后台状态订阅
│  ├─ assets/                  应用图标
│  ├─ index.html
│  └─ package.json
├─ xiuxian-rs/                 Rust workspace
│  ├─ core/                    派遣、识图、连点、窗口操作
│  ├─ app-server/              main.rs 处理传输，engine.rs 管理状态
│  ├─ app-server-protocol/     请求、命令与状态快照类型
│  ├─ Cargo.toml
│  ├─ Cargo.lock
│  └─ rust-toolchain.toml
├─ scripts/                    开发编排
├─ design/                     设计稿与截图
├─ package.json                工作区统一命令
└─ 修仙助手.exe                最终便携版入口
~~~

Rust 目录参照 Codex 公开仓库的 core、app-server、app-server-protocol 分层。desktop 是本项目的 Electron/React 工程，进程间使用逐行 JSON 通信。参考 [Codex App Server 官方文档](https://learn.chatgpt.com/docs/app-server)。

## 开发

安装 Node.js 22.12+、Rust MSVC 工具链后，在仓库根目录运行。

~~~powershell
npm ci
npx install-electron
npm run dev
~~~

开发模式下 React 支持热更新。修改界面布局可从 desktop/src/components/App/index.tsx 开始，每个组件的样式位于同目录的 index.module.scss。需要操作管理员身份的游戏时，以管理员身份启动开发终端。

## 功能

自动派遣与连点器默认关闭。开启派遣后立即检查，之后每 30 秒检查一次。只有发现小队返回的蓝色提示才进入蛮荒派遣，完成后返回原挂机地图。一次操作计一次成功或失败，失败后等待 30 秒继续轮询。如果未完成回程，下一轮先尝试恢复原地图。首次在其他地图使用时，按运行记录提示用 F8 标记原地图。

游戏最小化时会短暂还原检查，然后重新最小化。连点和热键由 Rust 独立线程处理，Electron 窗口隐藏后继续运行。

“后台运行”、最小化和关闭窗口都隐藏到托盘。双击托盘或再次打开 EXE 恢复已有窗口。托盘菜单“退出”或 Ctrl+Q 会停止连点，等待当前派遣收尾后退出。

## 验证与打包

~~~powershell
npm run check
npm run build
~~~

便携版位于 release/修仙助手.exe，包含 Electron、React 页面、Rust 后台与图标。复制到根目录即可使用单个 EXE 启动。发行版请求管理员权限，以便操作管理员身份运行的游戏。

JavaScript/TypeScript 函数行数由 scripts/check-functions.mjs 检查，Rust 由 workspace Clippy 配置约束，上限均为 200 行。命令校验集中在 desktop/electron/commands.cjs，进程通信与退出异常的回归测试在 desktop/electron/engine.test.cjs。

飞剑祥云图标的设计说明位于 design/icon.md。
