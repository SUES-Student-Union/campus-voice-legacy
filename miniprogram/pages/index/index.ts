import { get, getRole, isStaff, ssoLogin } from '../../utils/api'
import { ApiIssueListItem, IssueListItem, toIssueListItem } from '../../types/api'

const ROLE_PRESETS: { key: string; label: string; campusId: string }[] = [
  { key: 'student', label: '学生', campusId: '20260001' },
  { key: 'worker', label: '权益小组', campusId: 'W0001' },
  { key: 'department_staff', label: '部门老师', campusId: 'T0001' },
  { key: 'admin', label: '管理员', campusId: 'admin' }
]

Component({
  data: {
    role: 'student',
    roleLabel: '学生',
    rolePresets: ROLE_PRESETS,
    currentIssue: null as IssueListItem | null,
    completedCount: 0,
    processingCount: 0,
    averageRating: '—',
    openFaq: -1,
    faqs: [
      { question: '哪些问题可以在这里反馈？', answer: '可反馈宿舍、食堂、教学、交通、校园环境、网络、医疗健康和学生服务等校园权益问题。' },
      { question: '一般多久能收到回复？', answer: '权益工作小组会尽快完成受理，具体办理时长取决于问题类型和负责部门，关键节点会通过订阅消息提醒。' },
      { question: '公开反馈会展示个人信息吗？', answer: '不会。公开内容须经审核脱敏，不展示姓名、学号、手机号、精确宿舍号和原始图片。' }
    ],
    guides: [
      { tag: '校历', title: '学期校历与假期安排', source: '教务相关指引' },
      { tag: '志愿', title: '三下乡与志愿服务咨询', source: '青春 SUES 指引' },
      { tag: '团务', title: '团委与第二课堂常见问题', source: '学生服务指引' }
    ]
  },
  pageLifetimes: {
    show() {
      this.syncRole()
      this.loadOverview()
    }
  },
  methods: {
    syncRole() {
      const role = getRole()
      const preset = ROLE_PRESETS.find((r) => r.key === role)
      this.setData({ role, roleLabel: preset ? preset.label : '学生' })
    },
    switchRole(event: any) {
      const key = event.currentTarget.dataset.key
      if (key === this.data.role) return
      const preset = ROLE_PRESETS.find((r) => r.key === key)
      if (!preset) return
      wx.showModal({
        title: '切换身份',
        content: `切换为「${preset.label}」？（开发环境演示用）`,
        success: (res) => {
          if (!res.confirm) return
          ssoLogin(preset.campusId, 'demo123')
            .then(() => {
              this.syncRole()
              wx.showToast({ title: `已切换为${preset.label}`, icon: 'none' })
              this.loadOverview()
            })
            .catch((err) => {
              wx.showToast({ title: (err as Error).message || '切换失败', icon: 'none' })
            })
        }
      })
    },
    goAdmin() {
      if (isStaff()) {
        wx.navigateTo({ url: '/pages/admin/home/home' })
      }
    },
    async loadOverview() {
      try {
        const list = await get<ApiIssueListItem[]>('/my/issues')
        const items = list.map(toIssueListItem)
        this.setData({
          currentIssue: items[0] || null,
          completedCount: items.filter((i) => i.statusKey === 'completed').length,
          processingCount: items.filter((i) => ['accepted', 'processing', 'review'].includes(i.statusKey)).length
        })
      } catch (err) {
        wx.showToast({ title: (err as Error).message || '加载失败', icon: 'none' })
      }
    },
    goSubmit() {
      wx.navigateTo({ url: '/pages/submit/submit' })
    },
    goIssues() {
      wx.switchTab({ url: '/pages/issues/issues' })
    },
    goPublic() {
      wx.switchTab({ url: '/pages/public/public' })
    },
    goIssueDetail() {
      const issue = this.data.currentIssue
      if (issue) wx.navigateTo({ url: `/pages/issue-detail/issue-detail?id=${issue.id}` })
    },
    toggleFaq(event: any) {
      const index = Number(event.currentTarget.dataset.index)
      this.setData({ openFaq: this.data.openFaq === index ? -1 : index })
    },
    openGuide() {
      wx.showToast({ title: 'Demo 中暂以办事指引卡片展示', icon: 'none' })
    }
  }
})
