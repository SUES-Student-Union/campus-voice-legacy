# CampusVoice（程园小二）

> 上海工程技术大学校学生会权益生活部 · 权益工作小组

面向同学的线上权益服务入口：通过微信小程序收集同学的意见、建议、咨询与投诉，由权益工作小组分类流转、跟进处理；相关职能部门老师可在线更新进度并回复，同学可查询办理结果。

当前状态（2026-08）：

- **微信小程序学生端**：已接入本地后端（Rust + PostgreSQL），无 mock。提交诉求、生成编号、查看进度与时间线、补充材料、满意度评价、公开反馈筛选与脱敏详情、图片上传。
- **管理端**：规划为小程序内页面（worker / department_staff / admin 角色：受理、流转、回复、审核、日志、看板），复用学生端登录体系 + 角色路由。旧 `admin-web/` 独立网页仅作界面参考，不再是目标形态。
- **后端**：`backend/`，Rust + axum + PostgreSQL，设计与接口契约见 [`backend/DESIGN.md`](backend/DESIGN.md)。

## 运行方式

### 微信小程序（学生端）

使用微信开发者工具打开项目根目录（`miniprogramRoot` 已指向 `miniprogram/`）。

1. 启动后端：`cd backend && cargo run`（首次自动执行数据库迁移、创建管理员账号并注入演示数据）。
2. 开发者工具「详情 → 本地设置」勾选 **「不校验合法域名、web-view（业务域名）、TLS 版本以及 HTTPS 证书」**。
3. 编译运行。开发模式下 `wx.login` 自动登录（后端未配置微信 appid 时使用固定 `dev_student` 学生身份）。

### 管理端

规划为小程序内页面（开发中）。当前可参考旧网页原型 `admin-web/index.html`（浏览器打开，纯 mock 数据）。

## 数据说明

- 小程序端数据来自本地后端 PostgreSQL；启动时注入演示诉求与脱敏公开案例。
- 后端 `backend/` 的设计与接口契约见 [`backend/DESIGN.md`](backend/DESIGN.md)。

## 开发检查

```bash
npm run typecheck              # 小程序 TS 静态检查
npm run check:admin            # 旧 admin-web 原型语法
cd backend && cargo test -- --test-threads=1 && cargo clippy --all-targets && cargo fmt --check
```

## Demo 边界

- 学生端已接入真实服务端 API 与数据库；暂未接入真实微信登录（dev 降级）、订阅消息、邮件。
- 管理端小程序页面未完成（M2）；`admin-web/` 为纯 mock 原型。
- 正式开发时应由后端实施身份校验、部门数据隔离、附件访问控制和审计日志持久化（后端已实现）。
