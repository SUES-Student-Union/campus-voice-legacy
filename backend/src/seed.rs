use crate::domain::issue_no;
use anyhow::Context;
use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

/// 演示数据种子：注入一个开发学生、若干诉求与公开案例。
/// 幂等：仅当库中没有任何公开案例时执行。
pub async fn seed_demo(pool: &PgPool) -> anyhow::Result<()> {
    let has_cases: i64 = sqlx::query_scalar("SELECT count(*) FROM public_cases")
        .fetch_one(pool)
        .await?;
    if has_cases > 0 {
        return Ok(());
    }

    tracing::info!("开始注入演示数据（学生诉求 + 公开案例）");

    let mut tx = pool.begin().await?;

    // 开发学生（与 dev 登录的 openid 一致）
    let student = ensure_student(&mut tx).await?;

    // 演示工作人员（作为回复/进度操作人）
    let worker = ensure_demo_worker(&mut tx).await?;

    let logistics = find_dept(&mut tx, "logistics").await?;
    let ict = find_dept(&mut tx, "ict").await?;
    let capital = find_dept(&mut tx, "capital").await?;
    let student_affairs = find_dept(&mut tx, "student-affairs").await?;

    let now = Utc::now();

    // ---- 1. 我的诉求：公共洗衣房（部门处理中）----
    let i1 = insert_issue(
        &mut tx,
        student,
        "设施报修建议",
        "宿舍",
        "松江校区",
        "学生公寓区域",
        "公共洗衣房部分设备长期停用",
        "近期公共洗衣房有多台设备显示故障，希望尽快检修并张贴报修进度。",
        "部门处理中",
        true,
        Some(logistics),
        "维修单位已完成现场排查，配件正在调拨。",
        now - Duration::days(18),
    )
    .await?;
    push_progress(
        &mut tx,
        i1,
        "学生提交",
        Some(student),
        "学生",
        "诉求已提交并生成编号。",
        now - Duration::days(18),
    )
    .await?;
    push_progress(
        &mut tx,
        i1,
        "权益工作小组受理",
        Some(worker),
        "权益工作小组",
        "确认属于受理范围，转交后勤部门。",
        now - Duration::days(17),
    )
    .await?;
    push_progress(
        &mut tx,
        i1,
        "部门处理中",
        Some(worker),
        "后勤实业发展中心",
        "已完成现场排查，配件正在调拨。",
        now - Duration::days(14),
    )
    .await?;

    // ---- 2. 我的诉求：接驳车班次（待补充）----
    let i2 = insert_issue(
        &mut tx,
        student,
        "意见建议",
        "交通",
        "松江校区",
        "教学楼周边",
        "建议优化晚间校园接驳车班次",
        "晚课结束后候车人数较多，建议在高峰时段增加班次。",
        "待补充",
        false,
        None,
        "请补充常用候车站点和具体时段。",
        now - Duration::days(24),
    )
    .await?;
    push_progress(
        &mut tx,
        i2,
        "学生提交",
        Some(student),
        "学生",
        "诉求已提交并生成编号。",
        now - Duration::days(24),
    )
    .await?;
    push_progress(
        &mut tx,
        i2,
        "待补充",
        Some(worker),
        "权益工作小组",
        "请补充常用候车站点和具体时段。",
        now - Duration::days(23),
    )
    .await?;

    // ---- 3. 我的诉求：校园网咨询（已办结 + 公开案例）----
    let i3 = insert_issue(
        &mut tx,
        student,
        "问题咨询",
        "网络",
        "长宁校区",
        "教学区域",
        "校园网访客设备接入方式咨询",
        "想了解临时访客设备如何申请接入校园网络。",
        "已办结",
        true,
        Some(ict),
        "已提供访客网络申请指引。",
        now - Duration::days(38),
    )
    .await?;
    set_completed(&mut tx, i3, now - Duration::days(34)).await?;
    push_progress(
        &mut tx,
        i3,
        "学生提交",
        Some(student),
        "学生",
        "诉求已提交并生成编号。",
        now - Duration::days(38),
    )
    .await?;
    push_progress(
        &mut tx,
        i3,
        "已受理",
        Some(worker),
        "权益工作小组",
        "已转交信息化办公室。",
        now - Duration::days(37),
    )
    .await?;
    push_progress(
        &mut tx,
        i3,
        "部门回复",
        Some(worker),
        "信息化办公室",
        "已提供访客网络申请指引。",
        now - Duration::days(35),
    )
    .await?;
    push_progress(
        &mut tx,
        i3,
        "已办结",
        Some(worker),
        "权益工作小组",
        "正式回复已反馈学生。",
        now - Duration::days(34),
    )
    .await?;
    insert_reply(
        &mut tx,
        i3,
        ict,
        worker,
        "访客可由校内接待单位在校园服务平台发起临时网络申请，审核通过后按短信指引完成认证。",
        now - Duration::days(35),
    )
    .await?;
    insert_public_case(
        &mut tx,
        i3,
        "网络",
        "长宁校区",
        "长宁校区教学区域",
        "校园网访客设备接入指引",
        "同学咨询临时访客设备接入校园网络的方式。",
        "访客可由校内接待单位在校园服务平台发起临时网络申请，审核通过后按短信指引完成认证。",
        "已办结",
        now - Duration::days(38),
        Some(now - Duration::days(34)),
    )
    .await?;

    // ---- 4. 公开案例：食堂晚间档口（已办结）----
    let i4 = insert_issue(
        &mut tx,
        student,
        "意见建议",
        "食堂",
        "松江校区",
        "第一食堂区域",
        "关于晚间档口营业时间的建议",
        "同学建议考试周适当延长部分档口晚间营业时间。",
        "已办结",
        true,
        Some(logistics),
        "考试周部分档口延长营业至 21:30。",
        now - Duration::days(27),
    )
    .await?;
    set_completed(&mut tx, i4, now - Duration::days(21)).await?;
    push_progress(
        &mut tx,
        i4,
        "受理并转交",
        Some(worker),
        "权益工作小组",
        "已转交食堂管理部门评估。",
        now - Duration::days(26),
    )
    .await?;
    push_progress(
        &mut tx,
        i4,
        "部门处理",
        Some(worker),
        "后勤实业发展中心",
        "完成客流调研和档口协调。",
        now - Duration::days(24),
    )
    .await?;
    push_progress(
        &mut tx,
        i4,
        "审核公开",
        Some(worker),
        "权益工作小组",
        "脱敏审核通过并公开。",
        now - Duration::days(21),
    )
    .await?;
    insert_reply(
        &mut tx,
        i4,
        logistics,
        worker,
        "考试周期间将安排一楼两处档口延长营业至 21:30，并根据客流动态调整。",
        now - Duration::days(22),
    )
    .await?;
    insert_public_case(
        &mut tx,
        i4,
        "食堂",
        "松江校区",
        "松江校区食堂区域",
        "关于晚间档口营业时间的建议",
        "同学建议考试周适当延长部分档口晚间营业时间。",
        "考试周期间将安排一楼两处档口延长营业至 21:30，并根据客流动态调整。",
        "已办结",
        now - Duration::days(27),
        Some(now - Duration::days(21)),
    )
    .await?;

    // ---- 5. 公开案例：雨天连廊湿滑（已办结）----
    let i5 = insert_issue(
        &mut tx,
        student,
        "投诉反馈",
        "校园环境",
        "长宁校区",
        "图书馆周边",
        "雨天连廊地面湿滑问题",
        "同学反映雨天部分连廊容易积水，希望增加防滑提醒。",
        "已办结",
        true,
        Some(capital),
        "已增设防滑垫与警示牌。",
        now - Duration::days(33),
    )
    .await?;
    set_completed(&mut tx, i5, now - Duration::days(28)).await?;
    push_progress(
        &mut tx,
        i5,
        "受理并转交",
        Some(worker),
        "权益工作小组",
        "确认问题并转交校园管理部门。",
        now - Duration::days(32),
    )
    .await?;
    push_progress(
        &mut tx,
        i5,
        "现场处理",
        Some(worker),
        "基建处",
        "完成防滑设施增设。",
        now - Duration::days(30),
    )
    .await?;
    push_progress(
        &mut tx,
        i5,
        "审核公开",
        Some(worker),
        "权益工作小组",
        "脱敏审核通过并公开。",
        now - Duration::days(28),
    )
    .await?;
    insert_reply(
        &mut tx,
        i5,
        capital,
        worker,
        "已增设防滑垫与警示牌，并将该区域纳入雨天重点巡查清单。",
        now - Duration::days(29),
    )
    .await?;
    insert_public_case(
        &mut tx,
        i5,
        "校园环境",
        "长宁校区",
        "长宁校区图书馆周边",
        "雨天连廊地面湿滑问题",
        "同学反映雨天部分连廊容易积水，希望增加防滑提醒。",
        "已增设防滑垫与警示牌，并将该区域纳入雨天重点巡查清单。",
        "已办结",
        now - Duration::days(33),
        Some(now - Duration::days(28)),
    )
    .await?;

    // ---- 6. 公开案例：暑期证明材料（部门处理中）----
    let i6 = insert_issue(
        &mut tx,
        student,
        "问题咨询",
        "学生服务",
        "松江校区",
        "学生服务中心",
        "暑期证明材料办理时间咨询",
        "同学咨询暑期在校证明等材料的办理方式和服务时间。",
        "部门处理中",
        true,
        Some(student_affairs),
        "正在汇总暑期值班和线上办理安排。",
        now - Duration::days(16),
    )
    .await?;
    push_progress(
        &mut tx,
        i6,
        "受理并转交",
        Some(worker),
        "权益工作小组",
        "已转交学生工作部门。",
        now - Duration::days(15),
    )
    .await?;
    push_progress(
        &mut tx,
        i6,
        "部门处理中",
        Some(worker),
        "学生工作部",
        "正在汇总暑期值班和线上办理安排。",
        now - Duration::days(14),
    )
    .await?;
    insert_public_case(
        &mut tx,
        i6,
        "学生服务",
        "松江校区",
        "松江校区学生服务中心",
        "暑期证明材料办理时间咨询",
        "同学咨询暑期在校证明等材料的办理方式和服务时间。",
        "相关暑期服务安排正在汇总，将通过校园公告统一发布。",
        "部门处理中",
        now - Duration::days(16),
        None,
    )
    .await?;

    tx.commit().await?;
    tracing::info!("演示数据注入完成（6 条诉求，4 条公开案例）");
    Ok(())
}

async fn ensure_student(tx: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> anyhow::Result<Uuid> {
    if let Some(id) =
        sqlx::query_scalar::<_, Uuid>("SELECT id FROM users WHERE openid = 'dev_student'")
            .fetch_optional(&mut **tx)
            .await?
    {
        return Ok(id);
    }
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO users (openid, role, name) VALUES ('dev_student', 'student', '开发同学') RETURNING id",
    )
    .fetch_one(&mut **tx)
    .await?;
    Ok(id)
}

async fn find_dept(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    code: &str,
) -> anyhow::Result<Uuid> {
    sqlx::query_scalar::<_, Uuid>("SELECT id FROM departments WHERE code = $1")
        .bind(code)
        .fetch_one(&mut **tx)
        .await
        .with_context(|| format!("找不到部门 {code}"))
}

async fn ensure_demo_worker(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> anyhow::Result<Uuid> {
    if let Some(id) =
        sqlx::query_scalar::<_, Uuid>("SELECT id FROM users WHERE username = 'demo_worker'")
            .fetch_optional(&mut **tx)
            .await?
    {
        return Ok(id);
    }
    let hash = crate::auth::hash_password("demo123")?;
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO users (username, password_hash, role, name) VALUES ('demo_worker', $1, 'worker', '演示工作人员') RETURNING id",
    )
    .bind(&hash)
    .fetch_one(&mut **tx)
    .await?;
    Ok(id)
}

/// 确保演示账号存在（幂等）：demo_worker（权益小组）、dept_staff1（后勤部门老师）。
pub async fn ensure_demo_accounts(pool: &PgPool) -> anyhow::Result<()> {
    let mut tx = pool.begin().await?;
    ensure_demo_worker(&mut tx).await?;
    if sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM users WHERE username = 'dept_staff1')",
    )
    .fetch_one(&mut *tx)
    .await?
    {
        return Ok(());
    }
    let dept = find_dept(&mut tx, "logistics").await?;
    let hash = crate::auth::hash_password("demo123")?;
    sqlx::query(
        "INSERT INTO users (username, password_hash, role, name, department_id)
         VALUES ('dept_staff1', $1, 'department_staff', '演示后勤老师', $2)",
    )
    .bind(&hash)
    .bind(dept)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn insert_issue(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    student: Uuid,
    kind: &str,
    category: &str,
    campus: &str,
    location: &str,
    title: &str,
    description: &str,
    status: &str,
    is_public: bool,
    department_id: Option<Uuid>,
    latest_progress: &str,
    created_at: chrono::DateTime<chrono::Utc>,
) -> anyhow::Result<Uuid> {
    let issue_no = issue_no::next_issue_no(tx).await?;
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO issues
           (issue_no, submitter_id, kind, category, campus, location, title, description,
            status, is_public, public_will, department_id, latest_progress, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $10, $11, $12, $13, $13)
         RETURNING id",
    )
    .bind(issue_no)
    .bind(student)
    .bind(kind)
    .bind(category)
    .bind(campus)
    .bind(location)
    .bind(title)
    .bind(description)
    .bind(status)
    .bind(is_public)
    .bind(department_id)
    .bind(latest_progress)
    .bind(created_at)
    .fetch_one(&mut **tx)
    .await?;
    Ok(id)
}

async fn set_completed(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    issue_id: Uuid,
    completed_at: chrono::DateTime<chrono::Utc>,
) -> anyhow::Result<()> {
    sqlx::query("UPDATE issues SET completed_at = $1, updated_at = $1 WHERE id = $2")
        .bind(completed_at)
        .bind(issue_id)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

async fn push_progress(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    issue_id: Uuid,
    status: &str,
    operator_id: Option<Uuid>,
    operator_name: &str,
    note: &str,
    created_at: chrono::DateTime<chrono::Utc>,
) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO issue_progress (issue_id, status, operator_id, operator_name, note, created_at)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(issue_id)
    .bind(status)
    .bind(operator_id)
    .bind(operator_name)
    .bind(note)
    .bind(created_at)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn insert_reply(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    issue_id: Uuid,
    department_id: Uuid,
    author_id: Uuid,
    content: &str,
    created_at: chrono::DateTime<chrono::Utc>,
) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO replies (issue_id, department_id, author_id, content, created_at)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(issue_id)
    .bind(department_id)
    .bind(author_id)
    .bind(content)
    .bind(created_at)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn insert_public_case(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    issue_id: Uuid,
    category: &str,
    campus: &str,
    location: &str,
    title: &str,
    summary: &str,
    reply: &str,
    status: &str,
    submitted_at: chrono::DateTime<chrono::Utc>,
    completed_at: Option<chrono::DateTime<chrono::Utc>>,
) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO public_cases
           (issue_id, category, campus, location, title, summary, reply, status, submitted_at, completed_at, published_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
    )
    .bind(issue_id)
    .bind(category)
    .bind(campus)
    .bind(location)
    .bind(title)
    .bind(summary)
    .bind(reply)
    .bind(status)
    .bind(submitted_at)
    .bind(completed_at)
    .bind(Utc::now())
    .execute(&mut **tx)
    .await?;
    Ok(())
}
