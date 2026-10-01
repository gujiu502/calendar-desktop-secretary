# 日签 · Calendar Desktop Secretary

Windows 10 / 11 本地桌面日历。Tauri 2 + React / TypeScript + SQLite，半透明玻璃界面，默认优先在 Windows 显示器 2 打开。

本次构建的安装包：`release/日签-0.1.1-setup.exe`；免安装启动程序：`release/日签.exe`（需要系统已有 WebView2 Runtime）。

下载安装包与免安装程序：[GitHub Releases](https://github.com/gujiu502/calendar-desktop-secretary/releases/latest)。

## 使用

- 默认月视图，支持日 / 月 / 年切换与今天 / 前后导航。
- 点击月历日期打开快速添加；右上角 ＋ 或 Ctrl N 打开完整编辑器。两者复用同一保存流程。
- 日签支持开始日期、可选结束日期、开始当天的可选时间、标题、内容、分类、优先级、编辑、删除和完成。结束日期包含当天，留空表示单日；跨月、跨年均可。
- 日签覆盖的日期整格上色，月 / 年视图一致；选中状态保留分类颜色，全部完成后整格变灰。下方显示当天所有覆盖事项。Upcoming 显示 5 / 8 / 10 条未完成事项，包含进行中的跨日事项；已开始的事项在当天按全天显示。
- 背景持续半透明，激活和未激活时保持一致。设置可调 35%–85% 不透明度，文字不随背景变淡。
- 关闭按钮默认隐藏到托盘；托盘提供打开、快速添加、今天、设置与退出。
- 开机启动默认关闭，在设置中启用。窗口默认不置顶。
- 免安装版启用自启动后，请保留程序所在路径；移动程序后需重新关闭、启用自启动。
- 数据设置可导出 JSON / CSV、合并导入 JSON、创建今日备份、打开数据目录。

## 运行与构建

需要 Node.js、Rust MSVC、Visual Studio C++ Build Tools，以及 Windows WebView2 Runtime。项目的 rust-toolchain.toml 固定使用 MSVC，避免本机默认 GNU 工具链影响 Windows 构建。

```powershell
npm install
npm run tauri dev
```

```powershell
npm run desktop:build
```

安装程序位于 `src-tauri/target/x86_64-pc-windows-msvc/release/bundle/nsis/`；可直接运行的程序位于 `src-tauri/target/x86_64-pc-windows-msvc/release/calendar-desktop-secretary.exe`。直接运行也需要 WebView2。

`npm run dev` 启动浏览器预览，地址 http://localhost:1420。预览使用独立 localStorage，不会访问桌面 SQLite；原生显示器、托盘、自启动与数据库文件功能仅在桌面版本可用。

## 数据与备份

桌面数据目录：`%APPDATA%/com.calendar.desktopsecretary/`。

- `database.sqlite`：日签、设置和每块显示器的窗口位置。SQLite WAL + 参数化查询，Schema v2。旧版数据库自动增加结束日期列，保留日签与设置；旧 JSON 无结束日期时按单日导入。
- `backup/YYYY-MM-DD.sqlite`：启动时及跨日后台检查时生成，每天至多一份，保留 30 份。使用 SQLite 在线备份 API，避免直接复制正在使用的 WAL 数据库。
- JSON 导入先验证全部字段，在事务中合并，已有 ID 保留。导入前确保今日备份存在。
- CSV 包含 UTF-8 BOM，转义引号、换行，并防止标题或内容触发表格公式。

## 显示器行为

使用 QueryDisplayConfig 获取源 GDI 名称与目标设备路径，再与 EnumDisplayMonitors / GetMonitorInfoW 的工作区匹配。Display 2 从 `\\.\DISPLAY2` 识别，绝不依赖列表下标；位置按设备路径保存。

启动窗口保持隐藏，完成定位与前端加载后再显示。缺少首选显示器时回主屏。WM_DISPLAYCHANGE、WM_SETTINGCHANGE、WM_DPICHANGED 与电源变化触发延迟合并后的重定位；没有持续轮询显示器。每块屏幕保存相对位置、物理尺寸、逻辑尺寸与缩放比例，并钳制到当前工作区。Windows 重新编号或更换设备后，应在设置中核对显示器；手动选定设备路径可避开编号变化。

支持显示器 2 / 主屏 / 上次使用 / 手动选择，以及自动返回、位置记忆、尺寸记忆。窗口直接使用透明背景与前端半透明底色，避免系统材质在非激活状态下变为实色；不叠加系统模糊层。

## 验证

```powershell
npm test
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
```

浏览器交互检查需要已安装 Edge，并在另一个终端运行 `npm run dev`：

```powershell
npm run test:ui
```

测试截图保存在 `work/screenshots/`。具体通过结果及尚需实机验收的矩阵见 `work/verification.md`。模拟测试不代替实际拔插、休眠、不同 Windows 版本和 Explorer 重启测试。

## 实现依据

基底来自 [Tauri 官方 create-tauri-app](https://github.com/tauri-apps/create-tauri-app)，复用官方托盘、自启动、单实例与文件选择能力，日期计算使用 date-fns，持久化使用 rusqlite。候选社区日历项目没有提供许可证，因此没有复制其实现。产品范围以 `docs/EngineeringSpec.md` 为参考，v0.1.1 按用户要求加入结束日期、整格上色与持续半透明；未加入 AI、云同步、账号或服务端。

系统材质的回退行为见 [Microsoft Acrylic 文档](https://learn.microsoft.com/en-us/windows/apps/design/style/acrylic)。本机实测模糊层导致背景接近实色，因此 v0.1.1 使用直接半透明背景，并通过切换背后窗口的红 / 蓝底色验证激活与未激活两种状态。

前端按可见日期范围一次读取日签，日期格在内存中分组，不逐格访问数据库；Upcoming 单独执行带索引的 SQL。设置、备份和显示器状态复用同一 SQLite 文件，不引入独立服务。启动耗时与内存目标仍需在发布程序上实测。
