use axum::http::{Method, StatusCode};
use serde_json::json;

mod common;
use common::*;

fn create_issue_body() -> serde_json::Value {
    json!({
        "kind": "设施报修建议",
        "category": "宿舍",
        "campus": "松江校区",
        "location": "学生公寓区域",
        "title": "公共洗衣房设备长期停用",
        "description": "希望尽快检修公共洗衣房的多台故障设备，并张贴报修进度。",
        "public_will": true,
        "contact": ""
    })
}

#[tokio::test]
async fn full_core_flow() -> anyhow::Result<()> {
    let app = setup().await?;
    register_admin(&app).await?;
    let dept_staff = create_department_staff(&app, "staff1", "pw").await?;

    let admin = login(&app, "admin", "admin-pw").await?;
    let student = student_token(&app).await?;

    // 1. 学生提交诉求
    let (status, body) = request(
        &app,
        Method::POST,
        "/api/v1/issues",
        Some(&student),
        Some(create_issue_body()),
    )
    .await?;
    assert_eq!(status, StatusCode::OK, "{body}");
    let issue_id = body["id"].as_str().unwrap().to_string();
    assert_eq!(body["status"], "待受理");
    assert!(body["issue_no"].as_str().unwrap().starts_with("CV"));
    assert_eq!(body["progress"].as_array().unwrap().len(), 1);
    assert_eq!(body["replies"].as_array().unwrap().len(), 0);

    // 2. 管理员受理
    let (status, body) = request(
        &app,
        Method::POST,
        &format!("/api/v1/admin/issues/{issue_id}/accept"),
        Some(&admin),
        Some(json!({ "category": "宿舍", "priority": "高", "is_public": true })),
    )
    .await?;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["status"], "已受理");

    // 3. 指派部门与负责人 -> 部门处理中
    let (status, body) = request(
        &app,
        Method::POST,
        &format!("/api/v1/admin/issues/{issue_id}/assign"),
        Some(&admin),
        Some(json!({ "department_id": dept_staff.department_id, "assignee_id": dept_staff.id })),
    )
    .await?;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["status"], "部门处理中");

    // 4. 部门老师更新进展
    let staff_token = login(&app, "staff1", "pw").await?;
    let (status, _) = request(
        &app,
        Method::POST,
        &format!("/api/v1/department/issues/{issue_id}/progress"),
        Some(&staff_token),
        Some(json!({ "note": "已完成现场排查，配件调拨中。" })),
    )
    .await?;
    assert_eq!(status, StatusCode::OK);

    // 5. 部门正式回复 -> 待审核
    let (status, body) = request(
        &app,
        Method::POST,
        &format!("/api/v1/department/issues/{issue_id}/reply"),
        Some(&staff_token),
        Some(json!({ "content": "已安排维修单位现场排查，配件调拨中，预计下周完成更换。" })),
    )
    .await?;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["status"], "待审核");
    assert_eq!(body["replies"].as_array().unwrap().len(), 1);

    // 6. 管理员审核通过并公开 -> 已办结
    let (status, body) = request(
        &app,
        Method::POST,
        &format!("/api/v1/admin/issues/{issue_id}/review"),
        Some(&admin),
        Some(json!({
            "action": "approve",
            "title": "公共洗衣房设备维修",
            "location": "松江校区学生公寓区域",
            "summary": "有同学反映公共洗衣房多台设备故障，希望尽快检修。",
            "reply": "已安排维修单位现场排查并调拨配件，预计下周完成更换。",
            "note": ""
        })),
    )
    .await?;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["status"], "已办结");

    // 7. 公开列表可见（无需登录）
    let (status, list) = request(&app, Method::GET, "/api/v1/public/cases", None, None).await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list.as_array().unwrap().len(), 1);

    // 8. 学生评价 + 仅一次
    let (status, body) = request(
        &app,
        Method::POST,
        &format!("/api/v1/my/issues/{issue_id}/rating"),
        Some(&student),
        Some(json!({ "rating": "满意", "comment": "处理及时" })),
    )
    .await?;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["rating"], "满意");
    let (status, _) = request(
        &app,
        Method::POST,
        &format!("/api/v1/my/issues/{issue_id}/rating"),
        Some(&student),
        Some(json!({ "rating": "一般" })),
    )
    .await?;
    assert_eq!(status, StatusCode::CONFLICT);

    // 9. 操作日志已记录
    let (status, logs) = request(
        &app,
        Method::GET,
        "/api/v1/admin/audit-logs",
        Some(&admin),
        None,
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert!(logs.as_array().unwrap().len() >= 3);

    // 10. 学生收到的通知
    let (status, notifs) = request(
        &app,
        Method::GET,
        "/api/v1/notifications",
        Some(&student),
        None,
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert!(!notifs.as_array().unwrap().is_empty());

    Ok(())
}

#[tokio::test]
async fn auth_and_permissions() -> anyhow::Result<()> {
    let app = setup().await?;
    register_admin(&app).await?;

    // 错误密码
    let (status, _) = request(
        &app,
        Method::POST,
        "/api/v1/auth/login",
        None,
        Some(json!({ "username": "admin", "password": "wrong" })),
    )
    .await?;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // 未登录访问受保护接口
    let (status, _) = request(&app, Method::GET, "/api/v1/my/issues", None, None).await?;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // 学生不能访问管理接口
    let student = student_token(&app).await?;
    let (status, _) = request(
        &app,
        Method::GET,
        "/api/v1/admin/issues",
        Some(&student),
        None,
    )
    .await?;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // 学生提交后：未办结不可评价
    let (status, body) = request(
        &app,
        Method::POST,
        "/api/v1/issues",
        Some(&student),
        Some(json!({
            "kind": "问题咨询",
            "category": "网络",
            "campus": "松江校区",
            "location": "教学区域",
            "title": "校园网接入咨询",
            "description": "想了解临时访客设备如何申请接入校园网络。",
            "public_will": false,
            "contact": ""
        })),
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    let issue_id = body["id"].as_str().unwrap();
    let (status, _) = request(
        &app,
        Method::POST,
        &format!("/api/v1/my/issues/{issue_id}/rating"),
        Some(&student),
        Some(json!({ "rating": "满意" })),
    )
    .await?;
    assert_eq!(status, StatusCode::CONFLICT);

    // 参数校验：描述太短
    let (status, _) = request(
        &app,
        Method::POST,
        "/api/v1/issues",
        Some(&student),
        Some(json!({
            "kind": "问题咨询",
            "category": "网络",
            "campus": "松江校区",
            "location": "教学区域",
            "title": "标题",
            "description": "太短",
            "public_will": false
        })),
    )
    .await?;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    Ok(())
}
