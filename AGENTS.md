# CampusVoice（程园小二）项目 AGENTS.md

本文件是本仓库（小程序 + 管理端 + Rust 后端）的**项目级**补充约定。全局规则见 `~/.config/opencode/AGENTS.md`，冲突时以本项目为准。

## 一、总体状态（2026-08）

| 部分 | 状态 |
| --- | --- |
| `miniprogram/` | 学生端已接入本地后端（无 mock）；**管理端规划为小程序内页面**（角色：worker/department_staff/admin） |
| `backend/` | Rust + axum + PostgreSQL，M1 完成，`cargo test` 通过 |
| `admin-web/` | **旧独立网页原型**（`index.html` 浏览器打开，数据在 `app.js` mock），保留作界面参考，不再是目标形态 |
| 仓库 | 已 `git init`；`git status` 检查后手动提交，**绝不自动 commit** |

## 二、运行方式

### 后端（Rust）
```bash
cd backend
cargo run                  # 启动前自动执行迁移 + 创建管理员账号（读 .env）
cargo test -- --test-threads=1   # 集成测试共享 test 库，必须单线程！
cargo clippy --all-targets
cargo fmt --check
```
- `.env` 由 `.env.example` 复制而来，**不入库**。缺失 `JWT_SECRET`（≥32 字符）/`ADMIN_PASSWORD` 会启动失败。
- 本地 Postgres：用户/密码 `campus_voice`，库 `campus_voice`（开发）/ `campus_voice_test`（测试）。
- 新迁移文件放 `backend/migrations/000N_*.sql`，启动时自动执行（sqlx migrate）。

### 停止后端 —— 用端口，不要用 pkill！
```bash
fuser -k 8080/tcp
```
> ⚠️ 教训：`pkill -f "campus-voice-backend"` 会匹配到当前 shell 命令行里同样的字符串，**把 shell 自己杀掉**（表现为命令卡死超时）。用 `fuser -k <端口>` 或先 `pgrep` 拿 PID 再 `kill`。

### 小程序（微信开发者工具）
1. 后端先 `cargo run`。
2. 开发者工具 → 详情 → 本地设置 → 勾选 **「不校验合法域名、web-view（业务域名）、TLS 版本以及 HTTPS 证书」**，否则请求全挂。
3. `miniprogram/utils/api.ts` 的 `BASE_URL`：模拟器用 `127.0.0.1` 即可；**真机调试/预览必须改成开发机局域网 IP**（当前已按开发机 `192.168.31.159` 配置，IP 变化需同步改）。

## 三、常用命令
```bash
npm run typecheck          # 小程序 TS 静态检查（必跑）
npm run check:admin        # 管理端 app.js 语法
cargo test -- --test-threads=1
cargo clippy --all-targets && cargo fmt --check
```

## 四、架构速查

### 后端（`backend/`）
- **库 + 薄二进制**：`src/lib.rs`（导出全部模块）+ `src/main.rs`（启动）。改后端逻辑先看 lib；集成测试只能 `use campus_voice_backend::...`。
- 分层：`handlers/`（参数/权限/响应）→ `domain/`（状态机、审计、通知、编号）→ `models.rs`（`FromRow` 实体 + 输入/输出 DTO）。
- 写操作用事务：`state.db.begin()` → `&mut *tx` → `tx.commit()`。
- **状态机**在 `domain/status.rs`：所有 `issues.status` 变更必须过 `assert_transition`，非法流转返回 409。
- **权限层**在 `src/permissions.rs`（借鉴 LuckPerms：组继承 + 权限节点 + 上下文作用域）。新增受保护操作时**加权限节点**，handler 里用 `user.check(node, &Ctx::for_issue(&issue))?`，不要手写 `role` 判断或归属校验。角色常量在 `src/auth.rs`。
- 每次状态变更写 `issue_progress` + `audit_logs`（需带操作人）。

### 小程序（`miniprogram/`）
- `utils/api.ts`：`BASE_URL`、token（key=`cv_token`）、`get/post`、`uploadImages`（`wx.uploadFile`）、`downloadAttachment`。
- `types/api.ts`：后端 DTO（**snake_case**）→ 页面展示模型（camelCase），含「状态中文 → `statusKey`」映射（供 `status-{{statusKey}}` 配色）。
- 登录：`app.ts` onLaunch 调 `ensureLogin()`；后端无微信 appid 时**固定 `dev_student`** 身份（所有开发设备同一学生）。
- **统一身份（M3）**：身份权威源是学校认证（`sso.rs`），开发用 mock——`admin`→管理员、工号 `W` 开头→权益小组、`T` 开头→后勤部门老师、其余→学生；`POST /auth/sso` 绑微信 openid。`issues.submitter_id` = 提交人（**老师也可提交反馈**，不再是 `student_id`）。学生页不再对员工账号重定向。
- 补充材料页 `pages/supplement/` 已注册进 `app.json`。

### 管理端（小程序内，规划中）
- 目标形态：在小程序内实现 worker / department_staff / admin 角色的管理页面（受理、流转、回复、审核、日志、看板），复用学生端登录体系 + 角色路由。
- 入口：首页「身份切换」卡片直接切换角色（学生/权益小组/部门老师/管理员，开发演示用）；员工角色会显示「进入管理端」。已删除独立 `admin/login` 页。
- 演示账号（seed 幂等创建）：管理员 `admin`（`ADMIN_PASSWORD`），权益小组 `demo_worker`/`demo123`，后勤老师 `dept_staff1`/`demo123`；sso dev mock 中 `W` 开头工号→权益小组、`T` 开头→后勤老师、`admin`→管理员。
- `admin-web/`（旧独立网页原型）：数据在 `app.js` 内 mock 数组，仅作界面参考；接口契约见 `backend/DESIGN.md`。

## 五、踩过的坑 / 高频易错

1. **axum 0.8 路由参数用 `{id}`，不是 `:id`**（matchit 0.8 语法）。`:id` 编译不报错、运行时 404/panic。
2. **集成测试共享 `campus_voice_test` 库**，每个 test 开头 TRUNCATE → 必须 `--test-threads=1`，否则互相清库。
3. **sqlx 用运行时查询**（`query_as` + `FromRow`），未启用编译期 `query!` 宏，编译不依赖 DATABASE_URL；动态筛选用 `query_as_with` + `PgArguments` + `push_arg`。
4. **开发登录身份必须稳定**：`wx.login` 的 code 每次变化，不能当 openid（已改为固定 `dev_student`）。
5. **后端字段是 snake_case**（`issue_no`、`department_name`…），小程序展示层在 `types/api.ts` 里转 camelCase，别在 WXML 里直接引后端字段。
6. **`sudo` 需密码**：系统级安装（如 postgres）给用户命令让其执行，不要尝试交互式 sudo。
7. **`.env`、`backend/target/`、`backend/uploads/`、`project.private.config.json` 均 gitignore**；`.env` 修改后需重启后端。
8. 首页「平均评分」无学生侧聚合源时显示 `—`；不要写死数字。
9. 新增页面的 `json`、`wxss`、`wxml`、`ts` 四个文件都要建，并注册进 `app.json` 的 `pages`。
10. 改后端 DTO 后，小程序 `types/api.ts` 映射、集成测试断言可能都需要同步更新；跑 `cargo test` + `npm run typecheck` 双验证。
11. **微信开发者工具（Linux IDE 2.01 等旧版）不认 ES2020 语法**：小程序 TS 里不要用可选链 `?.` / 空值合并 `??`（编译报 `invalid file: xxx.js: SyntaxError: Unexpected token .`）。`tsconfig.json` 的 `target` 已设 `ES2018` 兜底，但源码仍应避免直接写 `?.`/`??`。
12. **统一身份字段语义**：诉求的提交人字段是 `submitter_id`（学生和老师都能提交），不是 `student_id`；用户权威身份是 `campus_id`（学号/工号）。`sso.rs` 目前是 dev mock（`T` 开头工号→后勤老师、`admin`→管理员），上线时替换为真实学校认证调用。

## 六、工作纪律（继承全局，简记）
- 简体中文回复，先结论后细节。
- 默认只读；改动前说明将创建/修改的文件，完成后汇报清单。
- 验证以行为为准：小改动定向检查，跨端接口改动跑 `cargo test` + `npm run typecheck`。
- 不自动提交、不主动重构无关代码、不留技术债（数据字段/契约两端保持一致）。
