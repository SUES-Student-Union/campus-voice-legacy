-- 统一身份改造（M3）：
-- 1) 学校统一认证（学号/工号）为权威身份源，微信 openid 仅作绑定。
-- 2) 老师也可提交反馈：issues.student_id 更名 submitter_id。

ALTER TABLE users ADD COLUMN campus_id text UNIQUE;

-- 允许员工凭 campus_id（工号）存在，而不强制 username/password。
ALTER TABLE users DROP CONSTRAINT IF EXISTS users_check;
ALTER TABLE users ADD CONSTRAINT users_check CHECK (
    role = 'student' OR campus_id IS NOT NULL OR (username IS NOT NULL AND password_hash IS NOT NULL)
);

ALTER TABLE issues RENAME COLUMN student_id TO submitter_id;
