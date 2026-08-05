# CampusVoice 后端设计文档（Rust）

> 版本：v0.2｜状态：M1 已实现（2026-08）
> 关联需求：[`REQUIREMENTS.md`](../REQUIREMENTS.md)｜前端：微信小程序 `miniprogram/`（学生端 + 规划中的小程序管理端）

## 1. 概述

### 1.1 背景

当前仓库是「程园小二」的双端高保真 Demo：小程序端已接入真实后端；管理端将以**小程序内页面**实现（worker/department_staff/admin 角色），旧 `admin-web/` 独立网页仅作界面参考。本后端为 V1 提供真实可演进的 API 服务，支撑完整权益服务闭环：

```text
提交诉求 -> 分类受理 -> 部门流转 -> 跟进处理 -> 结果反馈 -> 脱敏公开
```

### 1.2 目标（本轮范围）

- 纯后端 API 服务 + 数据库迁移 + 自动化测试 + 工程文档。
- 小程序学生端已接入真实 API；管理端（worker/department_staff/admin）规划为小程序内页面并接入同一套 API。
- 覆盖 V1 验收标准：提交/附件、状态进度查询、受理流转、部门回复、审核脱敏公开、操作日志、通知、满意度评价。

### 1.3 非目标

- 独立 Web 管理端（旧 `admin-web/` 网页仅作界面参考，不再演进）。
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
- 小程序学生端：`wx.login()` -> code -> `POST /api/v1/auth/wechat` -> 拿 JWT -> 请求头 `Authorization: Bearer <token>`。
- 管理端（小程序内页面）：员工账号 -> `POST /api/v1/auth/login` -> JWT，按角色路由。
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
| `users` | 学生/员工/管理员 | `campus_id`(学号/工号,唯一,统一身份)、`openid`(微信绑定)、`username`/`password_hash`(员工登录)、`role`、`department_id` |
| `departments` | 职能部门 | `name`、`code` |
| `categories` | 诉求分类 | `name`、`sort`（种子：宿舍/食堂/教学/交通/校园环境/网络/医疗健康/学生服务/其他） |
| `issues` | 诉求主表 | `issue_no`、`submitter_id`(提交人=学生或老师)、`kind`、`category`、`campus`、`location`、`status`、`priority`、`is_public`、`public_will`、`department_id`、`assignee_id`、`deadline`、`latest_progress`、`rating`/`rating_comment`/`rated_at` |
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

### 5.1 角色与权限模型（借鉴 LuckPerms：组 + 节点 + 上下文）

实现见 `src/permissions.rs`。用户角色即「组」，组之间可继承，权限按**节点**授权，节点带**上下文作用域**：

```text
admin ──继承──> worker ──继承──> student
department_staff ──继承──> student
```

| 作用域 | 含义 | 示例 |
| --- | --- | --- |
| `Any` | 无条件 | `issue.accept`（worker+）、`issue.submit`（student） |
| `Own` | 仅本人诉求 | `issue.view.own`、`issue.supplement`、`issue.rate`、`attachment.upload` |
| `Dept` | 仅本部门诉求 | `issue.reply`、`issue.progress`、`issue.view.all`（部门老师） |

- 关键节点：`issue.submit/view.own/view.all/supplement/rate/accept/assign/request_supplement/close/merge/progress/reply`、`review.save/approve/reject`、`audit.view`、`dashboard.view`、`department.manage`、`attachment.upload/download`、`notification.view`。
- 校验入口：`AuthUser::check(node, ctx)`；上下文 `Ctx::for_issue(&issue)` 携带 `owner=学生`、`dept=负责部门`，一个函数同时完成「角色/越权」与「数据范围」判断，取代散落的 `require_role` + 归属判断。
- JWT `Claims` 内附 `dept`，避免每次校验查库。
- 与 LuckPerms 的取舍：V1 不做动态节点存储、显式否定（`!node`）与 web 编辑器；组与节点为声明式常量。

### 5.2 登录（统一身份）

**身份权威源是学校统一认证（学号/工号 + 密码），微信 openid 仅是快捷通道。**

- **学生/老师**：`POST /api/v1/auth/sso {campus_id, password, code}`。后端经 `sso.rs` 适配器调学校统一认证（CAS/OAuth）验证，拿到学号/工号、姓名、角色与部门，创建/更新用户并**绑定当前微信 openid**，签发 JWT。首次需账号密码，之后 `wx.login` 走 openid 快捷登录。
- **老师可提交反馈**：所有人都是「用户」，都可提交诉求、查看「我的诉求」；员工（worker/department_staff/admin）额外有管理权限。`issues.submitter_id` 因此命名，而非 `student_id`。
- **开发降级**：未配置学校认证时，`sso.rs` 用 mock 校验（任意账号或固定演示账号），接口契约一致，上线替换实现。
- **员工账号**：同走统一认证（工号）后自动带角色/部门；不再单独维护 `username/password` 登录（保留兼容）。

**网络边界（部署层）**：服务部署于校园内网，仅校园网可达；校外经学校 VPN（vpn.sues.edu.cn）接入，防止校外攻击。代码侧配合 HTTPS、限流、密钥环境变量化。

### 5.3 会话

- JWT HS256，`sub`=user_id、`role`、`name`、`dept`、`exp`；有效期由 `JWT_EXPIRES_HOURS` 控制（默认 168h）。
- 请求经 `AuthUser` 提取器校验签名与有效期；`AuthUser::check` 做权限节点校验。
- 附件下载 `/attachments/:id` 单独做数据隔离：公开附件任意登录用户、否则按 `attachment.download` 上下文（学生本人 / 部门 / worker）。

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

### 9.6 部署拓扑（微信准入 + 校园网安全）

**微信小程序生产环境强制**：`wx.request`/`wx.uploadFile` 的域名必须是**已备案、HTTPS、且已加入小程序后台白名单**的域名；裸 IP、未备案域名、明文 `http://` 均不可用。该限制仅在「开发者工具模拟器 / 自动真机调试」勾选「不校验合法域名」时可绕过，**扫码预览 / 体验版 / 发布版强制生效**。

**推荐拓扑（高校标准做法，同时满足安全与准入）**：

```text
微信小程序 --HTTPS--> api.sues.edu.cn（学校已备案子域 + HTTPS 反代/网关，位于校园网边界）
                          │ 校内转发
                          ▼
                    后端（校园内网，外部不可直连）
```

- 后端保持在校园网隔离，外部无法直接触达（防攻击目标达成）；
- 微信只连学校的反向代理，对外仅暴露这一个 HTTPS 入口；
- 域名备案走学校主体（`sues.edu.cn` 子域），无需自行备案；
- 需要向学校信息中心申请：已备案子域名、HTTPS 证书、反代/网关、校内转发到后端端口。

**退路（学校不提供域名/反代）**：自购域名 → 大陆备案（约 1–3 周）→ HTTPS 证书 → 解析到公网服务器；此时后端在公网，安全依赖 **HTTPS + 限流 + IP 白名单（仅校园网段/微信来源）**，放弃网络隔离。当前阿里云 `139.196.79.195:8080` 仅用于联调。

## 10. 迭代计划

- **M1（已完成）**：工程骨架 + 迁移 + 核心闭环 API + 认证 + 状态机 + 审计 + 通知 + 附件 + 集成测试 + 文档。
- **M2（已完成）**：小程序内管理端页面（worker/department_staff/admin：受理、流转、回复、审核、日志、看板）；LuckPerms 式权限层（组继承 + 节点 + 上下文）。
- **M3（进行中）**：统一身份改造——学校统一认证（学号/工号+密码 + openid 绑定，`sso.rs` 适配器）、`users.campus_id`、`issues.student_id`→`submitter_id`、前端去掉员工守卫（老师也可提交反馈）；网络边界校园网+VPN（部署层）。
- **M4**：微信订阅消息、超期提醒、OpenAPI(utoipa)+swagger、CI、对象存储迁移。
