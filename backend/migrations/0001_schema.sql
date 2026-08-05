-- CampusVoice 核心 schema
-- 依赖：PostgreSQL 13+（gen_random_uuid 内建）

CREATE TABLE departments (
    id          uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name        text NOT NULL UNIQUE,
    code        text NOT NULL UNIQUE,
    description text,
    created_at  timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE categories (
    id      uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name    text NOT NULL UNIQUE,
    sort    int  NOT NULL DEFAULT 0,
    enabled bool NOT NULL DEFAULT true
);

CREATE TABLE users (
    id            uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    openid        text UNIQUE,
    username      text UNIQUE,
    password_hash text,
    role          text NOT NULL DEFAULT 'student'
                  CHECK (role IN ('student', 'worker', 'department_staff', 'admin')),
    name          text NOT NULL,
    student_no    text,
    department_id uuid REFERENCES departments(id),
    phone_masked  text,
    created_at    timestamptz NOT NULL DEFAULT now(),
    updated_at    timestamptz NOT NULL DEFAULT now(),
    CHECK (role = 'student' OR (username IS NOT NULL AND password_hash IS NOT NULL))
);

CREATE TABLE issues (
    id              uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    issue_no        text NOT NULL UNIQUE,
    student_id      uuid NOT NULL REFERENCES users(id),
    kind            text NOT NULL,
    category        text NOT NULL,
    campus          text NOT NULL,
    location        text NOT NULL,
    title           text NOT NULL,
    description     text NOT NULL,
    status          text NOT NULL DEFAULT '待受理'
                    CHECK (status IN ('待受理', '待补充', '已受理', '部门处理中', '待审核', '已办结', '已关闭')),
    priority        text NOT NULL DEFAULT '中' CHECK (priority IN ('高', '中', '低')),
    is_public       bool NOT NULL DEFAULT false,
    public_will     bool NOT NULL DEFAULT false,
    contact         text NOT NULL DEFAULT '微信联系',
    department_id   uuid REFERENCES departments(id),
    assignee_id     uuid REFERENCES users(id),
    deadline        timestamptz,
    latest_progress text NOT NULL DEFAULT '',
    rating          text CHECK (rating IN ('满意', '一般', '不满意')),
    rating_comment  text,
    rated_at        timestamptz,
    created_at      timestamptz NOT NULL DEFAULT now(),
    updated_at      timestamptz NOT NULL DEFAULT now(),
    accepted_at     timestamptz,
    completed_at    timestamptz,
    closed_at       timestamptz
);

CREATE INDEX idx_issues_student   ON issues(student_id, created_at DESC);
CREATE INDEX idx_issues_status    ON issues(status);
CREATE INDEX idx_issues_department ON issues(department_id);
CREATE INDEX idx_issues_category  ON issues(category);
CREATE INDEX idx_issues_created   ON issues(created_at DESC);

CREATE TABLE issue_progress (
    id            uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    issue_id      uuid NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    status        text NOT NULL,
    operator_id   uuid REFERENCES users(id),
    operator_name text NOT NULL,
    note          text NOT NULL DEFAULT '',
    created_at    timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX idx_progress_issue ON issue_progress(issue_id, created_at);

CREATE TABLE replies (
    id            uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    issue_id      uuid NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    department_id uuid REFERENCES departments(id),
    author_id     uuid NOT NULL REFERENCES users(id),
    content       text NOT NULL,
    created_at    timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX idx_replies_issue ON replies(issue_id, created_at);

CREATE TABLE attachments (
    id            uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    issue_id      uuid REFERENCES issues(id) ON DELETE CASCADE,
    uploader_id   uuid NOT NULL REFERENCES users(id),
    original_name text NOT NULL,
    stored_name   text NOT NULL UNIQUE,
    size          bigint NOT NULL,
    mime          text NOT NULL,
    is_public     bool NOT NULL DEFAULT false,
    created_at    timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX idx_attachments_issue ON attachments(issue_id);

CREATE TABLE notifications (
    id         uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id    uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title      text NOT NULL,
    content    text NOT NULL,
    kind       text NOT NULL DEFAULT 'system',
    read       bool NOT NULL DEFAULT false,
    issue_id   uuid REFERENCES issues(id) ON DELETE SET NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX idx_notifications_user ON notifications(user_id, read, created_at DESC);

CREATE TABLE audit_logs (
    id          uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    operator_id uuid REFERENCES users(id),
    action      text NOT NULL,
    target_type text NOT NULL,
    target_id   text,
    before      text,
    after       text,
    detail      text NOT NULL DEFAULT '',
    ip          text,
    created_at  timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX idx_audit_created ON audit_logs(created_at DESC);

CREATE TABLE public_cases (
    id           uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    issue_id     uuid NOT NULL UNIQUE REFERENCES issues(id) ON DELETE CASCADE,
    category     text NOT NULL,
    campus       text NOT NULL,
    location     text NOT NULL,
    title        text NOT NULL,
    summary      text NOT NULL,
    reply        text NOT NULL,
    status       text NOT NULL,
    submitted_at timestamptz NOT NULL,
    completed_at timestamptz,
    published_at timestamptz,
    created_at   timestamptz NOT NULL DEFAULT now(),
    updated_at   timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX idx_public_published ON public_cases(published_at DESC) WHERE published_at IS NOT NULL;

CREATE SEQUENCE issue_no_seq;
