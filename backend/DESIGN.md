# CampusVoice 后端设计文档（Rust）

> 版本：v0.2｜状态：M1 已实现（2026-08）
> 关联需求：[`REQUIREMENTS.md`](../REQUIREMENTS.md)｜前端：微信小程序 `miniprogram/`、管理端 `admin-web/`

## 1. 概述

### 1.1 背景

当前仓库是「程园小二」的双端高保真 Demo：小程序端与管理端全部使用本地 mock 数据，没有真实服务端、数据库、登录与持久化。本后端为 V1 提供真实可演进的 API 服务，支撑完整权益服务闭环：

```text
提交诉求 -> 分类受理 -> 部门流转 -> 跟进处理 -> 结果反馈 -> 脱敏公开
```

### 1.2 目标（本轮范围）

- 纯后端 API 服务 + 数据库迁移 + 自动化测试 + 工程文档。
- 小程序端、管理端暂不改造，接口按本文档约定对接。
- 覆盖 V1 验收标准：提交/附件、状态进度查询、受理流转、部门回复、审核脱敏公开、操作日志、通知、满意度评价。

### 1.3 非目标

- 前端两端接入真实 API（后续单独迭代）。
- 与学校 OA / 工单系统对接、AI 处理、多级审批（REQUIREMENTS §9）。
- 公众号 H5 入口。

## 2. 技术选型

| 关注点 | 选型 | 说明 |
| --- | --- | --- |
| 语言/运行时 | Rust 2021 + Tokio | 单二进制、低资源占用、类型安全 |
| Web 框架 | axum 0.8 | 生态成熟、提取器/中间件清晰 |
| 数据库 | PostgreSQL | 生产级，符合系统目标形态 |
| 数据访问 | sqlx 0.8（异步、编译期元数据可选） | 迁移内建，避免 ORM 魔法 |
| 迁移 | sqlx migrate | 版本化 SQL |
| 认证 | JWT（jsonwebtoken）+ argon2 口令散列 | 无状态会话，学生/员工两套签发 |
| 微信登录 | reqwest 调 code2session | 无 appid 时开发降级 |
| 日志/观测 | tracing + tracing-subscriber + tower-http trace | 结构化日志 + 请求追踪 |
| 上传 | axum multipart，本地磁盘 `uploads/` | 图片、≤5MB、每诉求≤3 张 |
| 配置 | 环境变量 + `.env`（dotenvy） | 不入库 |
| 部署 | Dockerfile + docker-compose（Postgres） | 也可本机裸跑 |
| 测试 | 集成测试（真实 Postgres） | 跑通核心闭环 |

> 说明：为保证无数据库时也能独立编译，查询使用 `sqlx::query_as` + `FromRow`（运行时绑定），不启用编译期 `query!` 宏；后续可切 `sqlx::query!` + offline 数据提升编译期校验。

## 3. 架构

### 3.1 分层

```text
HTTP 层(handlers) -> 领域/服务层(domain) -> 数据访问(sqlx/models)
        \-------------- 共享：AuthUser / AppError / AppState ----------------/
```

- **handlers**：参数解析、权限检查、调用领域函数、组装 JSON 响应；不含 SQL。
- **domain**：状态机、审计、通知、编号、脱敏等业务规则；事务边界在此。
- **models / sqlx**：`FromRow` 结构体与 SQL；不写业务逻辑。

### 3.2 请求生命周期

```text
客户端 -> axum Router -> [CORS/trace] -> guard(AuthUser 提取) -> handler
        -> domain::fn(&pool, ...) 事务 -> SQL -> 响应
```

### 3.3 与前端对接

- 统一前缀 `/api/v1`，JSON 响应。
- 小程序端：`wx.login()` -> code -> `POST /api/v1/auth/wechat` -> 拿 JWT -> 请求头 `Authorization: Bearer <token>`。
- 管理端：账号密码 -> `POST /api/v1/auth/login` -> JWT。
- 状态码语义：`200` 成功、`400` 参数、`401` 未认证、`403` 越权、`404` 不存在、`409` 冲突、`413` 附件过大、`500` 服务端。

## 4. 数据模型

### 4.1 ER 概览

```text
users ─┬─< issues.student_id
       ├─< issues.assignee_id
       └─< departments(id)                    departments
       └─< issue_progress.operator_id
       └─< attachments.uploader_id
       └─< notifications.user_id
       └─< audit_logs.operator_id
       └─< replies.author_id

issues ─< issue_progress
       ─< replies
       ─< attachments
       ─< notifications(issue_id)
       ──1:1> public_cases
       └─> categories(name 校验) / departments(流转)
```

### 4.2 表清单

| 表 | 用途 | 关键字段 |
| --- | --- | --- |
| `users` | 学生/员工/管理员 | `openid`(学生,唯一,可空)、`username`(员工,唯一)、`password_hash`、`role`(`student/worker/department_staff/admin`)、`department_id` |
| `departments` | 职能部门 | `name`、`code` |
| `categories` | 诉求分类 | `name`、`sort`（种子：宿舍/食堂/教学/交通/校园环境/网络/医疗健康/学生服务/其他） |
| `issues` | 诉求主表 | `issue_no`、`kind`(类型)、`category`、`campus`、`location`、`status`、`priority`、`is_public`(最终公开)、`public_will`(学生意愿)、`department_id`、`assignee_id`、`deadline`、`latest_progress`、`rating`/`rating_comment`/`rated_at` |
| `issue_progress` | 完整时间线 | `status`、`operator_id`、`operator_name`、`note` |
| `replies` | 部门正式回复 | `issue_id`、`department_id`、`author_id`、`content` |
| `attachments` | 附件（仅图片） | `issue_id`、`original_name`、`stored_name`、`size`、`mime`、`is_public` |
| `notifications` | 站内通知 | `user_id`、`title`、`content`、`kind`、`read`、`issue_id` |
| `audit_logs` | 管理操作日志 | `operator_id`、`action`、`target_type`、`target_id`、`before`、`after`、`detail`、`ip` |
| `public_cases` | 脱敏公开案例 | `issue_id`(唯一)、脱敏后的 `title/location/summary/reply`、`published_at`（NULL=草稿） |

### 4.3 关键约束

- `issues.status` ∈ `待受理/待补充/已受理/部门处理中/待审核/已办结/已关闭`（CHECK 约束）。
- `issues.rating` ∈ `满意/一般/不满意`，仅办结后可评价一次。
- `public_cases.issue_id` 唯一：每条诉求最多一份脱敏稿。
- 附件：`mime` 必须 `image/*`、`size ≤ 5MB`、每诉求 `≤ 3` 张（服务端强制）。

## 5. 认证与授权

### 5.1 角色与数据隔离（对应 REQUIREMENTS §3、§7）

| 角色 | 能做什么 | 数据范围 |
| --- | --- | --- |
| `student` | 提交/补充/查看本人诉求/评价；上传附件 | 仅本人 |
| `worker` | 受理、分类、指派、请求补充、关闭、审核公开、日志、看板 | 全部 |
| `department_staff` | 查看本部门诉求、写进展、正式回复、申请办结 | 仅本部门 |
| `admin` | 管理账号/部门/分类、审计（含 worker 全部能力） | 全部 |

### 5.2 登录

- **学生**：`POST /api/v1/auth/wechat {code}`。配置了 `WECHAT_APPID/SECRET` 时调用 `jscode2session` 换 `openid`；未配置（开发）时按 `dev_{code}` 生成伪 openid 降级，便于本地调试。签发 JWT。
- **员工**：`POST /api/v1/auth/login {username,password}`，argon2 校验。
- **种子账号**：启动时若库中无对应 `username`，则按 `ADMIN_USERNAME/ADMIN_PASSWORD`（默认 `admin`，需修改）创建管理员，并确保部门/分类种子存在。

### 5.3 会话

- JWT HS256，`sub`=user_id、`role`、`name`、`exp`；有效期由 `JWT_EXPIRES_HOURS` 控制（默认 168h）。
- 请求经 `AuthUser` 提取器校验签名与有效期；`require_role(&claims, ["worker","admin"])` 做角色门禁。
- 附件下载 `/attachments/:id` 单独做**数据隔离**：学生仅本人、员工仅本部门、worker/admin 全部、`is_public=true` 对登录用户开放。

## 6. 状态机（服务端强制）

任何 `issues.status` 变更都先经 `assert_transition`，非法流转返回 `409`；每次变更写 `issue_progress` 与 `audit_logs`：

```text
待受理 ──┬─> 已受理     已受理 ──┬─> 部门处理中
         ├─> 待补充               ├─> 待补充
         └─> 已关闭               └─> 已关闭

待补充 ──┬─> 已受理（学生补充后）
         └─> 已关闭

部门处理中 ──┬─> 待审核（部门提交正式回复）
            ├─> 待补充
            └─> 已关闭

待审核 ──┬─> 已办结（权益组审核通过，可同时公开）
         ├─> 部门处理中（打回）
         └─> 已关闭

已办结 / 已关闭 = 终态
```

## 7. API 一览

> 完整字段级契约见 §8 逐端点描述与集成测试；响应统一为 JSON。

### 7.1 公开（无需登录）

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| GET | `/api/v1/public/cases` | 已发布脱敏案例，支持 `category/keyword/status` 筛选 |
| GET | `/api/v1/public/cases/:id` | 案例详情 |

### 7.2 认证

| 方法 | 路径 | 角色 |
| --- | --- | --- |
| POST | `/api/v1/auth/wechat` | 公开 |
| POST | `/api/v1/auth/login` | 公开 |
| GET | `/api/v1/auth/me` | 任意登录 |

### 7.3 学生

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| POST | `/api/v1/issues` | 提交诉求（生成编号、状态=待受理） |
| GET | `/api/v1/my/issues` | 我的诉求列表 |
| GET | `/api/v1/my/issues/:id` | 诉求详情（含时间线/回复/附件） |
| POST | `/api/v1/my/issues/:id/supplement` | 补充信息（正文+图片，待补充->已受理） |
| POST | `/api/v1/my/issues/:id/rating` | 满意度评价（仅办结、仅一次） |

### 7.4 权益小组 / 管理员

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| GET | `/api/v1/admin/issues` | 诉求列表（status/category/department/priority/keyword 筛选） |
| GET | `/api/v1/admin/issues/:id` | 详情 |
| POST | `/api/v1/admin/issues/:id/accept` | 受理：设分类/优先级/公开属性 |
| POST | `/api/v1/admin/issues/:id/assign` | 指派部门+负责人，->部门处理中 |
| POST | `/api/v1/admin/issues/:id/request-supplement` | 请求学生补充 |
| POST | `/api/v1/admin/issues/:id/close` | 关闭（含合并场景） |
| POST | `/api/v1/admin/issues/:id/merge` | 合并到指定诉求 |
| POST | `/api/v1/admin/issues/:id/review` | 脱敏稿 草稿/通过/打回 |
| GET | `/api/v1/admin/audit-logs` | 操作日志 |
| GET | `/api/v1/admin/dashboard` | 看板统计 |

### 7.5 部门老师

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| GET | `/api/v1/department/issues` | 本部门诉求 |
| POST | `/api/v1/department/issues/:id/progress` | 更新进展 |
| POST | `/api/v1/department/issues/:id/reply` | 正式回复，->待审核 |

### 7.6 通用

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| GET | `/api/v1/departments` | 部门列表（登录即可） |
| POST | `/api/v1/issues/:id/attachments` | 上传图片（本人/员工） |
| GET | `/api/v1/attachments/:id` | 下载（含数据隔离） |
| GET | `/api/v1/notifications` | 我的通知 |
| POST | `/api/v1/notifications/:id/read` | 标记已读 |

## 8. 关键业务规则

1. **提交**：校验必填项与长度；分类必须是 `categories` 已启用项；`public_will` 记录学生公开意愿；`is_public` 仅由 worker 在受理/审核时设置。
2. **编号**：`CV + YYYYMMDD + 4位序列`，序列来自 PG sequence `issue_no_seq`。
3. **公开脱敏**：`public_cases` 存脱敏字段；通过审核写 `published_at` 后才在公开接口可见。审核页原始信息仅 worker/admin 可见。
4. **评价**：仅 `已办结` 可评，每人每诉求一次；写 `rating/rating_comment/rated_at`。
5. **通知节点**（REQUIREMENTS §8）：已受理、待补充、部门回复、已办结 → 通知学生；指派新诉求、临近超期 → 通知部门/负责人（超期提醒 v1 从简，仅记录 `deadline`）。
6. **审计**：worker/admin 的所有变更写 `audit_logs`（操作人、前后状态、说明、IP）。
7. **附件**：仅图片；5MB 上限、每诉求 3 张；磁盘 `uploads/` + `stored_name=uuid`；下载走鉴权与隔离；不提供原始图片到公开页。

## 9. 工程化

### 9.1 目录结构

```text
backend/
├── Cargo.toml
├── .env.example
├── Dockerfile
├── docker-compose.yml
├── migrations/            # 0001_schema.sql / 0002_seed.sql
├── src/
│   ├── main.rs            # 启动：配置->连接池->迁移->seed->路由->监听
│   ├── config.rs          # 环境变量配置
│   ├── error.rs           # AppError -> IntoResponse
│   ├── state.rs           # AppState(db,cfg,http)
│   ├── models.rs          # FromRow 实体
│   ├── auth.rs            # JWT/argon2/微信登录
│   ├── guards.rs          # AuthUser 提取器 + require_role
│   ├── domain/            # status/audit/notify/issue_no/desensitize
│   ├── handlers/          # auth/issues/admin/department/review/public/stats/notifications/attachments
│   └── routes.rs          # 路由组装
└── tests/                 # 集成测试（真实 Postgres）
```

### 9.2 配置（`.env`，不入库）

```dotenv
DATABASE_URL=postgres://campus_voice:campus_voice@localhost:5432/campus_voice
SERVER_ADDR=0.0.0.0:8080
JWT_SECRET=<随机长串>
JWT_EXPIRES_HOURS=168
WECHAT_APPID=
WECHAT_SECRET=
ADMIN_USERNAME=admin
ADMIN_PASSWORD=<必改>
UPLOAD_DIR=uploads
```

### 9.3 运行

```bash
# 本地（需已安装 PostgreSQL）
cd backend
cp .env.example .env
cargo run                 # 自动执行迁移 + seed，监听 8080

# Docker
docker compose up --build
```

### 9.4 测试与质量

- `cargo test`：集成测试连真实测试库（`DATABASE_URL` 指向独立库，迁移后跑）。
- 覆盖核心闭环：登录 → 提诉求 → 受理 → 指派 → 部门回复 → 审核 → 公开 → 评价，及越权/非法流转负例。
- lint：`cargo clippy`；format：`cargo fmt`。
- CI（后续）：GitHub Actions `cargo fmt --check && cargo clippy -D warnings && cargo test`。

### 9.5 安全基线

- 口令 argon2 散列，密码永不明文落库/日志。
- JWT_SECRET 走环境变量；`.env`、`uploads/`、`target/` gitignore。
- 所有用户输入经长度/枚举/类型校验；SQL 全部参数化（sqlx bind）。
- 附件访问隔离 + 越权返回 403；公开接口只暴露脱敏字段。

## 10. 迭代计划

- **M1（已完成）**：工程骨架 + 迁移 + 核心闭环 API + 认证 + 状态机 + 审计 + 通知 + 附件 + 集成测试 + 文档。
- **M2**：管理端 admin-web 接入真实 API（替换 mock）。
- **M3**：小程序端接入（微信登录 + token 存取 + 订阅消息）。
- **M4**：OpenAPI(utoipa)+swagger、CI、超期提醒、看板报表导出、对象存储迁移。
