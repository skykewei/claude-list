# Claude List 工具设计文档

## 概述

一个 Rust 命令行工具，用于列出 Claude Code 已安装的 skills 和 MCP servers。支持从本地配置文件和 Claude API 获取信息，支持表格和 JSON 两种输出格式。

## 架构设计

采用分层架构：

```
┌─────────────────────────────────────┐
│           CLI 层 (main.rs)          │
│    - 参数解析 (--json, --local,     │
│      --api, --verbose 等)           │
├─────────────────────────────────────┤
│          服务层 (service)           │
│    - SkillService: 获取 skill 列表  │
│    - McpService: 获取 MCP 列表      │
│    - 统一协调本地和 API 数据源        │
├─────────────────────────────────────┤
│          数据源层 (source)          │
│    - LocalSource: 读取 ~/.claude/   │
│    - ApiSource: 调用 Claude API     │
├─────────────────────────────────────┤
│          输出层 (output)            │
│    - TableFormatter: 表格输出       │
│    - JsonFormatter: JSON 输出       │
└─────────────────────────────────────┘
```

### 关键设计决策

1. **模块化数据源**：`LocalSource` 和 `ApiSource` 实现相同的 trait，可以独立测试和替换
2. **统一数据模型**：无论来源是本地还是 API，都转换为统一的 `Skill` 和 `McpServer` 结构
3. **输出格式化器分离**：数据获取和展示完全解耦

## 数据模型

```rust
pub struct Skill {
    pub name: String,
    pub version: Option<String>,  // 本地 skill 可能无版本
    pub source: SourceType,        // Local | Api
    pub path: Option<PathBuf>,     // 本地路径
    pub description: Option<String>,
}

pub struct McpServer {
    pub name: String,
    pub status: ConnectionStatus,  // Connected | Disconnected | Unknown
    pub config: Option<McpConfig>, // 本地配置或 API 返回的配置
    pub source: SourceType,
}

pub enum SourceType {
    Local,
    Api,
    Both(LocalInfo, ApiInfo),  // 当本地和 API 都有时合并
}

pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Unknown,
    Error(String),
}
```

## 命令行接口

```bash
claude-list [OPTIONS] [COMMAND]

COMMANDS:
  list        列出所有 skill 和 MCP（默认）
  skills      仅列出 skills
  mcps        仅列出 MCP servers
  help        显示帮助信息

OPTIONS:
  -j, --json           输出 JSON 格式
  -l, --local          仅使用本地数据源
  -a, --api            仅使用 API 数据源（需要 CLAUDE_API_KEY）
  -v, --verbose        显示详细信息
  -h, --help           显示帮助信息
```

### 配置发现规则

- **Skills 本地路径**: `~/.claude/skills/` 下的子目录
- **MCP 本地配置**:
  - `~/.claude/settings.json` 中的 `mcpServers` 字段
  - 或 `~/.claude/mcp.json`
- **API 端点**: 需要 `CLAUDE_API_KEY` 环境变量

## 错误处理策略

采用 `thiserror` 实现分层错误类型：

```rust
#[derive(Error, Debug)]
pub enum CliError {
    #[error("Failed to read local configuration: {0}")]
    LocalConfigError(#[from] LocalSourceError),

    #[error("API request failed: {0}")]
    ApiError(#[from] ApiSourceError),

    #[error("Missing API key. Set CLAUDE_API_KEY environment variable")]
    MissingApiKey,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

### 处理原则

- 数据源失败时：如果是 `--local` 或 `--api` 模式，返回错误退出；如果是默认模式，降级到可用数据源并显示警告
- API 未配置时：静默跳过并提示 `--local` 选项
- 使用 `anyhow` 在 main 函数中统一处理错误展示

## 测试策略

1. **单元测试**：每个数据源、格式化器独立测试，使用临时目录模拟配置
2. **集成测试**：测试完整 CLI 流程，使用 `assert_cmd` 和 `predicates`
3. **Mock 测试**：API 调用使用 `mockall` 或 `wiremock`

## 项目文件结构

```
src/
├── main.rs          # CLI 入口，参数解析
├── lib.rs           # 公共导出
├── model.rs         # 核心数据结构
├── error.rs         # 错误定义
├── service/
│   ├── mod.rs       # SkillService, McpService
│   └── merger.rs    # 合并本地和 API 数据
├── source/
│   ├── mod.rs       # Source trait
│   ├── local.rs     # LocalSource 实现
│   └── api.rs       # ApiSource 实现
└── output/
    ├── mod.rs       # Formatter trait
    ├── table.rs     # TableFormatter
    └── json.rs      # JsonFormatter
```
