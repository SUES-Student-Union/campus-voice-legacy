import { Issue, PublicCase } from '../types/issue'

export const mockIssues: Issue[] = [
  {
    id: 'CV202607180018',
    type: '设施报修建议',
    category: '宿舍',
    campus: '松江校区',
    location: '学生公寓区域',
    title: '公共洗衣房部分设备长期停用',
    description: '近期公共洗衣房有多台设备显示故障，希望尽快检修并张贴报修进度。',
    images: [],
    status: '部门处理中',
    statusKey: 'processing',
    createdAt: '2026-07-18 10:26',
    updatedAt: '2026-07-22 15:40',
    isPublic: true,
    contact: '微信联系',
    department: '后勤实业发展中心',
    latestProgress: '维修单位已完成现场排查，配件正在调拨。',
    progress: [
      { status: '学生提交', time: '07-18 10:26', operator: '学生', description: '诉求已提交并生成编号。' },
      { status: '权益工作小组受理', time: '07-18 14:05', operator: '权益工作小组', description: '确认属于受理范围，转交后勤部门。' },
      { status: '部门处理中', time: '07-22 15:40', operator: '后勤实业发展中心', description: '已完成现场排查，配件正在调拨。', current: true }
    ]
  },
  {
    id: 'CV202607120006',
    type: '意见建议',
    category: '交通',
    campus: '松江校区',
    location: '教学楼周边',
    title: '建议优化晚间校园接驳车班次',
    description: '晚课结束后候车人数较多，建议在高峰时段增加班次。',
    images: [],
    status: '待补充',
    statusKey: 'supplement',
    createdAt: '2026-07-12 19:32',
    updatedAt: '2026-07-13 09:18',
    isPublic: false,
    contact: '微信联系',
    department: '权益工作小组',
    latestProgress: '请补充常用候车站点和具体时段。',
    progress: [
      { status: '学生提交', time: '07-12 19:32', operator: '学生', description: '诉求已提交并生成编号。' },
      { status: '待补充', time: '07-13 09:18', operator: '权益工作小组', description: '请补充常用候车站点和具体时段。', current: true }
    ]
  },
  {
    id: 'CV202606280031',
    type: '问题咨询',
    category: '网络',
    campus: '长宁校区',
    location: '教学区域',
    title: '校园网访客设备接入方式咨询',
    description: '想了解临时访客设备如何申请接入校园网络。',
    images: [],
    status: '已办结',
    statusKey: 'completed',
    createdAt: '2026-06-28 08:44',
    updatedAt: '2026-06-30 16:12',
    isPublic: true,
    contact: '微信联系',
    department: '信息化办公室',
    latestProgress: '已提供访客网络申请指引。',
    reply: '访客可由校内接待单位在校园服务平台发起临时网络申请，审核通过后按短信指引完成认证。',
    progress: [
      { status: '学生提交', time: '06-28 08:44', operator: '学生', description: '诉求已提交并生成编号。' },
      { status: '已受理', time: '06-28 11:20', operator: '权益工作小组', description: '已转交信息化办公室。' },
      { status: '部门回复', time: '06-30 14:35', operator: '信息化办公室', description: '已提供访客网络申请指引。' },
      { status: '已办结', time: '06-30 16:12', operator: '权益工作小组', description: '正式回复已反馈学生。', current: true }
    ]
  }
]

export const publicCases: PublicCase[] = [
  {
    id: 'PUB20260715012',
    category: '食堂',
    campus: '松江校区',
    location: '第一食堂区域',
    title: '关于晚间档口营业时间的建议',
    summary: '同学建议考试周适当延长部分档口晚间营业时间。',
    status: '已办结',
    statusKey: 'completed',
    submittedAt: '2026-07-09',
    completedAt: '2026-07-15',
    department: '后勤实业发展中心',
    reply: '考试周期间将安排一楼两处档口延长营业至 21:30，并根据客流动态调整。',
    progress: [
      { status: '受理并转交', time: '07-10', operator: '权益工作小组', description: '已转交食堂管理部门评估。' },
      { status: '部门处理', time: '07-12', operator: '后勤实业发展中心', description: '完成客流调研和档口协调。' },
      { status: '审核公开', time: '07-15', operator: '权益工作小组', description: '脱敏审核通过并公开。', current: true }
    ]
  },
  {
    id: 'PUB20260708009',
    category: '校园环境',
    campus: '长宁校区',
    location: '图书馆周边',
    title: '雨天连廊地面湿滑问题',
    summary: '同学反映雨天部分连廊容易积水，希望增加防滑提醒。',
    status: '已办结',
    statusKey: 'completed',
    submittedAt: '2026-07-03',
    completedAt: '2026-07-08',
    department: '校园管理部门',
    reply: '已增设防滑垫与警示牌，并将该区域纳入雨天重点巡查清单。',
    progress: [
      { status: '受理并转交', time: '07-03', operator: '权益工作小组', description: '确认问题并转交校园管理部门。' },
      { status: '现场处理', time: '07-05', operator: '校园管理部门', description: '完成防滑设施增设。' },
      { status: '审核公开', time: '07-08', operator: '权益工作小组', description: '脱敏审核通过并公开。', current: true }
    ]
  },
  {
    id: 'PUB20260721017',
    category: '学生服务',
    campus: '松江校区',
    location: '学生服务中心',
    title: '暑期证明材料办理时间咨询',
    summary: '同学咨询暑期在校证明等材料的办理方式和服务时间。',
    status: '部门处理中',
    statusKey: 'processing',
    submittedAt: '2026-07-20',
    department: '学生工作部门',
    reply: '相关暑期服务安排正在汇总，将通过校园公告统一发布。',
    progress: [
      { status: '受理并转交', time: '07-20', operator: '权益工作小组', description: '已转交学生工作部门。' },
      { status: '部门处理中', time: '07-21', operator: '学生工作部门', description: '正在汇总暑期值班和线上办理安排。', current: true }
    ]
  }
]
