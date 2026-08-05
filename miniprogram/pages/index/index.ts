import { getIssues } from '../../utils/issues'

const RATING_SCORES: Record<string, number> = { '满意': 5, '一般': 3, '不满意': 1 }

Component({
  data: {
    currentIssue: null as ReturnType<typeof getIssues>[number] | null,
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
      const issues = getIssues()
      const rated = issues.filter((issue) => issue.rating)
      const averageRating = rated.length
        ? (rated.reduce((sum, issue) => sum + (RATING_SCORES[issue.rating as string] || 0), 0) / rated.length).toFixed(1)
        : '—'
      this.setData({
        currentIssue: issues[0] || null,
        completedCount: issues.filter((issue) => issue.statusKey === 'completed').length,
        processingCount: issues.filter((issue) => ['accepted', 'processing', 'review'].includes(issue.statusKey)).length,
        averageRating
      })
    }
  },
  methods: {
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
