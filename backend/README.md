# CampusVoice 后端（Rust）

「程园小二」权益服务平台的真实后端 API。设计文档见 [`DESIGN.md`](./DESIGN.md)。

## 技术栈

axum · tokio · sqlx(PostgreSQL) · JWT · argon2 · tracing

## 本地运行

前置：PostgreSQL 已启动，且已创建用户与库：

```sql
CREATE USER campus_voice WITH PASSWORD 'campus_voice';
CREATE DATABASE campus_voice OWNER campus_voice;
CREATE DATABASE campus_voice_test OWNER campus_voice;
```

```bash
cp .env.example .env        # 修改 JWT_SECRET 与 ADMIN_PASSWORD
cargo run                   # 自动执行迁移 + 创建管理员账号，监听 8080
```

启动后默认管理员：`.env` 中 `ADMIN_USERNAME/ADMIN_PASSWORD`。

## 测试

```bash
cargo test -- --test-threads=1    # 集成测试连 TEST_DATABASE_URL（默认 campus_voice_test）
cargo clippy --all-targets
cargo fmt --check
```

## Docker

```bash
JWT_SECRET=<长随机串> ADMIN_PASSWORD=<强密码> docker compose up --build
```

## 关键端点

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| POST | `/api/v1/auth/wechat` | 学生微信登录（未配置 appid 时开发降级） |
| POST | `/api/v1/auth/login` | 员工账号登录 |
| POST | `/api/v1/issues` | 学生提交诉求 |
| GET | `/api/v1/admin/issues` | 权益小组/管理员诉求列表 |
| POST | `/api/v1/admin/issues/{id}/accept` | 受理 |
| POST | `/api/v1/admin/issues/{id}/assign` | 指派部门 |
| POST | `/api/v1/department/issues/{id}/reply` | 部门正式回复 |
| POST | `/api/v1/admin/issues/{id}/review` | 审核并脱敏公开 |
| GET | `/api/v1/public/cases` | 公开脱敏案例 |

完整契约见 [`DESIGN.md`](./DESIGN.md) §7、§8。
