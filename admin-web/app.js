(function () {
  "use strict";

  const STATUS = {
    "待受理": { className: "status-pending", hint: "等待权益工作小组查看与分派" },
    "待补充": { className: "status-supplement", hint: "等待学生补充必要信息" },
    "已受理": { className: "status-accepted", hint: "已进入校内协同处理流程" },
    "部门处理中": { className: "status-processing", hint: "职能部门正在核查与办理" },
    "待审核": { className: "status-review", hint: "部门已回复，等待权益组审核" },
    "已办结": { className: "status-done", hint: "办理结果已反馈学生" },
    "已关闭": { className: "status-closed", hint: "诉求已关闭并保留记录" }
  };

  const STATUS_TRANSITIONS = {
    "待受理": ["已受理", "待补充", "已关闭"],
    "待补充": ["已受理", "待补充", "已关闭"],
    "已受理": ["部门处理中", "待补充", "已关闭"],
    "部门处理中": ["待审核", "待补充", "已关闭"],
    "待审核": ["已办结", "部门处理中", "已关闭"],
    "已办结": [],
    "已关闭": []
  };

  const TERMINAL_STATUSES = ["已办结", "已关闭"];

  const VIEW_TITLES = {
    dashboard: "数据看板",
    intake: "诉求受理",
    workflow: "办理流转",
    review: "公开审核",
    logs: "操作日志"
  };

  const issues = [
    {
      id: "CV202607240018",
      title: "东区食堂晚餐高峰排队时间过长",
      type: "意见建议",
      category: "食堂",
      priority: "高",
      status: "待受理",
      campus: "松江校区",
      location: "东区第二食堂一层",
      submitted: "2026-07-24 09:42",
      department: "后勤实业发展中心",
      assignee: "待分配",
      deadline: "剩余 23 小时",
      student: "陈同学",
      contact: "138****2619",
      publicWill: "同意脱敏公开",
      description: "最近一周晚餐高峰期只有两个窗口开放，18:00 左右通常需要排队二十多分钟。希望协调增加开放窗口，并在现场标注各窗口供应品类。",
      attachments: ["排队现场.jpg", "窗口情况.jpg"],
      finalReply: "",
      timeline: [
        { status: "学生提交", time: "2026-07-24 09:42", operator: "陈同学", note: "提交诉求并勾选隐私与诉求处理说明。" }
      ],
      review: {
        state: "待脱敏",
        publicTitle: "食堂晚餐高峰排队时间较长",
        publicLocation: "松江校区食堂区域",
        publicSummary: "有同学反映晚餐高峰开放窗口较少、排队时间较长，希望优化窗口安排。",
        publicReply: "后勤部门已根据就餐客流调整窗口开放时间，并安排现场人员动态增开窗口。"
      }
    },
    {
      id: "CV202607230015",
      title: "教学楼 B 区三楼饮水机持续漏水",
      type: "设施报修建议",
      category: "教学",
      priority: "高",
      status: "部门处理中",
      campus: "松江校区",
      location: "教学楼 B 区 3F 东侧",
      submitted: "2026-07-23 16:18",
      department: "基建处",
      assignee: "周明远",
      deadline: "剩余 8 小时",
      student: "王同学",
      contact: "156****9031",
      publicWill: "同意脱敏公开",
      description: "B 区三楼东侧饮水机底部从昨晚开始持续漏水，地面湿滑。本人是机械工程学院 220318 班王同学，联系电话 15612989031，请尽快安排维修。",
      attachments: ["饮水机漏水.jpg"],
      finalReply: "",
      timeline: [
        { status: "部门处理中", time: "2026-07-24 08:55", operator: "周明远 · 基建处", note: "维修人员已到场排查，初步判断为进水阀老化，配件将于今日下午更换。" },
        { status: "转交相关部门", time: "2026-07-23 17:03", operator: "林知夏 · 权益工作小组", note: "已指派基建处处理，并提醒现场放置防滑警示牌。" },
        { status: "权益组受理", time: "2026-07-23 16:44", operator: "林知夏 · 权益工作小组", note: "确认属于校园设施维护受理范围，优先级设为高。" },
        { status: "学生提交", time: "2026-07-23 16:18", operator: "王同学", note: "提交诉求及 1 张现场图片。" }
      ],
      review: {
        state: "待脱敏",
        publicTitle: "教学楼公共饮水机漏水问题",
        publicLocation: "松江校区教学楼公共区域",
        publicSummary: "有同学反映教学楼公共饮水机出现漏水，存在地面湿滑隐患，希望尽快检修。",
        publicReply: "基建处已更换老化进水阀，并完成设备试运行和现场清理。"
      }
    },
    {
      id: "CV202607230011",
      title: "图书馆研讨室预约后被他人占用",
      type: "投诉反馈",
      category: "学生服务",
      priority: "中",
      status: "待审核",
      campus: "松江校区",
      location: "图书馆 406 研讨室",
      submitted: "2026-07-23 10:26",
      department: "图书馆",
      assignee: "赵晓静",
      deadline: "剩余 1 天",
      student: "刘同学",
      contact: "微信账号关联",
      publicWill: "同意脱敏公开",
      description: "通过预约系统预订了 406 研讨室，按时到达后发现被其他同学占用。现场缺少预约状态展示，希望完善核验机制。",
      attachments: ["预约记录.jpg"],
      finalReply: "图书馆已为研讨室增加预约状态提示牌，并安排值班老师在高峰时段加强巡查。后续将评估门禁与预约系统联动。",
      timeline: [
        { status: "部门回复", time: "2026-07-24 09:12", operator: "赵晓静 · 图书馆", note: "已提交正式回复并申请办结，等待权益工作小组审核。" },
        { status: "部门处理中", time: "2026-07-23 15:30", operator: "赵晓静 · 图书馆", note: "已核对预约记录，现场工作人员与占用同学完成沟通。" },
        { status: "转交相关部门", time: "2026-07-23 11:05", operator: "林知夏 · 权益工作小组", note: "转交图书馆读者服务部处理。" },
        { status: "学生提交", time: "2026-07-23 10:26", operator: "刘同学", note: "提交问题与预约记录截图。" }
      ],
      review: {
        state: "待审核",
        publicTitle: "图书馆研讨室预约使用秩序问题",
        publicLocation: "松江校区图书馆",
        publicSummary: "有同学反映预约研讨室后遇到被占用的情况，希望加强预约状态提示与现场核验。",
        publicReply: "图书馆已增加预约状态提示，并安排工作人员在高峰时段加强巡查；后续将评估预约系统与门禁联动方案。"
      }
    },
    {
      id: "CV202607220009",
      title: "宿舍区夜间共享单车堵塞消防通道",
      type: "投诉反馈",
      category: "交通",
      priority: "高",
      status: "待补充",
      campus: "长宁校区",
      location: "学生公寓区域",
      submitted: "2026-07-22 21:10",
      department: "保卫处",
      assignee: "顾诚",
      deadline: "等待学生补充",
      student: "匿名同学",
      contact: "微信账号关联",
      publicWill: "不公开",
      description: "宿舍区晚上有很多共享单车堆在消防通道附近，但没有提供具体楼栋或现场照片。",
      attachments: [],
      finalReply: "",
      timeline: [
        { status: "待补充", time: "2026-07-23 09:05", operator: "林知夏 · 权益工作小组", note: "请补充具体楼栋附近位置或现场照片，便于保卫处快速定位。" },
        { status: "学生提交", time: "2026-07-22 21:10", operator: "匿名同学", note: "提交共享单车停放问题。" }
      ],
      review: {
        state: "不公开",
        publicTitle: "",
        publicLocation: "",
        publicSummary: "",
        publicReply: ""
      }
    },
    {
      id: "CV202607210006",
      title: "校园网晚间频繁断线影响线上学习",
      type: "问题咨询",
      category: "网络",
      priority: "中",
      status: "已办结",
      campus: "松江校区",
      location: "学生公寓 12-14 号楼区域",
      submitted: "2026-07-21 19:33",
      department: "信息化办公室",
      assignee: "孙启航",
      deadline: "已按时办结",
      student: "多名同学联名",
      contact: "微信账号关联",
      publicWill: "同意脱敏公开",
      description: "连续三天晚上 20:00 后校园网出现短时断线，影响在线课程和资料下载。",
      attachments: ["网络诊断截图.jpg"],
      finalReply: "信息化办公室已调整相关楼宇无线接入点负载，并更换一台异常交换设备。连续监测 24 小时后网络运行稳定。",
      timeline: [
        { status: "反馈学生并办结", time: "2026-07-23 14:20", operator: "林知夏 · 权益工作小组", note: "正式回复已通过审核并反馈学生，诉求办结。" },
        { status: "权益组审核", time: "2026-07-23 13:45", operator: "林知夏 · 权益工作小组", note: "核验部门处理记录与监测结果，审核通过。" },
        { status: "部门回复", time: "2026-07-23 11:30", operator: "孙启航 · 信息化办公室", note: "完成网络设备调整并提交正式回复。" },
        { status: "部门处理中", time: "2026-07-22 09:10", operator: "孙启航 · 信息化办公室", note: "已开始调取相关时段接入点日志。" },
        { status: "学生提交", time: "2026-07-21 19:33", operator: "学生", note: "提交网络稳定性问题及诊断截图。" }
      ],
      review: {
        state: "已通过",
        publicTitle: "学生公寓区域晚间网络不稳定",
        publicLocation: "松江校区学生公寓区域",
        publicSummary: "有同学反映部分学生公寓晚间校园网频繁短时断线，影响线上学习。",
        publicReply: "信息化办公室已优化无线接入点负载并更换异常设备，经连续监测，当前网络运行稳定。"
      }
    },
    {
      id: "CV202607200004",
      title: "校医院周末能否增加值班时段",
      type: "意见建议",
      category: "医疗健康",
      priority: "低",
      status: "已受理",
      campus: "松江校区",
      location: "校医院",
      submitted: "2026-07-20 14:08",
      department: "校医院",
      assignee: "沈医生",
      deadline: "剩余 2 天",
      student: "张同学",
      contact: "139****2206",
      publicWill: "默认不公开",
      description: "希望了解周末值班安排，并建议考试周适当延长常规门诊服务时间。",
      attachments: [],
      finalReply: "",
      timeline: [
        { status: "已受理", time: "2026-07-21 08:52", operator: "林知夏 · 权益工作小组", note: "已受理并转校医院评估考试周门诊安排。涉及医疗事项，默认不公开。" },
        { status: "学生提交", time: "2026-07-20 14:08", operator: "张同学", note: "提交咨询与建议。" }
      ],
      review: { state: "不公开", publicTitle: "", publicLocation: "", publicSummary: "", publicReply: "" }
    },
    {
      id: "CV202607190003",
      title: "北门公交站雨棚遮挡范围不足",
      type: "意见建议",
      category: "交通",
      priority: "低",
      status: "待审核",
      campus: "松江校区",
      location: "北门外公交站",
      submitted: "2026-07-19 12:16",
      department: "保卫处",
      assignee: "顾诚",
      deadline: "剩余 19 小时",
      student: "吴同学",
      contact: "微信账号关联",
      publicWill: "同意脱敏公开",
      description: "雨天等候公交的同学较多，现有雨棚遮挡范围不足，希望与相关单位沟通改善。",
      attachments: ["公交站现场.jpg"],
      finalReply: "保卫处已与属地公交管理单位沟通，短期将增设临时引导区域，雨棚改造建议已纳入下一轮设施协调事项。",
      timeline: [
        { status: "部门回复", time: "2026-07-23 16:05", operator: "顾诚 · 保卫处", note: "已提交对外协调情况和阶段性解决方案，申请权益组审核。" },
        { status: "部门处理中", time: "2026-07-21 10:30", operator: "顾诚 · 保卫处", note: "已联系属地公交管理单位现场踏勘。" },
        { status: "学生提交", time: "2026-07-19 12:16", operator: "吴同学", note: "提交公交站雨棚改进建议。" }
      ],
      review: {
        state: "待审核",
        publicTitle: "校园北门公交站雨棚遮挡范围不足",
        publicLocation: "松江校区北门周边",
        publicSummary: "有同学建议改善校园北门周边公交站候车区域的雨天遮挡条件。",
        publicReply: "学校已与属地公交管理单位沟通，短期增设临时候车引导区域，雨棚改造建议将纳入后续设施协调。"
      }
    },
    {
      id: "CV202607180001",
      title: "重复提交：食堂餐具消毒情况咨询",
      type: "问题咨询",
      category: "食堂",
      priority: "低",
      status: "已关闭",
      campus: "松江校区",
      location: "西区食堂",
      submitted: "2026-07-18 08:35",
      department: "后勤实业发展中心",
      assignee: "何雨",
      deadline: "已关闭",
      student: "匿名同学",
      contact: "微信账号关联",
      publicWill: "不公开",
      description: "与 CV202607170026 内容重复，已合并至原诉求继续跟进。",
      attachments: [],
      finalReply: "",
      timeline: [
        { status: "已关闭", time: "2026-07-18 09:20", operator: "林知夏 · 权益工作小组", note: "经核查与 CV202607170026 重复，已合并处理并向学生说明。" },
        { status: "学生提交", time: "2026-07-18 08:35", operator: "匿名同学", note: "提交餐具消毒情况咨询。" }
      ],
      review: { state: "不公开", publicTitle: "", publicLocation: "", publicSummary: "", publicReply: "" }
    }
  ];

  const auditLogs = [
    { time: "2026-07-24 10:18:32", operator: "林知夏", role: "权益工作小组", action: "审核公开", target: "CV202607210006", before: "待审核", after: "已通过", detail: "确认脱敏稿不含个人信息并公开。", ip: "10.24.8.31" },
    { time: "2026-07-24 09:12:06", operator: "赵晓静", role: "图书馆", action: "提交回复", target: "CV202607230011", before: "部门处理中", after: "待审核", detail: "提交正式回复与办结申请。", ip: "10.24.31.16" },
    { time: "2026-07-24 08:55:41", operator: "周明远", role: "基建处", action: "更新进度", target: "CV202607230015", before: "部门处理中", after: "部门处理中", detail: "记录现场排查结果及维修安排。", ip: "10.24.18.42" },
    { time: "2026-07-23 17:03:17", operator: "林知夏", role: "权益工作小组", action: "指派部门", target: "CV202607230015", before: "已受理", after: "部门处理中", detail: "指派至基建处，负责人周明远。", ip: "10.24.8.31" },
    { time: "2026-07-23 16:44:29", operator: "林知夏", role: "权益工作小组", action: "受理诉求", target: "CV202607230015", before: "待受理", after: "已受理", detail: "设置分类为教学、优先级为高。", ip: "10.24.8.31" },
    { time: "2026-07-23 09:05:12", operator: "林知夏", role: "权益工作小组", action: "请求补充", target: "CV202607220009", before: "待受理", after: "待补充", detail: "请求补充具体位置或现场图片。", ip: "10.24.8.31" },
    { time: "2026-07-18 09:20:05", operator: "林知夏", role: "权益工作小组", action: "关闭诉求", target: "CV202607180001", before: "待受理", after: "已关闭", detail: "与既有诉求重复，完成合并。", ip: "10.24.8.31" }
  ];

  const state = {
    view: "dashboard",
    selectedIssueId: "CV202607230015",
    selectedReviewId: "CV202607230011",
    intakeSearch: "",
    intakeStatus: "全部状态",
    intakeCategory: "全部分类",
    logSearch: "",
    batchSelected: [],
    modalReturnFocus: null
  };

  const mainContent = document.getElementById("mainContent");
  const viewTitle = document.getElementById("viewTitle");
  const mainNav = document.getElementById("mainNav");
  const statusModalBackdrop = document.getElementById("modalBackdrop");
  const statusForm = document.getElementById("statusForm");
  const statusNote = document.getElementById("statusNote");
  const nextStatus = document.getElementById("nextStatus");
  const batchModalBackdrop = document.getElementById("batchModalBackdrop");
  const batchForm = document.getElementById("batchForm");

  function escapeHtml(value) {
    return String(value ?? "")
      .replaceAll("&", "&amp;")
      .replaceAll("<", "&lt;")
      .replaceAll(">", "&gt;")
      .replaceAll('"', "&quot;")
      .replaceAll("'", "&#039;");
  }

  const ICON_PATHS = {
    home: '<path d="M3 11.5 12 4l9 7.5M5 10v10h5v-6h4v6h5V10"/>',
    inbox: '<rect x="3" y="6" width="18" height="12" rx="2"/><path d="M3 10h5l2 3h4l2-3h5"/>',
    flow: '<path d="M8 7H6a2 2 0 0 0-2 2v6a2 2 0 0 0 2 2h2M16 7h2a2 2 0 0 1 2 2v6a2 2 0 0 1-2 2h-2"/><path d="M8 12h8M8 15h8"/>',
    review: '<circle cx="12" cy="12" r="8"/><path d="m9 12 2 2 4-4"/>',
    list: '<path d="M8 6h13M8 12h13M8 18h13"/><path d="M3.5 6h.01M3.5 12h.01M3.5 18h.01"/>',
    menu: '<path d="M4 7h16M4 12h16M4 17h16"/>',
    bell: '<path d="M6 9a6 6 0 0 1 12 0c0 5 2 6 2 6H4s2-1 2-6"/><path d="M10 19a2 2 0 0 0 4 0"/>',
    chevronDown: '<path d="m6 9 6 6 6-6"/>',
    refresh: '<path d="M4 12a8 8 0 0 1 14-5l2 2M20 12a8 8 0 0 1-14 5l-2-2M18 4v5h-5M6 20v-5h5"/>',
    download: '<path d="M12 4v11m0 0 4-4m-4 4-4-4"/><path d="M4 19h16"/>',
    grid: '<rect x="3" y="3" width="7" height="7" rx="1"/><rect x="14" y="3" width="7" height="7" rx="1"/><rect x="3" y="14" width="7" height="7" rx="1"/><rect x="14" y="14" width="7" height="7" rx="1"/>',
    check: '<path d="m5 13 4 4L19 7"/>',
    alert: '<path d="M12 4 2.5 20h19L12 4Z"/><path d="M12 10v4M12 17.5v.5"/>',
    clock: '<circle cx="12" cy="12" r="8"/><path d="M12 8v4l3 2"/>',
    tally: '<path d="M5 4v16M9 4v16M13 4v16M17 4v16M21 4v16"/>',
    search: '<circle cx="11" cy="11" r="6"/><path d="m16 16 4 4"/>',
    image: '<rect x="3" y="4" width="18" height="16" rx="2"/><circle cx="9" cy="10" r="2"/><path d="m5 20 5-5 3 3 3-3 3 3"/>'
  };

  function icon(name) {
    const paths = ICON_PATHS[name] || "";
    return `<svg class="icon" width="1em" height="1em" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">${paths}</svg>`;
  }

  function statusBadge(status) {
    const config = STATUS[status] || { className: "status-closed" };
    return `<span class="status-badge ${config.className}">${escapeHtml(status)}</span>`;
  }

  function priorityBadge(priority) {
    const key = priority === "高" ? "high" : priority === "中" ? "medium" : "low";
    return `<span class="priority-badge priority-${key}">${escapeHtml(priority)}优先级</span>`;
  }

  function nowString() {
    return new Intl.DateTimeFormat("zh-CN", {
      year: "numeric", month: "2-digit", day: "2-digit", hour: "2-digit", minute: "2-digit", second: "2-digit", hour12: false
    }).format(new Date()).replaceAll("/", "-");
  }

  function getIssue(id = state.selectedIssueId) {
    return issues.find((issue) => issue.id === id) || issues[0];
  }

  function updateBadges() {
    document.getElementById("pendingBadge").textContent = issues.filter((issue) => issue.status === "待受理").length;
    document.getElementById("reviewBadge").textContent = issues.filter((issue) => issue.review.state === "待审核").length;
  }

  function showToast(title, message) {
    const toast = document.createElement("div");
    toast.className = "toast";
    toast.setAttribute("role", "status");
    toast.innerHTML = `<span class="toast-icon" aria-hidden="true">${icon("check")}</span><div><strong>${escapeHtml(title)}</strong><span>${escapeHtml(message)}</span></div>`;
    document.getElementById("toastRegion").appendChild(toast);
    window.setTimeout(() => toast.remove(), 3600);
  }

  function navigate(view) {
    if (!VIEW_TITLES[view]) return;
    state.view = view;
    viewTitle.textContent = VIEW_TITLES[view];
    document.querySelectorAll(".nav-item").forEach((item) => {
      const isActive = item.dataset.view === view;
      item.classList.toggle("active", isActive);
      if (isActive) item.setAttribute("aria-current", "page");
      else item.removeAttribute("aria-current");
    });
    closeSidebar();
    render();
    mainContent.focus({ preventScroll: true });
    window.scrollTo({ top: 0, behavior: "smooth" });
  }

  function render() {
    updateBadges();
    if (state.view === "dashboard") renderDashboard();
    if (state.view === "intake") renderIntake();
    if (state.view === "workflow") renderWorkflow();
    if (state.view === "review") renderReview();
    if (state.view === "logs") renderLogs();
  }

  function renderDashboard() {
    const recent = issues.slice(0, 5);
    const categoryData = [
      ["志愿服务", 8], ["校历", 6], ["教务", 4], ["共青团员", 4], ["第二课堂", 4], ["信息问询", 3], ["其他", 9]
    ];
    mainContent.innerHTML = `
      <div class="page-header">
        <div><h2>权益服务运行概览</h2><p>历史数据周期：2026-06-12 至 2026-07-16 · 来自“程园小二”交接周报</p></div>
        <div class="page-actions">
          <button class="button secondary" type="button" data-action="refresh">${icon("refresh")} 刷新数据</button>
          <button class="button primary" type="button" data-action="export-report">${icon("download")} 导出历史摘要</button>
        </div>
      </div>
      <section class="metric-grid" aria-label="关键指标">
        ${metricCard("累计问询", "45", "四期周报汇总", "", "条", "grid")}
        ${metricCard("咨询类", "38", "均已逐一回复", "trend-up", "条", "check")}
        ${metricCard("意见投诉", "7", "需进入流转闭环", "", "条", "alert")}
        ${metricCard("最近一期", "9", "7 月 10 日至 16 日", "", "条", "clock")}
        ${metricCard("无效信息", "2", "按周报数据口径", "", "条", "tally")}
      </section>
      <section class="dashboard-grid">
        <article class="panel">
          <header class="panel-header"><div><h3>历史咨询关键词分布</h3><p>按四期周报热点关键词归并</p></div><span class="category-badge">咨询类 38 条</span></header>
          <div class="panel-body chart-wrap">
            <div class="bar-chart" role="img" aria-label="志愿服务 8 条，校历 6 条，教务 4 条，共青团员 4 条，第二课堂 4 条，信息问询 3 条，其他 9 条">
              ${categoryData.map(([label, value]) => `<div class="bar-item"><span class="bar-value">${value}</span><span class="bar" style="height:${value * 18}px"></span><span class="bar-label">${label}</span></div>`).join("")}
            </div>
            <div class="chart-legend">
              ${categoryData.slice().sort((a,b) => b[1]-a[1]).map(([label,value], index) => `<div class="legend-item"><span class="legend-dot" style="background:${index === 0 ? "#176B5B" : index === 1 ? "#D84A3A" : "#93AAA4"}"></span><span>${label}</span><strong>${value}</strong></div>`).join("")}
            </div>
          </div>
        </article>
        <article class="panel">
          <header class="panel-header"><div><h3>周度咨询回复量</h3><p>周报已回复咨询数量</p></div><span class="category-badge">合计 38 条</span></header>
          <div class="panel-body department-list">
            ${progressRow("06/12–06/25", 12, 12)}
            ${progressRow("06/26–07/02", 10, 12)}
            ${progressRow("07/03–07/09", 7, 12)}
            ${progressRow("07/10–07/16", 9, 12)}
          </div>
        </article>
      </section>
      <section class="dashboard-bottom">
        <article class="panel">
          <header class="panel-header"><div><h3>数据口径说明</h3><p>真实历史汇总与流程 Demo 分开呈现</p></div></header>
          <div class="panel-body source-note">
            <strong>交接材料真实汇总</strong>
            <p>顶部指标、关键词分布与周度趋势来自交接目录中的四期周报及周报数据文档。</p>
            <strong>新系统流程演示</strong>
            <p>右侧最新诉求、受理流转与公开审核为脱敏模拟数据，用于展示未来系统能力，不代表历史真实工单。</p>
          </div>
        </article>
        <article class="panel table-panel">
          <header class="panel-header"><div><h3>流程演示数据</h3><p>脱敏模拟工单 · 非历史真实记录</p></div><button class="link-button" type="button" data-view-link="intake">进入受理台</button></header>
          <div class="table-scroll">
            <table><thead><tr><th>诉求</th><th>分类</th><th>状态</th><th>提交时间</th></tr></thead>
              <tbody>${recent.map(simpleIssueRow).join("")}</tbody>
            </table>
          </div>
        </article>
      </section>`;
  }

  function metricCard(label, value, foot, trendClass, unit, icon) {
    return `<article class="metric-card"><div class="metric-top"><span>${label}</span><span class="metric-icon" aria-hidden="true">${icon}</span></div><div class="metric-value">${value}<small style="font-size:12px;font-weight:600;margin-left:4px">${unit}</small></div><div class="metric-foot ${trendClass}">${foot}</div></article>`;
  }

  function progressRow(name, value, max) {
    return `<div class="progress-row"><div class="progress-meta"><span>${name}</span><strong>${value} 件</strong></div><div class="progress-track"><div class="progress-value" style="width:${value / max * 100}%"></div></div></div>`;
  }

  function simpleIssueRow(issue) {
    return `<tr><td class="issue-cell"><button type="button" data-open-issue="${issue.id}">${escapeHtml(issue.title)}</button><small>${issue.id}</small></td><td><span class="category-badge">${escapeHtml(issue.category)}</span></td><td>${statusBadge(issue.status)}</td><td class="id-text">${escapeHtml(issue.submitted.slice(5))}</td></tr>`;
  }

  function renderIntake() {
    const categories = [...new Set(issues.map((issue) => issue.category))];
    mainContent.innerHTML = `
      <div class="page-header">
        <div><h2>诉求受理台</h2><p>流程演示数据：集中完成范围判断、分类、优先级设置与部门指派</p></div>
        <div class="page-actions"><button class="button secondary" type="button" data-action="batch" id="batchButton">批量指派</button><button class="button primary" type="button" data-action="new-intake">＋ 登记线下诉求</button></div>
      </div>
      <section class="toolbar" aria-label="诉求筛选">
        <label class="search-field"><span aria-hidden="true">${icon("search")}</span><input id="intakeSearch" type="search" value="${escapeHtml(state.intakeSearch)}" placeholder="搜索编号、标题、地点或部门" aria-label="搜索诉求"></label>
        <label class="filter-field"><span>处理状态</span><select id="intakeStatus">${["全部状态", ...Object.keys(STATUS)].map((value) => `<option${value === state.intakeStatus ? " selected" : ""}>${value}</option>`).join("")}</select></label>
        <label class="filter-field"><span>诉求分类</span><select id="intakeCategory">${["全部分类", ...categories].map((value) => `<option${value === state.intakeCategory ? " selected" : ""}>${value}</option>`).join("")}</select></label>
        <button class="button secondary" type="button" data-action="reset-filters">重置</button>
        <span class="result-count" id="resultCount"></span>
      </section>
      <section class="panel table-panel">
        <div class="table-scroll">
          <table>
            <thead><tr><th><span class="checkbox-row"><input id="selectAll" type="checkbox" aria-label="选择全部诉求"></span></th><th>诉求编号</th><th>诉求内容</th><th>分类</th><th>状态 / 优先级</th><th>负责部门</th><th>提交时间</th><th>操作</th></tr></thead>
            <tbody id="intakeTableBody"></tbody>
          </table>
        </div>
        <div class="pagination"><span id="paginationCopy"></span><div class="page-actions"><button class="button small secondary" type="button" disabled>上一页</button><button class="button small primary" type="button">1</button><button class="button small secondary" type="button" disabled>下一页</button></div></div>
      </section>`;
    updateIntakeTable();
  }

  function filteredIssues() {
    const keyword = state.intakeSearch.trim().toLowerCase();
    return issues.filter((issue) => {
      const searchable = `${issue.id} ${issue.title} ${issue.location} ${issue.department}`.toLowerCase();
      return (!keyword || searchable.includes(keyword)) &&
        (state.intakeStatus === "全部状态" || issue.status === state.intakeStatus) &&
        (state.intakeCategory === "全部分类" || issue.category === state.intakeCategory);
    });
  }

  function updateIntakeTable() {
    const body = document.getElementById("intakeTableBody");
    if (!body) return;
    const filtered = filteredIssues();
    body.innerHTML = filtered.length ? filtered.map((issue) => `
      <tr>
        <td><input type="checkbox" class="row-check" data-id="${issue.id}" aria-label="选择 ${issue.id}"${state.batchSelected.includes(issue.id) ? " checked" : ""}></td>
        <td class="id-text">${issue.id}</td>
        <td class="issue-cell"><button type="button" data-open-issue="${issue.id}">${escapeHtml(issue.title)}</button><small>${escapeHtml(issue.campus)} · ${escapeHtml(issue.location)}</small></td>
        <td><span class="category-badge">${escapeHtml(issue.category)}</span></td>
        <td>${statusBadge(issue.status)}<span class="cell-sub">${priorityBadge(issue.priority)}</span></td>
        <td>${escapeHtml(issue.department)}<span class="cell-sub">${escapeHtml(issue.assignee)}</span></td>
        <td class="id-text">${escapeHtml(issue.submitted.slice(5))}</td>
        <td><button class="button small secondary" type="button" data-open-issue="${issue.id}">查看办理</button></td>
      </tr>`).join("") : `<tr><td colspan="8"><div class="empty-state"><strong>没有匹配的诉求</strong>请调整关键词或筛选条件后重试</div></td></tr>`;
    document.getElementById("resultCount").textContent = `筛选结果 ${filtered.length} 件`;
    document.getElementById("paginationCopy").textContent = `共 ${filtered.length} 条记录，第 1 / 1 页`;
    const selectAll = document.getElementById("selectAll");
    if (selectAll) selectAll.checked = filtered.length > 0 && filtered.every((issue) => state.batchSelected.includes(issue.id));
    const batchButton = document.getElementById("batchButton");
    if (batchButton) batchButton.textContent = state.batchSelected.length ? `批量指派（${state.batchSelected.length}）` : "批量指派";
  }

  function renderWorkflow() {
    const issue = getIssue();
    mainContent.innerHTML = `
      <div class="page-header">
        <div><button class="link-button" type="button" data-view-link="intake">← 返回诉求受理台</button></div>
        <div class="page-actions"><button class="button secondary" type="button" data-action="copy-id">复制诉求编号</button><button class="button primary" type="button" data-action="open-status"${TERMINAL_STATUSES.includes(issue.status) ? " disabled" : ""}>更新办理状态</button></div>
      </div>
      <section class="workflow-layout">
        <div class="workflow-main">
          <article class="panel">
            <div class="issue-heading">
              <div><span class="issue-number">诉求编号 ${issue.id}</span><h2>${escapeHtml(issue.title)}</h2><p>${escapeHtml(issue.type)} · 提交于 ${escapeHtml(issue.submitted)}</p></div>
              <div class="heading-badges">${statusBadge(issue.status)}${priorityBadge(issue.priority)}</div>
            </div>
            <div class="issue-meta-grid">
              ${metaItem("诉求分类", issue.category)}${metaItem("校区与地点", `${issue.campus} · ${issue.location}`)}${metaItem("负责部门", issue.department)}${metaItem("当前负责人", issue.assignee)}
              ${metaItem("学生", issue.student)}${metaItem("联系方式", issue.contact)}${metaItem("公开意愿", issue.publicWill)}${metaItem("办理时限", issue.deadline)}
            </div>
          </article>
          <article class="panel">
            <div class="content-block"><h3>诉求详细描述</h3><p>${escapeHtml(issue.description)}</p>${attachmentMarkup(issue.attachments)}</div>
            ${issue.finalReply ? `<div class="content-block content-divider"><h3>部门正式回复</h3><p>${escapeHtml(issue.finalReply)}</p></div>` : ""}
          </article>
          <article class="panel">
            <header class="panel-header"><div><h3>完整办理时间线</h3><p>每次状态变化均记录操作人、时间和处理说明</p></div><span class="category-badge">${issue.timeline.length} 条记录</span></header>
            <div class="timeline">${issue.timeline.map(timelineItem).join("")}</div>
          </article>
        </div>
        <aside class="panel workbench" aria-label="办理工作台">
          <div class="workbench-section"><h3>当前办理状态</h3><div class="current-status"><strong>${escapeHtml(issue.status)}</strong><span>${escapeHtml((STATUS[issue.status] || {}).hint || "已记录当前状态")}</span></div><div class="sla-row"><span>办理时限</span><strong>${escapeHtml(issue.deadline)}</strong></div></div>
          <div class="workbench-section"><h3>流转分派</h3><div class="assign-grid"><label class="compact-field"><span>负责部门</span><select id="departmentSelect">${["后勤实业发展中心", "基建处", "信息化办公室", "保卫处", "图书馆", "校医院", "学生工作部"].map((value) => `<option${value === issue.department ? " selected" : ""}>${value}</option>`).join("")}</select></label><label class="compact-field"><span>负责人</span><select id="assigneeSelect">${[issue.assignee, "周明远", "赵晓静", "顾诚", "何雨", "孙启航"].filter((value,index,array) => array.indexOf(value) === index).map((value) => `<option>${escapeHtml(value)}</option>`).join("")}</select></label><button class="button secondary" type="button" data-action="save-assignment">保存分派</button></div></div>
          <div class="workbench-section"><h3>快捷操作</h3><div class="workbench-actions"><button class="button primary" type="button" data-action="open-status"${TERMINAL_STATUSES.includes(issue.status) ? " disabled" : ""}>更新办理状态</button><button class="button secondary" type="button" data-action="request-supplement"${TERMINAL_STATUSES.includes(issue.status) ? " disabled" : ""}>请求学生补充材料</button><button class="button secondary" type="button" data-action="go-review">进入公开审核</button><button class="button danger" type="button" data-action="close-issue"${TERMINAL_STATUSES.includes(issue.status) ? " disabled" : ""}>关闭诉求</button></div></div>
        </aside>
      </section>`;
  }

  function metaItem(label, value) {
    return `<div class="meta-item"><span>${label}</span><strong>${escapeHtml(value)}</strong></div>`;
  }

  function attachmentMarkup(attachments) {
    if (!attachments.length) return `<div class="attachment-row"><span class="cell-sub">学生未上传图片附件</span></div>`;
    return `<div class="attachment-row">${attachments.map((name) => `<div class="attachment"><span class="attachment-thumb" aria-hidden="true">${icon("image")}</span><span>${escapeHtml(name)}<small class="cell-sub">图片附件 · 仅内部可见</small></span></div>`).join("")}</div>`;
  }

  function timelineItem(item) {
    return `<div class="timeline-item"><span class="timeline-dot" aria-hidden="true"></span><div class="timeline-head"><strong>${escapeHtml(item.status)}</strong><time>${escapeHtml(item.time)}</time></div><p>${escapeHtml(item.note)}</p><span class="timeline-operator">操作人：${escapeHtml(item.operator)}</span></div>`;
  }

  function reviewCandidates() {
    return issues.filter((issue) => issue.review.state === "待审核" || issue.review.state === "已通过");
  }

  function renderReview() {
    const candidates = reviewCandidates();
    if (!candidates.some((issue) => issue.id === state.selectedReviewId)) state.selectedReviewId = candidates[0]?.id;
    const issue = issues.find((item) => item.id === state.selectedReviewId);
    if (!issue) {
      mainContent.innerHTML = `<div class="empty-state"><strong>暂无待审核公开稿</strong>部门回复审核完成后，符合公开条件的诉求会显示在这里。</div>`;
      return;
    }
    mainContent.innerHTML = `
      <div class="page-header"><div><h2>公开内容审核</h2><p>逐项核对原始诉求与脱敏公开稿，公开页面不得含个人敏感信息</p></div><div class="page-actions"><button class="button secondary" type="button" data-action="privacy-guide">查看脱敏规范</button></div></div>
      <section class="review-layout">
        <aside class="panel review-queue" aria-label="审核队列">
          <div class="queue-header"><h3>审核队列</h3><p>${candidates.filter((item) => item.review.state === "待审核").length} 条待审核</p></div>
          <div class="queue-list">${candidates.map((item) => `<button class="queue-item${item.id === issue.id ? " active" : ""}" type="button" data-review-issue="${item.id}"><span class="issue-number">${item.id}</span><strong>${escapeHtml(item.title)}</strong><small>${item.category} · ${item.review.state}</small></button>`).join("")}</div>
        </aside>
        <article class="panel review-workspace">
          <div class="review-top"><div><h2>${escapeHtml(issue.title)}</h2><p>${issue.id} · ${escapeHtml(issue.department)} · ${escapeHtml(issue.submitted)}</p></div>${issue.review.state === "已通过" ? '<span class="status-badge status-done">已通过并公开</span>' : '<span class="status-badge status-review">等待审核</span>'}</div>
          <div class="review-columns">
            <section class="review-column" aria-label="原始内容"><div class="column-heading"><h3>原始诉求 · 仅内部可见</h3><span class="priority-badge priority-high">含敏感信息</span></div><div class="privacy-alert">系统已标记姓名、班级、手机号和精确位置；审核人仍需人工复核全文。</div>
              <div class="review-field"><span>原始标题</span><p class="original-copy">${escapeHtml(issue.title)}</p></div>
              <div class="review-field"><span>原始地点</span><p class="original-copy"><mark class="sensitive">${escapeHtml(issue.location)}</mark></p></div>
              <div class="review-field"><span>诉求描述</span><p class="original-copy">${highlightSensitive(issue.description)}</p></div>
              <div class="review-field"><span>学生与联系方式</span><p class="original-copy"><mark class="sensitive">${escapeHtml(issue.student)}</mark> · <mark class="sensitive">${escapeHtml(issue.contact)}</mark></p></div>
              <div class="review-field"><span>部门正式回复</span><p class="original-copy">${escapeHtml(issue.finalReply || "部门正式回复尚未录入")}</p></div>
            </section>
            <section class="review-column" aria-label="脱敏公开稿">
              <div class="column-heading"><h3>脱敏公开稿 · 学生端预览</h3><span class="category-badge">可编辑</span></div>
              <label class="review-field"><span>公开标题</span><input id="publicTitle" value="${escapeHtml(issue.review.publicTitle)}" maxlength="60"></label>
              <label class="review-field"><span>模糊地点</span><input id="publicLocation" value="${escapeHtml(issue.review.publicLocation)}" maxlength="40"></label>
              <label class="review-field"><span>问题摘要</span><textarea class="review-textarea" id="publicSummary" maxlength="300">${escapeHtml(issue.review.publicSummary)}</textarea></label>
              <label class="review-field"><span>公开回复</span><textarea class="review-textarea" id="publicReply" maxlength="500">${escapeHtml(issue.review.publicReply)}</textarea></label>
              <label class="checkbox-row"><input id="privacyCheck" type="checkbox"><span>我已确认公开稿不含姓名、学号、手机号、精确宿舍号及原始图片</span></label>
            </section>
          </div>
          <footer class="review-footer"><p>通过后将模拟同步至小程序公开反馈，本 Demo 不上传任何数据。</p><div class="page-actions"><button class="button secondary" type="button" data-action="save-draft">保存草稿</button><button class="button primary" type="button" data-action="approve-review"${issue.review.state === "已通过" ? " disabled" : ""}>审核通过并公开</button></div></footer>
        </article>
      </section>`;
  }

  function highlightSensitive(value) {
    let safe = escapeHtml(value);
    const markers = ["220318 班王同学", "15612989031", "406 研讨室", "B 区 3F 东侧"];
    markers.forEach((marker) => {
      safe = safe.replaceAll(escapeHtml(marker), `<mark class="sensitive">${escapeHtml(marker)}</mark>`);
    });
    return safe;
  }

  function renderLogs() {
    const keyword = state.logSearch.trim().toLowerCase();
    const filtered = auditLogs.filter((log) => `${log.operator} ${log.action} ${log.target} ${log.detail}`.toLowerCase().includes(keyword));
    mainContent.innerHTML = `
      <div class="page-header"><div><h2>操作日志</h2><p>追踪管理端关键操作、状态变化与账号来源</p></div><div class="page-actions"><button class="button secondary" type="button" data-action="export-logs">${icon("download")} 导出日志</button></div></div>
      <div class="toolbar"><label class="search-field"><span aria-hidden="true">${icon("search")}</span><input id="logSearch" type="search" value="${escapeHtml(state.logSearch)}" placeholder="搜索操作人、诉求编号或动作" aria-label="搜索操作日志"></label><span class="result-count">共 ${filtered.length} 条操作记录</span></div>
      <section class="panel table-panel"><div class="table-scroll"><table><thead><tr><th>操作时间</th><th>操作人</th><th>操作类型</th><th>对象</th><th>修改记录 / 处理说明</th><th>来源 IP</th></tr></thead><tbody>
        ${filtered.length ? filtered.map((log) => `<tr><td class="id-text">${escapeHtml(log.time)}</td><td><strong>${escapeHtml(log.operator)}</strong><span class="cell-sub">${escapeHtml(log.role)}</span></td><td><span class="log-action">${escapeHtml(log.action)}</span></td><td><button class="link-button" type="button" data-open-issue="${log.target}">${escapeHtml(log.target)}</button></td><td class="change-cell"><span>${escapeHtml(log.before)}</span><span class="change-arrow">→</span><strong>${escapeHtml(log.after)}</strong><span class="cell-sub">${escapeHtml(log.detail)}</span></td><td class="id-text">${escapeHtml(log.ip)}</td></tr>`).join("") : `<tr><td colspan="6"><div class="empty-state"><strong>没有匹配的日志</strong>请尝试其他搜索内容</div></td></tr>`}
      </tbody></table></div><div class="pagination"><span>日志保留 180 天 · 当前显示本地演示数据</span><div class="page-actions"><button class="button small secondary" disabled>上一页</button><button class="button small primary">1</button><button class="button small secondary" disabled>下一页</button></div></div></section>`;
  }

  function openStatusModal(prefillStatus, prefillNote) {
    const issue = getIssue();
    const allowed = STATUS_TRANSITIONS[issue.status] || [];
    if (!allowed.length) {
      showToast("已处于终态", `${issue.status}状态不可继续流转。 `);
      return;
    }
    if (prefillStatus && !allowed.includes(prefillStatus)) {
      showToast("操作不允许", `当前状态为“${issue.status}”，不能切换至“${prefillStatus}”。 `);
      return;
    }
    state.modalReturnFocus = document.activeElement;
    document.getElementById("modalIssueSummary").innerHTML = `<strong>${escapeHtml(issue.title)}</strong><small>${issue.id} · 当前状态：${escapeHtml(issue.status)}</small>`;
    nextStatus.innerHTML = allowed.map((status) => `<option${status === (prefillStatus || issue.status) ? " selected" : ""}>${status}</option>`).join("");
    statusNote.value = prefillNote || "";
    document.getElementById("noteCount").textContent = statusNote.value.length;
    document.getElementById("statusNoteError").textContent = "";
    statusModalBackdrop.hidden = false;
    document.body.style.overflow = "hidden";
    window.setTimeout(() => nextStatus.focus(), 0);
  }

  function closeStatusModal() {
    statusModalBackdrop.hidden = true;
    document.body.style.overflow = "";
    if (state.modalReturnFocus && document.contains(state.modalReturnFocus)) state.modalReturnFocus.focus();
  }

  function openBatchModal() {
    const targets = issues.filter((issue) => state.batchSelected.includes(issue.id));
    if (!targets.length) {
      showToast("请先选择诉求", "在列表中勾选需要指派的诉求后重试。 ");
      return;
    }
    state.modalReturnFocus = document.activeElement;
    document.getElementById("batchIssueSummary").innerHTML = `<strong>已选择 ${targets.length} 条诉求</strong><small>${targets.map((issue) => issue.id).join("、")}</small>`;
    document.getElementById("batchDepartment").innerHTML = [...new Set(issues.map((issue) => issue.department))].map((value) => `<option>${escapeHtml(value)}</option>`).join("");
    document.getElementById("batchAssignee").innerHTML = [...new Set(issues.map((issue) => issue.assignee).filter((value) => value && value !== "待分配"))].map((value) => `<option>${escapeHtml(value)}</option>`).join("");
    batchModalBackdrop.hidden = false;
    document.body.style.overflow = "hidden";
    window.setTimeout(() => document.getElementById("batchDepartment").focus(), 0);
  }

  function closeBatchModal() {
    batchModalBackdrop.hidden = true;
    document.body.style.overflow = "";
    if (state.modalReturnFocus && document.contains(state.modalReturnFocus)) state.modalReturnFocus.focus();
  }

  function addLog(action, issue, before, after, detail) {
    auditLogs.unshift({ time: nowString(), operator: "林知夏", role: "权益工作小组", action, target: issue.id, before, after, detail, ip: "10.24.8.31" });
  }

  function saveReviewDraft() {
    const issue = issues.find((item) => item.id === state.selectedReviewId);
    if (!issue) return;
    issue.review.publicTitle = document.getElementById("publicTitle").value.trim();
    issue.review.publicLocation = document.getElementById("publicLocation").value.trim();
    issue.review.publicSummary = document.getElementById("publicSummary").value.trim();
    issue.review.publicReply = document.getElementById("publicReply").value.trim();
    showToast("草稿已保存", "脱敏公开稿已保存到本次演示会话。 ");
  }

  function approveReview() {
    const issue = issues.find((item) => item.id === state.selectedReviewId);
    if (!issue) return;
    saveReviewDraft();
    const required = [issue.review.publicTitle, issue.review.publicLocation, issue.review.publicSummary, issue.review.publicReply];
    if (required.some((value) => !value)) {
      showToast("审核未完成", "请先补全公开标题、模糊地点、问题摘要和公开回复。 ");
      return;
    }
    if (!document.getElementById("privacyCheck").checked) {
      showToast("请完成隐私确认", "确认脱敏检查项后才能审核通过。 ");
      document.getElementById("privacyCheck").focus();
      return;
    }
    const beforeReview = issue.review.state;
    const beforeStatus = issue.status;
    issue.review.state = "已通过";
    if (issue.status === "待审核") {
      issue.status = "已办结";
      issue.deadline = "已按时办结";
      issue.timeline.unshift({ status: "反馈学生并公开", time: nowString().slice(0, 16), operator: "林知夏 · 权益工作小组", note: "正式回复与脱敏公开稿审核通过，已模拟反馈学生并公开。" });
    }
    addLog("审核公开", issue, beforeReview, "已通过", `脱敏稿审核通过；诉求状态 ${beforeStatus} → ${issue.status}。`);
    render();
    showToast("审核已通过", `${issue.id} 已模拟公开，并同步完整办理时间线。`);
  }

  function saveAssignment() {
    const issue = getIssue();
    const before = `${issue.department} / ${issue.assignee}`;
    issue.department = document.getElementById("departmentSelect").value;
    issue.assignee = document.getElementById("assigneeSelect").value;
    const after = `${issue.department} / ${issue.assignee}`;
    if (before === after) {
      showToast("分派未变化", "当前部门与负责人保持不变。 ");
      return;
    }
    issue.timeline.unshift({ status: "更新分派", time: nowString().slice(0, 16), operator: "林知夏 · 权益工作小组", note: `负责部门与负责人已更新为 ${after}。` });
    addLog("指派部门", issue, before, after, "更新诉求负责部门与负责人。 ");
    render();
    showToast("分派已保存", `${issue.id} 已指派至 ${after}。`);
  }

  function exportCsv(kind) {
    const isLog = kind === "logs";
    const rows = isLog
      ? [["操作时间", "操作人", "角色", "动作", "诉求编号", "修改前", "修改后", "说明"], ...auditLogs.map((log) => [log.time, log.operator, log.role, log.action, log.target, log.before, log.after, log.detail])]
      : [
          ["数据周期", "问询总数", "咨询类", "意见投诉类", "已回复咨询", "未解决", "无效信息"],
          ["2026-06-12 至 2026-06-25", 14, 12, 2, 12, 2, 0],
          ["2026-06-26 至 2026-07-02", 11, 10, 1, 10, 0, 1],
          ["2026-07-03 至 2026-07-09", 11, 7, 4, 7, 4, 0],
          ["2026-07-10 至 2026-07-16", 9, 9, 0, 9, 0, 1]
        ];
    const csv = "\uFEFF" + rows.map((row) => row.map((value) => `"${String(value).replaceAll('"', '""')}"`).join(",")).join("\n");
    const url = URL.createObjectURL(new Blob([csv], { type: "text/csv;charset=utf-8" }));
    const link = document.createElement("a");
    link.href = url;
    link.download = isLog ? "程园小二-操作日志.csv" : "程园小二-历史周报摘要.csv";
    link.click();
    URL.revokeObjectURL(url);
    showToast("文件已生成", `${link.download} 已开始下载。`);
  }

  function openSidebar() {
    document.getElementById("sidebar").classList.add("open");
    document.getElementById("sidebarBackdrop").hidden = false;
    document.getElementById("menuButton").setAttribute("aria-expanded", "true");
  }

  function closeSidebar() {
    document.getElementById("sidebar").classList.remove("open");
    document.getElementById("sidebarBackdrop").hidden = true;
    document.getElementById("menuButton").setAttribute("aria-expanded", "false");
  }

  mainNav.addEventListener("click", (event) => {
    const item = event.target.closest("[data-view]");
    if (item) navigate(item.dataset.view);
  });

  mainContent.addEventListener("click", async (event) => {
    const viewLink = event.target.closest("[data-view-link]");
    if (viewLink) { navigate(viewLink.dataset.viewLink); return; }
    const issueButton = event.target.closest("[data-open-issue]");
    if (issueButton) { state.selectedIssueId = issueButton.dataset.openIssue; navigate("workflow"); return; }
    const reviewButton = event.target.closest("[data-review-issue]");
    if (reviewButton) { state.selectedReviewId = reviewButton.dataset.reviewIssue; renderReview(); return; }
    const actionButton = event.target.closest("[data-action]");
    if (!actionButton) return;
    const action = actionButton.dataset.action;
    if (action === "refresh") { render(); showToast("数据已刷新", "工作台已载入最新演示数据。 "); }
    if (action === "export-report") exportCsv("report");
    if (action === "export-logs") exportCsv("logs");
    if (action === "reset-filters") { state.intakeSearch = ""; state.intakeStatus = "全部状态"; state.intakeCategory = "全部分类"; renderIntake(); }
    if (action === "open-status") openStatusModal();
    if (action === "request-supplement") openStatusModal("待补充", "请补充具体位置、发生时间或能够说明问题的现场图片，便于进一步核查处理。 ");
    if (action === "close-issue") openStatusModal("已关闭", "经核查，该诉求不再继续处理。请填写关闭原因及已向学生说明的事项。 ");
    if (action === "save-assignment") saveAssignment();
    if (action === "go-review") { state.selectedReviewId = getIssue().id; navigate("review"); }
    if (action === "save-draft") saveReviewDraft();
    if (action === "approve-review") approveReview();
    if (action === "privacy-guide") showToast("脱敏规范", "不得公开姓名、学号、手机号、精确宿舍号、原始图片及个人纠纷等信息。 ");
    if (action === "batch") openBatchModal();
    if (action === "new-intake") showToast("演示提示", "线下诉求登记表单不在本轮 Demo 范围内。 ");
    if (action === "copy-id") {
      try { await navigator.clipboard.writeText(getIssue().id); showToast("已复制", `诉求编号 ${getIssue().id} 已复制。`); }
      catch (_) { showToast("诉求编号", getIssue().id); }
    }
  });

  mainContent.addEventListener("input", (event) => {
    if (event.target.id === "intakeSearch") { state.intakeSearch = event.target.value; updateIntakeTable(); }
    if (event.target.id === "logSearch") { state.logSearch = event.target.value; renderLogs(); const input = document.getElementById("logSearch"); input.focus(); input.setSelectionRange(input.value.length, input.value.length); }
  });

  mainContent.addEventListener("change", (event) => {
    if (event.target.id === "intakeStatus") { state.intakeStatus = event.target.value; updateIntakeTable(); }
    if (event.target.id === "intakeCategory") { state.intakeCategory = event.target.value; updateIntakeTable(); }
    if (event.target.id === "selectAll") {
      state.batchSelected = event.target.checked ? filteredIssues().map((issue) => issue.id) : [];
      updateIntakeTable();
    }
    if (event.target.classList && event.target.classList.contains("row-check")) {
      const id = event.target.dataset.id;
      if (event.target.checked) {
        if (!state.batchSelected.includes(id)) state.batchSelected.push(id);
      } else {
        state.batchSelected = state.batchSelected.filter((item) => item !== id);
      }
      updateIntakeTable();
    }
  });

  statusForm.addEventListener("submit", (event) => {
    event.preventDefault();
    const note = statusNote.value.trim();
    if (!note) {
      document.getElementById("statusNoteError").textContent = "处理说明不能为空，请填写本次操作依据或下一步安排。";
      statusNote.focus();
      return;
    }
    const issue = getIssue();
    const before = issue.status;
    const after = nextStatus.value;
    if (!(STATUS_TRANSITIONS[before] || []).includes(after)) {
      document.getElementById("statusNoteError").textContent = `“${before}”状态不能直接切换为“${after}”，请选择合法的下一状态。`;
      nextStatus.focus();
      return;
    }
    issue.status = after;
    if (after === "已办结") issue.deadline = "已按时办结";
    if (after === "已关闭") issue.deadline = "已关闭";
    issue.timeline.unshift({ status: after, time: nowString().slice(0, 16), operator: "林知夏 · 权益工作小组", note });
    addLog("更新状态", issue, before, after, note);
    closeStatusModal();
    render();
    const notified = document.getElementById("notifyStudent").checked;
    showToast("状态更新成功", `${issue.id} 已更新为“${after}”${notified ? "，并模拟通知学生" : ""}。`);
  });

  batchForm.addEventListener("submit", (event) => {
    event.preventDefault();
    const department = document.getElementById("batchDepartment").value;
    const assignee = document.getElementById("batchAssignee").value;
    const targets = issues.filter((issue) => state.batchSelected.includes(issue.id));
    if (!targets.length) { closeBatchModal(); return; }
    targets.forEach((issue) => {
      issue.department = department;
      issue.assignee = assignee;
      issue.timeline.unshift({ status: "批量分派", time: nowString().slice(0, 16), operator: "林知夏 · 权益工作小组", note: `批量指派至 ${department}，负责人 ${assignee}。` });
    });
    addLog("批量指派", targets[0], "多选", "多选", `批量指派 ${targets.length} 条诉求至 ${department} / ${assignee}。`);
    const count = targets.length;
    closeBatchModal();
    state.batchSelected = [];
    render();
    showToast("批量指派完成", `已为 ${count} 条诉求指派 ${department} / ${assignee}。`);
  });

  statusNote.addEventListener("input", () => {
    document.getElementById("noteCount").textContent = statusNote.value.length;
    if (statusNote.value.trim()) document.getElementById("statusNoteError").textContent = "";
  });

  document.getElementById("closeModalButton").addEventListener("click", closeStatusModal);
  document.getElementById("cancelModalButton").addEventListener("click", closeStatusModal);
  statusModalBackdrop.addEventListener("click", (event) => { if (event.target === statusModalBackdrop) closeStatusModal(); });
  document.getElementById("closeBatchModalButton").addEventListener("click", closeBatchModal);
  document.getElementById("cancelBatchModalButton").addEventListener("click", closeBatchModal);
  batchModalBackdrop.addEventListener("click", (event) => { if (event.target === batchModalBackdrop) closeBatchModal(); });
  document.getElementById("menuButton").addEventListener("click", () => {
    document.getElementById("sidebar").classList.contains("open") ? closeSidebar() : openSidebar();
  });
  document.getElementById("sidebarBackdrop").addEventListener("click", closeSidebar);
  document.getElementById("notificationButton").addEventListener("click", () => showToast("2 条待办提醒", "1 件诉求临近超期，1 份部门回复等待审核。 "));
  document.addEventListener("keydown", (event) => {
    if (event.key !== "Escape") return;
    if (!statusModalBackdrop.hidden) closeStatusModal();
    if (!batchModalBackdrop.hidden) closeBatchModal();
  });

  render();
})();
