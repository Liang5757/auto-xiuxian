# 项目协作约定

- 桌面界面与 Electron 在 desktop/，Rust 核心、服务入口和协议在 xiuxian-rs/ 的独立 crate。
- JavaScript、TypeScript 与 Rust 函数不超过 200 行，代码变更后运行 npm run check 并审查 diff。
- 产品行为变更同步写入 CHANGELOG.md 的“未发布”部分，发布时归档为带日期的版本条目。
- 发布版本同步根目录 package.json、desktop/package.json、Rust workspace 及锁文件。
- 正式发布使用 vX.Y.Z 标签，GitHub Release 提供 Windows x64 EXE、SHA256SUMS.txt 和对应更新记录。
- 已发布版本保持可追溯，修复以新版本发布。可执行文件通过 Release 分发，构建输出由 .gitignore 排除。
