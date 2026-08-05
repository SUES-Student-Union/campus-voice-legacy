import { Issue } from '../../types/issue'
import { getIssues } from '../../utils/issues'

Component({
  data: {
    tabs: [
      { key: 'all', label: '全部' },
      { key: 'processing', label: '处理中' },
      { key: 'supplement', label: '待补充' },
      { key: 'completed', label: '已办结' }
    ],
    activeTab: 'all',
    issues: [] as Issue[],
    filteredIssues: [] as Issue[]
  },
  pageLifetimes: {
    show() { this.loadIssues() }
  },
  methods: {
    loadIssues() {
      const issues = getIssues()
      this.setData({ issues }, () => this.applyFilter())
    },
    selectTab(event: any) {
      this.setData({ activeTab: event.currentTarget.dataset.key }, () => this.applyFilter())
    },
    applyFilter() {
      const { activeTab, issues } = this.data
      let filtered = issues
      if (activeTab === 'processing') filtered = issues.filter((issue) => ['pending', 'accepted', 'processing', 'review'].includes(issue.statusKey))
      if (activeTab === 'supplement') filtered = issues.filter((issue) => issue.statusKey === 'supplement')
      if (activeTab === 'completed') filtered = issues.filter((issue) => issue.statusKey === 'completed')
      this.setData({ filteredIssues: filtered })
    },
    openIssue(event: any) {
      wx.navigateTo({ url: `/pages/issue-detail/issue-detail?id=${event.currentTarget.dataset.id}` })
    },
    goSubmit() {
      wx.navigateTo({ url: '/pages/submit/submit' })
    }
  }
})
