# Safe Keep — 照片备份桌面软件

## 一、项目概述

Safe Keep 是一款高效、安全、操作简单的照片与视频备份桌面工具，采用 Tauri 2.x + Rust + Vue 3 技术栈开发，专注于将 Android 手机、SD 卡、U 盘、相机等设备中的媒体文件快速备份到本地硬盘或外部存储。

**iOS / iPhone**：不支持。

---

## 二、技术栈

| 层级 | 技术选型 | 说明 |
|------|---------|------|
| 桌面壳层 | Tauri 2.x | 系统原生 Webview，包体 ~10MB，内存占用低 |
| 后端逻辑 | Rust | 多线程文件 I/O、哈希计算、数据库操作 |
| 前端 UI | Vue 3 + Vite + TypeScript | 轻量响应式 SPA |
| UI 库 | Naive UI / Element Plus | 组件化界面 |
| 数据库 | SQLite (rusqlite) | 备份记录与文件元数据持久化 |
| 哈希库 | xxhash-rust | 极快非加密哈希，局部校验 |
| 并发库 | rayon | Rust 数据并行 |
| 文件遍历 | walkdir | 递归目录遍历 |

---

## 三、完整功能清单

### ⚙️ 3.1 核心引擎（Rust 层）

#### 3.1.1 文件扫描模块

- 递归遍历目录树（walkdir）
- 多线程并发扫描（rayon），充分利用磁盘 I/O
- 扩展名白名单过滤，支持的格式：
  - 图片：`.jpg`, `.jpeg`, `.png`, `.heic`, `.heif`, `.webp`, `.gif`, `.bmp`, `.tiff`, `.raw`, `.arw`, `.cr2`, `.nef`
  - 视频：`.mp4`, `.mov`, `.avi`, `.mkv`, `.mts`, `.m2ts`
- 自动忽略系统隐藏/临时目录（`$RECYCLE.BIN`、`System Volume Information`、`._` 开头的 macOS 隐藏文件、`.DS_Store` 等）
- 扫描进度通过 Tauri IPC 事件实时推送到前端
- 支持中断扫描（用户取消）
- **扫描策略**：
  - 由于源端是手机/U 盘/SD 卡等可移动介质（通常为 MTP、FAT32/exFAT），不支持目录 mtime 跳过，**每次均全量扫描源端目录树**
  - 扫描只读文件元数据（路径 + 文件名 + 大小 + 修改时间），不读内容、不做哈希
  - 扫描结果 `Vec<FileEntry>` 在内存中与数据库记录对比，毫秒级分出 新增/已备份/已变更

#### 3.1.2 文件对比/增量模块

**增量对比流程：**

1. **源端扫描**：完整遍历源端目录树，得到 `Vec<FileEntry>`（只读元数据，不做哈希）
2. **查询数据库**：根据 `source_root`（源目录路径）查询该源下已有的备份记录
3. **内存哈希表比对**：将扫描结果与数据库记录做内存级 O(1) 哈希表比对

**对比策略（三级）：**

- **一级对比（默认，最快）**：`relative_path + 文件大小 + 修改时间`
  - 三种字段组合在真实照片中碰撞概率趋近于 0，日常使用完全足够
  - 数据库中存在且三者一致 → "已备份"
  - 数据库中存在但任一不同 → "已变更"
  - 数据库中不存在 → "新增"
  - 数据库中有但源端不存在（源端已删除）→ 标记"源端已消失"，不纳入本次备份
- **二级对比（可选）**：读取文件头部 4KB + 尾部 4KB，计算 xxHash 确认（仅在删除前校验阶段执行，此时数量少、可接受）
- **三级对比（用户可选）**：全文件 SHA256（适用于高安全性场景）

**三分类显示：**
  - "新增"：数据库中不存在
  - "已备份（未变更）"：数据库中存在且元数据一致
  - "已变更"：数据库中存在但元数据变化（需重新备份）

#### 3.1.3 文件复制模块

- **保持源端目录结构**：复制到目标端时，按照源端的**相对路径**创建目录结构
  ```
  源: /Volumes/NO NAME/DCIM/100APPLE/IMG_0001.jpg
                    ↓ 相对路径: DCIM/100APPLE/IMG_0001.jpg
  目标: D:/Backups/手机照片/DCIM/100APPLE/IMG_0001.jpg
  ```
- 多线程并行复制（rayon + 可控并发数）
- 带粒度进度回调：
  - 总体进度（已完成文件数 / 总文件数）
  - 当前文件进度（已复制字节 / 总字节）
  - 实时速度（MB/s）
  - 预计剩余时间
- 错误处理：
  - 磁盘空间不足 → 检测并提前警告，预留所需空间检查
  - 文件名冲突 → 自动重命名（`文件名_1.ext`）/ 跳过 / 覆盖（用户可配置）
  - 只读/权限错误 → 跳过并记录日志
  - 设备断开/源路径消失 → 暂停任务，提示用户
  - 文件被占用 → 重试策略（最多 3 次，间隔 500ms）
- 断点续传：已复制完成的文件标记成功，下次不再重新复制

#### 3.1.4 数据库模块（SQLite）

**设计原则：**
- 使用 `source_root`（源目录路径）作为备份记录的隔离维度
- 同一个源目录的目标文件通过 `relative_path` 唯一标识
- 不同源目录（如不同 U 盘、不同手机）各自独立记录
- 不使用任何标志文件机制，数据库是唯一的真相来源

**files 表定义：**

```sql
CREATE TABLE IF NOT EXISTS files (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    source_root     TEXT NOT NULL,               -- 源目录根路径（如 "E:/DCIM"）
    relative_path   TEXT NOT NULL,               -- 相对于源根目录的路径（如 "100APPLE/IMG_0001.jpg"）
    dest_path       TEXT NOT NULL,               -- 备份目标完整路径
    file_name       TEXT NOT NULL,               -- 文件名（含扩展名）
    file_size       INTEGER NOT NULL,            -- 文件大小（字节）
    modified_at     INTEGER NOT NULL,            -- 修改时间（Unix 时间戳）
    status          TEXT NOT NULL DEFAULT 'pending',
                    -- pending: 待备份
                    -- backing_up: 备份中
                    -- backed_up: 已备份
                    -- verified: 已验证通过
                    -- deleted: 源文件已删除
                    -- failed: 备份失败
    backed_up_at    TEXT,                        -- 备份完成时间（ISO8601）
    verified_at     TEXT,                        -- 最近验证时间
    error_message   TEXT,                        -- 失败原因

    -- 同一源目录下的相对路径唯一
    UNIQUE(source_root, relative_path)
);

CREATE INDEX idx_files_source_root ON files(source_root);
CREATE INDEX idx_files_status ON files(status);
CREATE INDEX idx_files_modified_at ON files(modified_at);
```

**备份任务表：**

```sql
CREATE TABLE IF NOT EXISTS backup_tasks (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    source_root     TEXT NOT NULL,
    dest_path       TEXT NOT NULL,
    total_files     INTEGER,
    total_bytes     INTEGER,
    backed_up_files INTEGER DEFAULT 0,
    backed_up_bytes INTEGER DEFAULT 0,
    failed_files    INTEGER DEFAULT 0,
    status          TEXT NOT NULL DEFAULT 'running',
                    -- running / paused / completed / cancelled / failed
    started_at      TEXT,
    completed_at    TEXT,
    avg_speed       REAL                    -- 平均速度（MB/s）
);
```

**历史记录表（用于清理/删除决策）：**

```sql
CREATE TABLE IF NOT EXISTS delete_history (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    file_id         INTEGER REFERENCES files(id),
    source_root     TEXT NOT NULL,
    source_path     TEXT NOT NULL,
    deleted_at      TEXT NOT NULL,
    delete_method   TEXT NOT NULL,          -- recycle / permanent
    verified        BOOLEAN NOT NULL DEFAULT 1
);
```

**设置表：**

```sql
CREATE TABLE IF NOT EXISTS settings (
    key             TEXT PRIMARY KEY,
    value           TEXT NOT NULL
);
```

#### 3.1.5 安全删除模块

- **删除前双重校验：**
  1. 检查目标路径文件存在
  2. 校验目标文件与源文件的大小和修改时间一致（必要时校验哈希）
- 支持两种删除策略：
  - "移至回收站"（使用 `trash` crate，仅本地盘符有效）
  - "永久删除"（MTP 手机默认行为，不可恢复）
- **MTP 设备特殊处理**：
  - 删除前强制校验哈希（头尾 xxHash），确保备份完整
  - 删除不可逆，提示语强调"如需还原，可从备份目录恢复"
  - 删除操作单独记录日志，方便追溯
- **Dry Run（安全模拟）模式**：仅列出将删除的文件和释放的空间，不执行实际删除
- 删除成功后更新 `files.status = 'deleted'`，写入 `delete_history`
- 删除失败时记录错误，不影响其他文件的删除

#### 3.1.6 设备类型识别模块

**作用**：根据用户选择的源路径，自动判断设备连接方式，进而调整扫描策略、删除行为、用户提示。

**判断逻辑**：

| 判断方法 | 判定结果 |
|---------|---------|
| 路径前缀为 `\\?\` 或使用 `GetVolumeInformation` 返回 `MTP` 驱动类型（Windows） | MTP 设备 |
| 路径挂载点为 `/run/user/*/gvfs/` 或 `/var/run/user/*/gvfs/`（Linux） | MTP 设备 |
| 路径所在卷的 `DriveType` 为 `DRIVE_REMOVABLE`（Windows `GetDriveType`） | USB 可移动磁盘 |
| 路径所在卷的 `DriveType` 为 `DRIVE_FIXED` | 本地硬盘 |
| macOS 下通过 `statfs` 获取文件系统类型，`mtmfs` 或 `fusefs` 前缀 | MTP 设备 |
| macOS 下路径为 `/Volumes/*` 且非系统盘 | 可移动磁盘 |

**Rust 实现**：
```rust
enum DeviceType {
    MTP,             // Android 手机 MTP 连接
    RemovableDisk,   // U 盘、SD 卡读卡器
    LocalDisk,       // 本地硬盘或 SSD
    Unknown,         // 无法识别
}

fn detect_device_type(path: &str) -> DeviceType {
    // Windows: GetDriveTypeW / GetVolumeInformationW
    // macOS: statfs -> f_fstypename
    // Linux: statfs -> f_type
}
```

**对业务的影响**：

| 设备类型 | 扫描方式 | 删除方式 | 可写文件 | 还原速度 |
|---------|---------|---------|---------|---------|
| MTP | 全量扫描（MTP 协议遍历较慢） | 永久删除（不可恢复） | ⚠️ 写入慢且不可靠 | 较慢（MTP 写入比读取慢） |
| RemovableDisk | 全量扫描（快速） | 可入回收站 | ✅ 完全自由读写 | 正常 |
| LocalDisk | 全量扫描（最快） | 可入回收站 | ✅ 完全自由读写 | 正常 |

**前端展示**：
- 路径选择后，在路径旁边显示设备类型图标（📱 MTP / 💾 USB / 💻 Local）
- 若为 MTP 设备，删除页面的提示语改为"此设备为 MTP 连接，删除后不可恢复，但可从备份目录还原"

#### 3.1.7 按条件筛选清理模块

- 按时间段筛选备份成功的源文件：
  - 最近 N 天（7、30、90、180、365）
  - 指定日期区间
  - 指定日期之前
- 按文件类型筛选
- 按文件大小筛选（最小值/最大值）
- 筛选结果分页展示，提供总释放空间预估

#### 3.1.8 自动升级模块

- 依赖 `tauri-plugin-updater` 实现，无需自有服务器
- **更新源**：GitHub Releases（解析 Release Tag 与版本号对比）
- **更新策略**：
  - 启动时后台静默检查新版本
  - 检测到新版本 → 前端通知栏提示"发现新版本 vX.X.X"
  - 用户点击"立即更新" → 下载安装包 → 自动安装重启
  - 支持强制更新（标注为 `critical` 的 Release 不可跳过）
- **打包产物**：通过 `tauri build` 生成各平台安装包并上传至 GitHub Releases
  - Windows: `.msi` 或 `.exe`
  - macOS: `.dmg`（需代码签名）
  - Linux: `.deb` + AppImage
- **版本号规范**：语义化版本 `MAJOR.MINOR.PATCH`

#### 3.1.9 文件还原模块

**设计背景**：MTP 手机删除文件是直接删除、不可恢复。因此安全删除的"后门"是从备份还原。

- **还原流程**：
  1. 用户在清理页勾选需要还原的文件（即可清理的文件列表）
  2. 选择还原目标路径（默认填充源路径，允许修改）
  3. 确认还原 → 从目标备份目录复制回源端
  4. 还原完成后更新 `files.status = 'restored'`
- **覆盖策略**：
  - 源端已不存在该文件 → 直接复制
  - 源端存在同名文件 → 提示用户选择：覆盖 / 跳过 / 保留两者
- **批量还原**：支持一次性还原多个文件
- **进度反馈**：同复制模块，实时显示速度 + 剩余时间

---

### 🖥️ 3.2 用户界面（Vue 3 层）

#### 3.2.1 主窗口布局

```
┌─────────────────────────────────────────────┐
│  Safe Keep  ─── □ ✕                         │
├─────────────────────────────────────────────┤
│  [源路径 ▾]      →    [目标路径 ▾]  [开始备份]  │
│  格式: [✓图片] [✓视频]  最小: [0KB]  最大: [∞] │
├───────────────────┬─────────────────────────┤
│                   │                         │
│   文件预览列表     │    详情/日志面板         │
│   （分页表格）     │                         │
│                   │                         │
├───────────────────┴─────────────────────────┤
│  ████████░░░░░░░░░░ 50%  128MB/s  剩余 2分钟  │
└─────────────────────────────────────────────┘
```

#### 3.2.2 页面与路由

| 路由 | 页面 | 功能 |
|------|------|------|
| `/` | 首页 | 源/目标选择、过滤器、开始备份入口 |
| `/preview` | 扫描预览 | 显示扫描结果，勾选/取消文件，预览总览统计 |
| `/backup` | 备份任务 | 实时进度、速度、日志、暂停/取消 |
| `/cleanup` | 清理管理 | Dry Run 模拟、条件筛选、执行清理 |
| `/history` | 历史记录 | 备份任务历史列表、详情、报告导出 |
| `/restore` | 还原管理 | 选择已删除的备份文件，还原回源设备 |
| `/settings` | 设置 | 所有配置项 |

#### 3.2.3 首页（路径与过滤器）

- 源路径选择组件：
  - 输入框 + 文件夹选择对话框按钮
  - 最近选择过的路径下拉列表
  - 显示路径可用空间、总文件数预估
- 目标路径选择组件：
  - 同上
  - 显示目标盘剩余空间、已备份文件数
- 过滤器：
  - 图片格式勾选框组（全选/取消）
  - 视频格式勾选框组（全选/取消）
  - 最小文件大小输入（默认 0）
  - 最大文件大小输入（默认无限制）
  - 排除已备份文件（增量模式）勾选框
- 开始备份按钮（扫描并进入预览/直接开始备份）

#### 3.2.4 扫描预览页

- 统计卡片：
  - 扫描到文件总数
  - 总大小（格式化显示）
  - 新增 / 已备份 / 已变更 三类计数
- 文件列表表格：
  - 列：文件名、大小、修改日期、状态（新增/已备份/已变更）
  - 支持排序（点击列头）
  - 支持勾选/取消勾选（默认全选新增+已变更）
  - 分页
  - 按状态筛选的 Tab 或下拉
- 底部操作栏：
  - 已选文件数 / 总大小
  - 「开始备份」按钮
  - 「返回修改」按钮

#### 3.2.5 备份任务页

- 大进度条（总进度）
- 当前文件名称与进度（小进度条）
- 实时指标：
  - 速度（MB/s）
  - 已备份文件数 / 总文件数
  - 已传输数据量（GB/MB）
  - 预计剩余时间
- 操作按钮：暂停 / 继续 / 取消
- 日志面板：滚动显示每步操作（"正在复制 xxx.jpg... ✓"）
- 备份完成后的汇总弹窗：
  - 成功 X 个 / 失败 Y 个
  - 总耗时 X 分 Y 秒
  - 平均速度 X MB/s
  - 「查看详情」→ 跳转到历史记录

#### 3.2.6 安全清理页

- 步骤一：选择清理条件
  - 时间段选择（备份超过 N 天 / 指定日期前）
  - 筛选预览按钮（Dry Run）
- 步骤二（Dry Run 结果）：
  - 将删除的文件列表（分页表格）
  - 总文件数 / 总释放空间
  - 「执行清理」按钮（需要二次确认弹窗）
  - 「返回修改」按钮
- 步骤三（清理执行）：
  - 清理进度
  - 完成报告：成功删除 X 个 / 失败 Y 个 / 释放 X MB

#### 3.2.7 历史记录页

- 备份任务列表（时间倒序）
  - 每次备份任务的摘要卡片：时间、源→目标、文件数、大小、速度、状态
- 点击展开/跳转到详情
- 任务详情：
  - 成功文件列表
  - 失败文件列表（含错误原因）
  - 可重新备份失败文件（一键重试）
  - 导出报告（CSV / JSON）

#### 3.2.8 还原管理页

- 选择还原源（从哪个备份还原）：
  - 下拉选择历史备份方案（由 `source_root` 标识）
- 可还原文件列表：
  - 显示 `status = 'deleted'` 且源端不存在的文件
  - 列表字段：文件名、原路径、备份时间、大小
- 还原目标路径：
  - 默认填充原 `source_root`
  - 允许用户手动修改（如手机重新连接后路径变了）
- 还原执行：
  - 从 `dest_path` 复制回用户指定的还原目标路径
  - 保持原始目录结构
  - 实时进度（同备份模块）
  - 完成后更新 `files.status = 'restored'`
- 覆盖冲突处理：
  - 目标已存在同名文件 → 选择覆盖 / 跳过 / 保留两者

#### 3.2.9 设置页

| 设置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| 默认目标路径 | 路径字符串 | 空 | 下次启动自动填入 |
| 并发复制线程数 | 数字（1~16） | 4 | 复制时同时处理多少文件 |
| 文件对比策略 | 下拉选项 | 大小+时间 | 可选：仅大小时间 / 快速哈希 / SHA256 |
| 文件名冲突处理 | 下拉选项 | 自动重命名 | 可选：跳过 / 覆盖 / 重命名 |
| 删除策略 | 下拉选项 | 回收站 | 可选：回收站 / 永久删除 |
| Dry Run 默认启用 | 开关 | 关 | 清理前总是先进入模拟模式 |
| 自动清理提醒 | 开关 | 开 | 启动时检查可清理文件并提示 |
| 语言 | 下拉选项 | 中文 | 可选：中文 / English |
| 主题 | 下拉选项 | 跟随系统 | 可选：浅色 / 深色 / 跟随系统 |
| 自动更新检查 | 开关 | 开 | 启动时检查 GitHub Releases 新版本 |
| 更新通道 | 下拉选项 | stable | 可选：stable / beta |

---

## 四、目录结构

```
safe-keep/
├── src-tauri/                  # Rust 后端
│   ├── src/
│   │   ├── main.rs             # Tauri 入口
│   │   ├── lib.rs              # 模块导出
│   │   ├── device/             # 设备类型检测（MTP / U盘 / 本地磁盘）
│   │   │   ├── mod.rs
│   │   │   ├── detect.rs       # 判断设备类型
│   │   │   └── platform.rs     # 各平台实现（Win / macOS / Linux）
│   │   ├── commands/           # Tauri IPC 命令
│   │   │   ├── mod.rs
│   │   │   ├── scan.rs         # 扫描相关命令
│   │   │   ├── backup.rs       # 备份相关命令
│   │   │   ├── cleanup.rs      # 清理相关命令
│   │   │   ├── restore.rs      # 还原相关命令
│   │   │   └── settings.rs     # 设置相关命令
│   │   ├── scanner/            # 文件扫描引擎
│   │   │   ├── mod.rs
│   │   │   ├── walk.rs         # 目录遍历
│   │   │   ├── filter.rs       # 扩展名/大小过滤
│   │   │   └── progress.rs     # 进度回调
│   │   ├── copier/             # 文件复制引擎
│   │   │   ├── mod.rs
│   │   │   ├── parallel.rs     # 多线程复制
│   │   │   ├── conflict.rs     # 冲突处理
│   │   │   └── progress.rs     # 进度回调
│   │   ├── hasher/             # 哈希计算（仅删除前校验阶段使用）
│   │   │   ├── mod.rs
│   │   │   ├── fast_hash.rs    # xxHash 头尾哈希
│   │   │   └── sha256.rs       # 全文件 SHA256
│   │   ├── database/           # 数据库层
│   │   │   ├── mod.rs
│   │   │   ├── models.rs       # 数据模型
│   │   │   ├── migrations.rs   # 建表/迁移
│   │   │   └── queries.rs      # 查询方法
│   │   ├── deleter/            # 安全删除模块
│   │   │   ├── mod.rs
│   │   │   ├── verify.rs       # 校验
│   │   │   └── delete.rs       # 执行删除
│   │   ├── restorer/           # 文件还原模块
│   │   │   ├── mod.rs
│   │   │   ├── restore.rs      # 还原逻辑（从备份复制回源端）
│   │   │   └── conflict.rs     # 还原时的覆盖冲突处理
│   │   └── utils/              # 工具函数
│   │       ├── mod.rs
│   │       ├── path.rs         # 路径处理
│   │       └── format.rs       # 格式化（大小/时间）
│   ├── Cargo.toml            # 依赖：tauri-plugin-updater
│   └── tauri.conf.json       # 配置：updater 端点指向 GitHub Releases
├── src/                        # Vue 3 前端
│   ├── main.ts                 # 入口
│   ├── App.vue                 # 根组件
│   ├── router/                 # 路由
│   │   └── index.ts
│   ├── views/                  # 页面
│   │   ├── Home.vue
│   │   ├── Preview.vue
│   │   ├── Backup.vue
│   │   ├── Cleanup.vue
│   │   ├── History.vue
│   │   ├── Restore.vue
│   │   └── Settings.vue
│   ├── components/             # 通用组件
│   │   ├── PathSelector.vue
│   │   ├── FileFilter.vue
│   │   ├── FileTable.vue
│   │   ├── ProgressBar.vue
│   │   ├── SpeedIndicator.vue
│   │   └── StatsCard.vue
│   ├── composables/            # 组合式函数
│   │   ├── useScan.ts
│   │   ├── useBackup.ts
│   │   ├── useCleanup.ts
│   │   └── useSettings.ts
│   ├── types/                  # TypeScript 类型
│   │   ├── file.ts
│   │   ├── backup.ts
│   │   └── settings.ts
│   ├── assets/                 # 静态资源
│   │   └── styles/
│   └── locales/                # 国际化
│       ├── zh-CN.json
│       └── en.json
├── package.json
├── vite.config.ts
└── tsconfig.json
```

---

## 五、开发计划（6 周迭代）

### 第 1 周：项目搭建 + 扫描引擎

| 任务 | 详细内容 |
|------|---------|
| 项目初始化 | cargo init + npm create vue@latest，配置 Tauri 2.x 集成 |
| 扫描引擎 | 实现 `FileScanner`（walkdir + rayon），返回 `Vec<FileEntry>` |
| 过滤逻辑 | 扩展名白名单 + 文件大小过滤 |
| 进度回调 | 扫描进度通过 Tauri IPC 事件（`scan:progress` / `scan:complete`）发送给前端 |
| 前端骨架 | 创建路由、首页布局、PathSelector 组件 |
| 前端展示 | 扫描结果显示（文件列表 + 总数统计） |
| 测试 | FAT32 / MTP 下 1000+ 文件扫描性能测试 |

**可交付物：** 选择目录 → 多线程扫描 → 前端显示图片/视频文件列表

### 第 2 周：数据库 + 增量对比

| 任务 | 详细内容 |
|------|---------|
| SQLite 集成 | rusqlite 建库，创建 files / backup_tasks / delete_history / settings 表 |
| 数据库 CRUD | `insert_file`、`is_backed_up`、`get_backed_up_files`、`batch_update_status` |
| 增量对比逻辑 | 全量扫描 → 内存级哈希表比对数据库记录，区分 新增/已备份/已变更 |
| 哈希模块 | 实现 xxHash 头尾哈希函数（扫描阶段不用，仅删除前校验使用） |
| 前端适配 | 扫描预览页显示三类文件计数，区分颜色/标签 |
| 测试 | 增量场景（新增文件、修改文件、重复扫描） |

**可交付物：** 扫描后自动区分"新增/已备份/已变更"

### 第 3 周：文件复制引擎

| 任务 | 详细内容 |
|------|---------|
| 复制引擎 | 实现 `FileCopier`（rayon 并行 + 可控并发数） |
| 进度上报 | 总进度 + 当前文件进度 + 速度 + 剩余时间 → IPC 事件推送给前端 |
| 错误处理 | 磁盘满、冲突、权限、设备断开、重试逻辑 |
| 冲突策略 | 自动重命名、跳过、覆盖（配置可切换） |
| 暂停/继续/取消 | 通过原子标志控制任务生命周期 |
| 前端绑定 | 备份任务页：进度条、速度、日志、暂停/取消按钮 |
| 数据库更新 | 复制完成后批量更新 `files.status = 'backed_up'` |

**可交付物：** 完整的备份执行流程，前端实时进度显示

### 第 4 周：安全删除 + Dry Run + 条件筛选

| 任务 | 详细内容 |
|------|---------|
| 校验模块 | `verify_backup_integrity(source, dest) -> Result` 双重校验 |
| 删除执行 | 回收站（trash crate） / 永久删除 |
| Dry Run | 只模拟列出可清理文件，不实际删除 |
| 条件筛选 | 时间段 <-> 文件类型 <-> 文件大小组合筛选 |
| 前端适配 | 安全清理页：条件选择 → Dry Run 预览 → 确认清理 → 进度报告 |
| 删除记录 | 写入 delete_history 表，更新 files.status |

**可交付物：** 按条件安全清理已备份源文件

### 第 5 周：历史记录 + 设置页 + 完善 UI

| 任务 | 详细内容 |
|------|---------|
| 历史记录 | 备份任务列表 + 详情查看 + 失败重试 |
| 报告导出 | CSV / JSON 导出备份结果 |
| 设置页 | 全部配置项 + 持久化到 SQLite settings 表 |
| 主题切换 | 浅色 / 深色 / 跟随系统 |
| 国际化 | 中文 / English 切换 |
| 异常测试 | 拔 USB、空间不足、文件占用、超长路径 |
| 边界优化 | 扫描/复制中的各种异常处理完善 |

**可交付物：** 功能完整的 Beta 版本

### 第 6 周：性能调优 + 打包发布

| 任务 | 详细内容 |
|------|---------|
| 大规模测试 | 10,000+ 文件场景，验证扫描速度、复制速度、内存占用 |
| 性能调优 | 调整线程数、缓冲区大小等参数 |
| 自动升级配置 | 集成 `tauri-plugin-updater`，配置 GitHub Releases 作为更新源 |
| 打包配置 | Windows `.msi` / macOS `.dmg` / Linux `.deb` + AppImage |
| 文档 | README 编写、用户使用说明 |
| 发布 | 上传安装包到 GitHub Releases + 编写 changelog |

**可交付物：** 正式发布的安装包

---

## 六、技术选型讨论要点

如果需要讨论技术选型，以下是需要决策的关键点：

### 6.1 前端 UI 框架
- **Naive UI**：Vue 3 原生，Tree-shaking，文件选择对话框等组件丰富，中式审美
- **Element Plus**：生态更成熟，但包体略大
- **Headless UI + Tailwind**：完全自定义，但开发量更大

### 6.2 Rust Crates 依赖决策
- **walkdir** vs **ignore**：walkdir 更轻量，ignore 还需处理 gitignore，选 walkdir
- **rusqlite** vs **sqlx**：rusqlite 是同步的，更简单直接；sqlx 异步但在这个场景没必要
- **xxhash-rust** vs **md-5 / sha2**：xxhash 快 10x+，用于头尾校验足够
- **rayon** vs **tokio**：文件 I/O 是 CPU 密集型 + 阻塞，rayon 并行更合适
- **trash** crate：跨平台回收站操作

### 6.3 进程通信
- **Tauri IPC（invoke + events）**：命令调用 + 事件推送
- 扫描/备份进度使用 `app_handle.emit("event_name", payload)` 推送给前端

### 6.4 状态管理
- **Vue Pinia**：替代 Vuex，轻量 + TypeScript 友好，管理全局备份状态

---

## 七、设备兼容性详表

| 设备类型 | 连接方式 | 显示为盘符？ | 可读文件？ | 可写文件？ | 可删除？ | 删除后可恢复？ | 本 App 支持？ |
|---------|---------|------------|-----------|----------|---------|--------------|-------------|
| **U 盘 / 移动硬盘** | USB | ✅ | ✅ | ✅ 自由读写 | ✅ 回收站 | ✅ | ✅ **完全支持** |
| **SD 卡（读卡器）** | USB 读卡器 | ✅ | ✅ | ✅ 自由读写 | ✅ 回收站 | ✅ | ✅ **完全支持** |
| **相机（USB 直连）** | PTP/MTP | ⚠️ 部分相机 | ✅ | ⚠️ 通常只读 | ❌ | ❌ | ⚠️ 仅读取备份 |
| **Android 手机（MTP）** | USB 数据线 | ❌ | ✅ | ⚠️ 可写但慢 | ✅ 永久删除 | ❌ | ✅ **全功能支持**（含还原） |
| **iPhone / iPad** | — | — | — | — | — | — | **不支持** |

**MTP 与本地盘符的区分方式**：

| 判断方法 | Windows | macOS | Linux |
|---------|---------|-------|-------|
| API | `GetDriveTypeW` 返回 `DRIVE_REMOTE` | `statfs` → `f_fstypename` 为 `mtmfs` 或 `fusefs` | `statfs` → `f_type` 或路径含 `gvfs` |
| 路径特征 | `Computer\` 下不显示盘符 | `/Volumes/` 下出现但不可写测试可判定 | `/run/user/*/gvfs/` 路径 |
| 写测试 | 尝试创建临时文件失败 | 尝试写入失败 | 尝试写入失败 |


