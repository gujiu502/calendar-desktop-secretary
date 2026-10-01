# Calendar Desktop Secretary 工程设计规格

> 日期：2026-10-01  
> 平台：Windows 10 / Windows 11  
> 目标：制作一个以「日历 + Todo / 日签」为核心的半透明桌面秘书应用。  
> 原则：优先基于成熟开源项目修改与组合，不从零重复实现已经成熟的日历、窗口管理、SQLite、系统托盘、开机启动等能力。

---

# 1. 项目目标

制作一个长期常驻桌面的 TodoList + 日历应用。

应用正常状态下以「日历」作为主界面，用户可以：

- 查看日 / 月 / 年视图。
- 点击日期。
- 为某一天创建「日签」。
- 日签可以包含日期、时间、标题、内容、状态等信息。
- 某一天存在日签时，该日期格自动变色。
- 点击某个日期后，在日历下方显示该日期的全部记录。
- 右侧自动排列距离现在最近的 5～10 个未完成日签。
- Upcoming 区域显示日期、时间和标题。
- 整个 UI 使用半透明 / 毛玻璃效果，可以看到桌面壁纸。
- 应用启动时优先自动出现在 Windows 的「显示器 2」。
- 显示器 2 不存在时自动回退到主屏幕。
- 显示器 2 再次连接后，可以自动移回显示器 2。
- 记住每块显示器上的窗口位置与尺寸。
- 支持开机自动启动。
- 支持系统托盘常驻。

---

# 2. 核心设计原则

## 2.1 不从零造轮子

尽量复用成熟项目中的以下能力：

- Tauri 2 桌面壳
- React 前端
- SQLite 数据持久化
- 日历组件
- Todo / Task 数据模型
- Windows 多显示器 API
- 系统托盘
- 开机启动
- Mica / Acrylic / Vibrancy
- 窗口位置保存
- DPI / 多屏缩放处理

真正需要自行实现的重点主要是：

1. 日签业务逻辑。
2. 日历日期格着色逻辑。
3. Upcoming 最近事项排序逻辑。
4. 日 / 月 / 年视图交互。
5. Windows Display 2 优先绑定。
6. UI 重新设计。
7. 各模块之间的整合。

---

# 3. 推荐技术栈

## Desktop Framework

```text
Tauri 2
```

原因：

- Windows 原生桌面应用。
- 比 Electron 更轻。
- 可以调用 Rust / Win32 API。
- 支持多显示器。
- 支持透明窗口。
- 支持系统托盘。
- 支持开机自启动。
- 可以使用 SQLite。
- 前端仍然可以使用 React。

---

## Frontend

```text
React
TypeScript
Vite
```

推荐 UI：

```text
Tailwind CSS
```

或者：

```text
CSS Modules
```

不要依赖过重的 UI Framework。

原因是最终 UI 是高度定制的玻璃桌面风格。

---

## Calendar

优先从成熟组件中选择，例如：

```text
FullCalendar
React Big Calendar
Day.js
date-fns
```

但本项目的主界面并不是传统 Outlook / Google Calendar 时间轴。

因此建议：

```text
Month Grid：自定义
Day / Year View：自定义
日期计算：date-fns / Day.js
```

也就是说：

```text
日期逻辑复用成熟库
UI 网格自己定制
```

这样才能做出桌面 Widget 风格。

---

## Database

```text
SQLite
```

推荐：

```text
Tauri SQL Plugin
```

或者：

```text
Rust + rusqlite
```

SQLite 文件示例：

```text
%APPDATA%/
CalendarDesktopSecretary/
database.sqlite
```

---

# 4. 推荐工程组合策略

不要寻找一个「100% 完全符合需求」的项目。

更实际的方式：

```text
成熟 Calendar/Todo 项目
        +
透明桌面 Todo 项目
        +
Tauri 官方窗口能力
        +
Windows Display API
        ↓
Calendar Desktop Secretary
```

可以分别借用：

### A. Calendar / Todo 项目

保留：

- React 架构
- SQLite
- Task CRUD
- Settings
- 日历日期逻辑
- 数据同步逻辑

删除：

- 不需要的 Dashboard
- 项目管理
- 团队功能
- 云端帐号
- 在线服务

---

### B. Floating Todo 类项目

借用：

- 半透明窗口
- Always-on-desktop
- 系统托盘
- 启动隐藏
- 开机启动
- 窗口状态保存

---

### C. Tauri Window Vibrancy

负责：

```text
Windows 11 Mica
Windows Acrylic
Blur
透明背景
```

---

### D. Windows Native Monitor API

负责：

```text
Display 1 / Display 2
显示器拓扑变化
DPI
Work Area
窗口移动
屏幕重新连接
```

---

# 5. 最终界面结构

推荐主窗口：

```text
┌─────────────────────────────────────────────────────────────┐
│  <            October 2026             >      日 月 年   ＋ │
│                                                             │
│  一      二      三      四      五      六      日           │
│                                                             │
│          1       2       3       4       5                   │
│                                                             │
│   6      7       8       9      10      11      12           │
│                                                             │
│  13     14      15      16      17      18      19           │
│                                                             │
│  20     21      22      23      24      25      26           │
│                                                             │
│  27     28      29      30      31                           │
│                                                             │
│───────────────────────────────────────┬─────────────────────│
│  2026 / 10 / 01                      │ Upcoming            │
│                                       │                     │
│  ● 14:00 微积分作业                   │ 10/02 09:00         │
│     Chapter 3                         │ 电子学作业           │
│                                       │                     │
│  ● 20:30 写 ESP32                     │ 10/03 13:30         │
│     测试 WiFi                         │ 实验报告             │
│                                       │                     │
│                ＋ 添加日签             │ 10/05 20:00         │
│                                       │ Project             │
│                                       │                     │
└───────────────────────────────────────┴─────────────────────┘
```

布局比例建议：

```text
主区域：75%
Upcoming：25%
```

---

# 6. 半透明 UI

## 目标效果

不是简单：

```css
opacity: 0.8;
```

因为那会导致：

```text
文字也一起透明
```

正确方法是：

```text
Window Transparency
+
Backdrop Blur
+
Semi-transparent Panels
```

推荐：

```text
Windows 11:
Mica / Acrylic

Windows 10:
Blur / Acrylic fallback
```

CSS：

```css
.calendar-panel {
    background: rgba(20, 20, 24, 0.50);
    backdrop-filter: blur(24px);
    border: 1px solid rgba(255, 255, 255, 0.10);
    border-radius: 18px;
}
```

文字保持：

```text
100% 不透明
```

---

# 7. 月视图

默认启动页面：

```text
Month View
```

顶部：

```text
<   October 2026   >
```

右边：

```text
日 | 月 | 年
```

---

## 日期格状态

### 普通日期

```text
透明
```

### 今天

```text
高亮边框
```

### 被选择日期

```text
更明显高亮
```

### 有日签

改变日期格颜色。

例如：

```text
无日签：
rgba(255,255,255,0.03)

有日签：
rgba(80,150,255,0.22)

今天：
border: 2px solid accent-color

选择：
rgba(255,255,255,0.16)
```

---

# 8. 日期颜色规则

建议不要简单使用：

```text
有任务 = 蓝色
```

以后应该支持不同状态。

例如：

```text
普通任务       蓝色
重要任务       红色
学习           紫色
生活           绿色
已完成         灰色
```

如果一天有多个分类：

```text
日期格底部显示多个小圆点
```

例如：

```text
15

● ● ●
```

这样比整个日期格变成一种颜色更容易扩展。

第一版同时支持：

```text
日期格轻微变色
+
底部分类圆点
```

---

# 9. 日签定义

「日签」是本项目的核心业务对象。

它不是单纯 Todo。

建议定义：

```text
DailyNote
```

---

## 数据字段

```ts
interface DailyNote {
    id: string;

    date: string;
    time?: string;

    title: string;
    content?: string;

    category?: string;

    priority: 0 | 1 | 2 | 3;

    completed: boolean;

    createdAt: string;
    updatedAt: string;
}
```

---

# 10. SQLite 数据结构

```sql
CREATE TABLE daily_notes (
    id TEXT PRIMARY KEY,

    date TEXT NOT NULL,
    time TEXT,

    title TEXT NOT NULL,
    content TEXT,

    category TEXT,

    priority INTEGER DEFAULT 0,

    completed INTEGER DEFAULT 0,

    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

索引：

```sql
CREATE INDEX idx_daily_notes_date
ON daily_notes(date);

CREATE INDEX idx_daily_notes_datetime
ON daily_notes(date, time);

CREATE INDEX idx_daily_notes_completed
ON daily_notes(completed);
```

---

# 11. 点击日期行为

用户点击：

```text
October 15
```

执行：

```text
selectedDate = 2026-10-15
```

然后下方区域刷新：

```text
2026 / 10 / 15

09:00 微积分
13:00 电子实验
20:00 写 Project
```

---

# 12. 添加日签

日签只有一套真正的创建逻辑：

```text
openDailyNoteEditor(targetDate)
```

但 UI 提供两个入口。

---

## 12.1 入口 A：右上角固定 `＋`

主窗口右上角始终显示：

```text
日 | 月 | 年 | ＋
```

`＋` 是主要的新建入口。

点击：

```text
＋
```

执行：

```text
如果当前已经 selectedDate
    → targetDate = selectedDate

否则
    → targetDate = today

↓

打开 DailyNoteEditor
```

例如当前选中：

```text
2026-10-15
```

点击右上角：

```text
＋
```

编辑器自动带入：

```text
日期：2026-10-15
```

---

## 12.2 入口 B：直接点击日期

用户点击月历中的任意日期格，例如：

```text
October 18
```

执行：

```text
selectedDate = 2026-10-18

↓

下方当天记录立即切换到 2026-10-18

↓

同时打开 DailyNoteEditor

↓

targetDate = 2026-10-18
```

也就是说：

> 点击日期本身，也可以直接添加该日期的日签。

不需要：

```text
先点日期
↓
再去找另外一个添加按钮
```

---

## 12.3 两个入口必须共用同一编辑器

禁止实现成：

```text
右上角 ＋ → Editor A

点击日期 → Editor B
```

正确方式：

```text
右上角 ＋ ──────┐
                 ├──> openDailyNoteEditor(targetDate)
点击日期 ───────┘
```

这样：

- 表单验证只有一套。
- SQLite INSERT 只有一套。
- UI 不会出现行为不一致。
- 后续增加提醒、分类、重复任务时只修改一个 Editor。

---

## 12.4 DailyNoteEditor

建议使用：

```text
Modal
```

或：

```text
右侧 / 中央浮动 Glass Panel
```

不要跳转到新页面。

界面：

```text
┌──────────────────────────┐
│ 新建日签              ×  │
│                          │
│ 日期  2026 / 10 / 18     │
│ 时间  14 : 00            │
│                          │
│ 标题                      │
│ [ 微积分作业           ]  │
│                          │
│ 内容                      │
│ [ Chapter 3           ]   │
│                          │
│ 分类 [学习 ▼]             │
│                          │
│        取消     保存       │
└──────────────────────────┘
```

---

## 12.5 日期字段

通过日期格进入时：

```text
日期 = 被点击日期
```

通过右上角 `＋` 进入时：

```text
日期 = selectedDate
```

没有 selectedDate 时：

```text
日期 = today
```

用户仍然可以在编辑器中手动修改日期。

---

## 12.6 点击日期时保留当天记录功能

虽然点击日期会打开新增日签窗口，但原本的：

```text
点击日期
→ 下方显示当天记录
```

功能仍然保留。

完整行为：

```text
Click CalendarCell

↓

setSelectedDate(date)

↓

刷新 DailyNoteList(date)

↓

打开 DailyNoteEditor(date)
```

因此关闭新增窗口以后：

```text
下方已经是刚才那个日期的全部记录
```

---

## 12.7 防止误触

因为月历日期格很常被用来“查看”，建议设置：

```text
单击日期：
    选中日期
    显示当天记录
    打开轻量 Quick Add

Quick Add：
    默认只显示
    ├─ 时间
    ├─ 标题
    └─ 保存

点击“更多”：
    展开完整 DailyNoteEditor
```

这样不会让每次点日期都出现一个很重的表单。

右上角 `＋`：

```text
直接打开完整 DailyNoteEditor
```

这两种方式最后仍然调用相同的：

```text
createDailyNote()
```

---

## 12.8 Quick Add 推荐界面

点击日期：

```text
October 18
```

日期格附近弹出：

```text
┌──────────────────────┐
│ 10 / 18              │
│                      │
│ 时间 [14:00]         │
│                      │
│ [输入日签标题......] │
│                      │
│ 更多            保存 │
└──────────────────────┘
```

这样可以实现：

```text
点击日期
↓
输入标题
↓
Enter
```

快速完成新增。

---

## 12.9 保存流程

无论来自：

```text
右上角 ＋
```

还是：

```text
点击日期
```

保存后统一执行：

```text
validate()
↓
SQLite INSERT
↓
更新当天 DailyNoteList
↓
更新 Calendar Date Indicator
↓
重新计算 Upcoming
↓
关闭 Editor / Quick Add
```

整个过程不刷新页面。

---

## 12.10 键盘操作

建议支持：

```text
Ctrl + N
```

等价于：

```text
点击右上角 ＋
```

Quick Add 中：

```text
Enter
→ 保存

Esc
→ 关闭
```

完整 Editor 中：

```text
Ctrl + Enter
→ 保存

Esc
→ 关闭
```

---

# 13. Upcoming 模块

右侧显示：

```text
未来最近的 5～10 个未完成日签
```

建议默认：

```text
8 个
```

用户设置：

```text
5
8
10
```

---

## 排序规则

```text
NOW
 ↓

按：
date ASC
time ASC
priority DESC
```

例如当前：

```text
2026-10-01 16:00
```

数据库：

```text
10/01 20:00
10/02 09:00
10/02 12:00
10/05 08:00
```

Upcoming：

```text
今天 20:00
明天 09:00
明天 12:00
10/05 08:00
```

---

# 14. 已完成任务

点击：

```text
✓
```

日签：

```text
completed = true
```

默认：

```text
Upcoming 不显示
```

当天记录中：

```text
仍然保留
```

显示为：

```text
~~微积分作业~~
```

或者：

```text
降低透明度
```

---

# 15. 日视图

点击：

```text
日
```

显示：

```text
2026 / 10 / 01

08:00
09:00
10:00
11:00
12:00
13:00
14:00   微积分作业
15:00
16:00
17:00
18:00
19:00
20:00   ESP32 Project
```

第一版可以不用复杂拖拽。

只需要：

```text
查看
添加
编辑
删除
完成
```

---

# 16. 年视图

点击：

```text
年
```

显示 12 个 mini calendar：

```text
Jan   Feb   Mar
Apr   May   Jun
Jul   Aug   Sep
Oct   Nov   Dec
```

存在日签的日期：

```text
显示小圆点
```

点击月份：

```text
进入 Month View
```

点击日期：

```text
进入 Day / Month selected date
```

---

# 17. 多显示器系统

这是本项目核心需求之一。

要求：

```text
程序启动
↓
检测 Windows Display 2
↓
如果存在
    → 主窗口在 Display 2 打开

如果不存在
    → 主窗口在 Primary Monitor 打开
```

---

# 18. 不允许直接 monitors[1]

不要写：

```ts
const monitor = monitors[1];
```

因为：

```text
API 枚举顺序
```

不一定等于：

```text
Windows 设置里的显示器编号
```

因此需要：

```text
Native Windows Monitor Manager
```

---

# 19. Monitor Manager

Rust 模块：

```text
src-tauri/
└── src/
    └── monitor/
        ├── mod.rs
        ├── detector.rs
        ├── windows_display.rs
        └── position_store.rs
```

职责：

```text
MonitorManager

├── 获取 Windows Display Configuration
├── 找到 Display 2
├── 获取 monitor bounds
├── 获取 work area
├── 获取 DPI
├── 获取 scaling
├── 获取 monitor ID
├── 保存窗口位置
├── 监听显示器变化
└── 控制窗口移动
```

---

# 20. 启动逻辑

```text
App Start

↓
加载 Settings

↓
初始化 SQLite

↓
初始化 Monitor Manager

↓
QueryDisplayConfig()

↓
寻找 preferred_monitor

preferred_monitor = 2

↓
Display 2 Found?
```

### Yes

```text
读取 Display 2 上一次窗口位置

↓

验证位置是否仍然位于 Display 2 WorkArea

↓

移动窗口

↓

Show Window
```

### No

```text
Primary Monitor

↓

读取 Primary Monitor 上一次位置

↓

Show Window
```

---

# 21. 防止窗口闪到主屏

错误做法：

```text
创建窗口
↓
显示
↓
再移动到 Screen 2
```

用户会看到窗口：

```text
Screen 1 闪一下
↓
Screen 2
```

正确方法：

```text
创建 hidden window

↓

检测 monitor

↓

set_position()

↓

set_size()

↓

show()
```

因此：

```text
启动时完全不会在 Screen 1 闪现
```

---

# 22. 显示器重新连接

例如：

```text
笔电 + 外接屏
```

然后拔掉外接屏。

系统：

```text
Display 2 removed
↓
检测当前窗口已经不存在有效屏幕
↓
移动到 Primary
```

重新插入：

```text
Display 2 connected
↓
MonitorManager 收到显示变化
↓
等待 Windows 布局稳定
↓
检测 Display 2
↓
恢复 Display 2 上一次窗口位置
```

---

# 23. Display Settings

设置页面：

```text
显示器

默认显示位置：

● Windows Display 2
○ Primary Display
○ Last Used Display
○ Manual

[x] Display 2 连接后自动移回
[x] 记住每块显示器的位置
[x] 记住窗口尺寸
```

---

# 24. Monitor Position 数据

保存：

```json
{
    "display_1": {
        "x": 100,
        "y": 100,
        "width": 1100,
        "height": 700
    },

    "display_2": {
        "x": 2050,
        "y": 80,
        "width": 1100,
        "height": 700
    }
}
```

更成熟版本应该使用：

```text
Monitor Stable ID
```

而不是只保存：

```text
display_2
```

因为用户可能更换显示器。

---

# 25. DPI / 缩放

必须正确处理：

```text
100%
125%
150%
175%
200%
```

不要直接把：

```text
Physical Pixels
```

当成：

```text
Logical Pixels
```

否则：

```text
Screen 1 = 100%
Screen 2 = 150%
```

窗口移动后尺寸会错。

Monitor Manager 必须保存：

```text
physical size
logical size
scale factor
```

---

# 26. 主窗口行为

推荐：

```text
Resizable        true
Decorations      false
Transparent      true
AlwaysOnTop      false
SkipTaskbar      false
```

可选：

```text
Always on desktop
```

不要默认：

```text
AlwaysOnTop
```

否则会覆盖游戏 / IDE。

---

# 27. 桌面模式

未来可以增加：

```text
Desktop Mode
```

打开后：

```text
窗口固定在桌面层
```

效果：

```text
像桌面 Widget
```

而不是：

```text
永远压在所有程序最前面
```

---

# 28. Title Bar

因为：

```text
decorations = false
```

需要自行实现标题栏。

推荐：

```text
┌────────────────────────────────────────────┐
│ Calendar                        — □ ×      │
└────────────────────────────────────────────┘
```

但是默认可以把控制按钮做得很淡。

鼠标移入：

```text
按钮提高亮度
```

---

# 29. 窗口拖动

顶部区域：

```text
data-tauri-drag-region
```

让用户拖动窗口。

---

# 30. 系统托盘

右下角 Tray：

```text
Calendar Desktop Secretary
```

菜单：

```text
打开
快速添加日签
今天
设置
退出
```

关闭窗口默认行为：

```text
隐藏到 Tray
```

而不是：

```text
结束程序
```

---

# 31. 开机启动

设置：

```text
[x] 开机自动启动
```

启动方式：

```text
Windows Login
↓
应用启动
↓
检测 Display 2
↓
直接在 Display 2 打开
```

---

# 32. 设置页面

建议：

```text
General
Display
Appearance
Calendar
Data
```

---

## General

```text
[x] 开机启动
[x] 关闭按钮最小化到 Tray
[x] 启动后自动显示窗口
```

---

## Display

```text
Preferred Display

Display 1
Display 2
Primary
Last Used

[x] Display 2 reconnect 后自动移动
[x] 记住每块显示器的位置
```

---

## Appearance

```text
Transparency   [──────●──]

Blur
● Mica
○ Acrylic
○ Blur
○ None

Corner Radius
```

---

## Calendar

```text
Week starts on:

● Monday
○ Sunday

Upcoming Count:

5
8
10
```

---

## Data

```text
Export
Import
Backup
Open Database Folder
```

---

# 33. 软件架构

```text
┌─────────────────────────┐
│       React UI          │
│                         │
│ Calendar                │
│ Daily Notes             │
│ Upcoming                │
│ Settings                │
└────────────┬────────────┘
             │
             │ Tauri IPC
             │
┌────────────▼────────────┐
│       Rust Core         │
│                         │
│ SQLite                  │
│ Monitor Manager         │
│ Window Manager          │
│ Tray                    │
│ Auto Start              │
│ Settings                │
└────────────┬────────────┘
             │
┌────────────▼────────────┐
│      Windows API        │
│                         │
│ QueryDisplayConfig      │
│ EnumDisplayMonitors     │
│ GetMonitorInfo          │
│ WM_DISPLAYCHANGE        │
│ DPI API                 │
└─────────────────────────┘
```

---

# 34. React 模块

```text
src/
├── app/
│   ├── App.tsx
│   └── routes.ts
│
├── components/
│   ├── calendar/
│   │   ├── CalendarHeader.tsx
│   │   ├── MonthView.tsx
│   │   ├── DayView.tsx
│   │   ├── YearView.tsx
│   │   ├── MonthGrid.tsx
│   │   ├── CalendarCell.tsx
│   │   └── MiniMonth.tsx
│   │
│   ├── notes/
│   │   ├── DailyNoteList.tsx
│   │   ├── DailyNoteItem.tsx
│   │   ├── DailyNoteEditor.tsx
│   │   └── AddNoteButton.tsx
│   │
│   ├── upcoming/
│   │   ├── UpcomingPanel.tsx
│   │   └── UpcomingItem.tsx
│   │
│   └── window/
│       ├── TitleBar.tsx
│       └── WindowControls.tsx
│
├── stores/
│   ├── calendarStore.ts
│   ├── noteStore.ts
│   └── settingsStore.ts
│
├── services/
│   ├── database.ts
│   ├── monitor.ts
│   └── settings.ts
│
└── styles/
    ├── glass.css
    └── calendar.css
```

---

# 35. Rust 模块

```text
src-tauri/
└── src/
    ├── main.rs
    │
    ├── database/
    │   ├── mod.rs
    │   ├── migration.rs
    │   └── daily_notes.rs
    │
    ├── monitor/
    │   ├── mod.rs
    │   ├── detector.rs
    │   ├── windows_display.rs
    │   └── position_store.rs
    │
    ├── window/
    │   ├── mod.rs
    │   ├── placement.rs
    │   └── vibrancy.rs
    │
    ├── tray/
    │   └── mod.rs
    │
    ├── autostart/
    │   └── mod.rs
    │
    └── settings/
        └── mod.rs
```

---

# 36. State Management

不需要 Redux。

推荐：

```text
Zustand
```

Store：

```text
CalendarStore
DailyNoteStore
SettingsStore
```

---

# 37. CalendarStore

```ts
interface CalendarState {
    selectedDate: Date;
    currentMonth: Date;

    viewMode:
        | "day"
        | "month"
        | "year";

    setSelectedDate(date: Date): void;

    next(): void;
    previous(): void;
    today(): void;
}
```

---

# 38. DailyNoteStore

```ts
interface DailyNoteState {
    notes: DailyNote[];

    loadByDate(date: string): Promise<void>;

    create(note: DailyNote): Promise<void>;

    update(note: DailyNote): Promise<void>;

    remove(id: string): Promise<void>;

    complete(id: string): Promise<void>;
}
```

---

# 39. Upcoming Query

SQL 示例：

```sql
SELECT *
FROM daily_notes

WHERE completed = 0

AND (
    date > :today

    OR

    (
        date = :today
        AND (
            time IS NULL
            OR time >= :current_time
        )
    )
)

ORDER BY
    date ASC,
    CASE
        WHEN time IS NULL THEN '23:59'
        ELSE time
    END ASC,
    priority DESC

LIMIT :limit;
```

---

# 40. 日期格查询优化

不要每天分别：

```text
SELECT
```

一个月 42 格就查询 42 次。

错误：

```text
42 cells
=
42 SQLite Queries
```

正确：

```sql
SELECT date,
       COUNT(*) AS note_count
FROM daily_notes

WHERE date BETWEEN :start AND :end

GROUP BY date;
```

一次得到整个月的数据。

---

# 41. 本地优先

第一版完全：

```text
Local First
```

不要求：

```text
帐号
服务器
云端 API
网络连接
```

所有数据：

```text
本地 SQLite
```

这样启动速度和稳定性最好。

---

# 42. Backup

自动备份：

```text
database.sqlite

↓

backup/
2026-10-01.sqlite
2026-10-02.sqlite
```

建议：

```text
每天最多 1 个
保留最近 30 个
```

---

# 43. Export

支持：

```text
JSON
CSV
```

未来：

```text
ICS
```

可以和：

```text
Google Calendar
Outlook
Apple Calendar
```

交换数据。

---

# 44. 第一阶段不要做的东西

为了防止工程膨胀，V1 不加入：

```text
AI 自动规划
Google Calendar Sync
手机 App
帐号系统
服务器
团队协作
复杂提醒规则
语音助手
在线数据库
插件市场
```

先把桌面日历做到真正稳定。

---

# 45. V1 功能范围

## 必须完成

```text
[x] Windows Desktop App

[x] 半透明 UI

[x] Mica / Acrylic

[x] Month View

[x] Day View

[x] Year View

[x] 点击日期

[x] 右上角 ＋ 添加日签

[x] 点击日期直接添加日签

[x] 编辑日签

[x] 删除日签

[x] 完成日签

[x] 日期格有日签自动变色

[x] 当前日期高亮

[x] 下方显示当天记录

[x] Upcoming 最近 5～10 项

[x] SQLite

[x] Display 2 优先

[x] Display 2 fallback

[x] Display 2 reconnect

[x] 每块屏幕记忆位置

[x] DPI Scaling

[x] System Tray

[x] Auto Start

[x] 设置页面

[x] 数据 Export

[x] Backup
```

---

# 46. 第二阶段

可以增加：

```text
桌面固定模式

提醒通知

Recurring Task

拖拽日期

搜索

快捷键

Quick Add

标签

分类

Priority

ICS Import / Export
```

---

# 47. 第三阶段

未来才考虑：

```text
AI Secretary

PDF 行事历导入

自动读取课程表

自动安排 Todo

Google Calendar

学校 Moodle / iStudy

邮件事项提取

自然语言添加任务
```

例如：

```text
明天下午三点写物理报告
```

AI 转成：

```json
{
    "date": "2026-10-02",
    "time": "15:00",
    "title": "写物理报告"
}
```

---

# 48. UI 性能目标

启动：

```text
目标 < 1 秒
```

Calendar 切换：

```text
目标 < 50 ms
```

日期点击：

```text
目标 < 30 ms
```

SQLite 查询：

```text
普通查询 < 20 ms
```

内存：

```text
目标 < 150 MB
```

闲置 CPU：

```text
接近 0%
```

---

# 49. 稳定性要求

以下情况不允许崩溃：

```text
拔掉 Screen 2

插入 Screen 2

修改 Windows 缩放

修改分辨率

Display 1 / Display 2 位置改变

休眠

唤醒

锁屏

解锁

Explorer 重启

数据库为空

没有 Upcoming

日签没有时间

窗口位置落在不存在的屏幕
```

---

# 50. Monitor 测试矩阵

必须测试：

```text
Laptop Only

Laptop + HDMI

Laptop + DP

Display 1:
1920x1080 @ 100%

Display 2:
2560x1440 @ 125%

Display 1:
2560x1600 @ 150%

Display 2:
1920x1080 @ 100%
```

还要测试：

```text
Screen 2 在左边

Screen 2 在右边

Screen 2 在上面

Screen 2 在下面
```

因为坐标可能出现：

```text
负数 X
负数 Y
```

---

# 51. 验收标准

## Calendar

启动：

```text
显示当前月份
```

点击：

```text
October 15
```

必须：

```text
selectedDate = October 15
```

下方：

```text
只显示 October 15 的记录
```

---

## Daily Note

必须支持两个新增入口：

```text
A. 右上角 ＋
B. 点击日期格
```

两者必须调用同一个 DailyNoteEditor / createDailyNote 流程。

创建：

```text
October 15
14:00
Physics Homework
```

必须：

```text
数据库出现记录
```

同时：

```text
October 15 日期格立即变化
```

Upcoming：

```text
立即刷新
```

---

## Display 2

设备拥有两个显示器：

```text
启动 App
```

要求：

```text
第一次显示就已经在 Screen 2
```

不允许：

```text
Screen 1 闪一下
```

---

## Screen 2 Disconnect

拔掉屏幕：

```text
窗口自动回到 Primary
```

不能：

```text
留在不可见坐标
```

---

## Screen 2 Reconnect

重新连接：

```text
窗口自动返回 Screen 2
```

如果设置：

```text
auto_return = false
```

则保持当前位置。

---

# 52. 最终产品定位

这个项目不是普通：

```text
Todo App
```

也不是普通：

```text
Calendar App
```

而是：

```text
Calendar-first Desktop Secretary
```

其核心体验：

```text
打开电脑
↓
第二屏幕自动出现一个半透明日历
↓
一眼看到这个月
↓
哪些日子有事情立即可见
↓
点击日期查看当天记录
↓
右边永远显示最近要发生的事情
↓
不需要打开浏览器
↓
不需要登录
↓
不需要找 Todo 软件
```

---

# 53. 最终 V1 定义

一句话：

> 一个自动驻留 Windows Display 2、采用半透明毛玻璃 UI、以日历为主界面，并整合日签、Todo 与 Upcoming 的 Local-First 桌面秘书。

---

# 54. 开发执行顺序

严格按照以下顺序：

```text
01
Fork / 选定成熟 Tauri Calendar/Todo 基底

02
删除不需要的云端 / 帐号 / 团队模块

03
跑通 Tauri + React

04
接 SQLite

05
完成 DailyNote CRUD

06
完成 Month View

07
完成日期格 Note Indicator

08
完成 Selected Date 下方记录

09
完成 Upcoming

10
完成 Day View

11
完成 Year View

12
完成 Window Transparency

13
完成 Mica / Acrylic

14
完成 Monitor Manager

15
完成 Display 2 启动定位

16
完成 Monitor Position Save

17
完成 Display Reconnect

18
完成 DPI Scaling

19
完成 Tray

20
完成 Auto Start

21
完成 Settings

22
完成 Backup / Export

23
进行 Multi Monitor QA

24
生成 Windows Release
```

不要一开始同时开发全部功能。

---

# 55. 工程开发规则

以后 Agent 修改本项目时必须遵守：

1. 有成熟实现优先修改成熟实现。
2. 不为了「代码漂亮」重写已经稳定工作的模块。
3. UI 与业务逻辑分离。
4. 数据库操作集中管理。
5. Monitor API 不允许散落在 React Component。
6. Windows 特定逻辑放 Rust backend。
7. 所有 Display 行为必须可测试。
8. 不允许用数组下标判断 Windows Display Number。
9. 不允许在每个 Calendar Cell 单独查询数据库。
10. 不允许破坏 Local-First。
11. 新功能必须说明是否影响启动速度。
12. 新功能必须说明是否影响 SQLite Schema。
13. 每个功能完成后必须有对应验收测试。
14. 先确保功能可靠，再做动画。
15. V1 不加入无关的大型功能。
16. 日签新增只能有一套底层创建逻辑，右上角 `＋` 与日期点击必须复用同一个 Editor / createDailyNote 流程。

---

# 56. 项目暂定名称

工作名：

```text
Calendar Desktop Secretary
```

后续可以再决定正式名称。

候选：

```text
DayGlass
GlassCal
DailyGlass
DeskCal
SecondScreen Calendar
Mizu Calendar
AquaCal
```

当前开发阶段建议保持：

```text
Calendar Desktop Secretary
```

避免因为命名影响工程推进。
