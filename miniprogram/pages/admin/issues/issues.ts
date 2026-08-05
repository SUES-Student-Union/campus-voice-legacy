import { get, getRole, isStaff } from '../../../utils/api'
import { ApiIssueListItem, IssueListItem, toIssueListItem } from '../../../types/api'

interface TabDef {
  key: string
  label: string
}

const WORKER_TABS: TabDef[] = [
  { key: 'all', label: '全部' },
  { key: '待受理', label: '待受理' },
  { key: '待补充', label: '待补充' },
  { key: '已受理', label: '已受理' },
  { key: '部门处理中', label: '部门处理中' },
  { key: '待审核', label: '待审核' },
  { key: '已办结', label: '已办结' }
]

const DEPT_TABS: TabDef[] = [
  { key: 'all', label: '全部' },
  { key: '部门处理中', label: '部门处理中' },
  { key: '待审核', label: '待审核' },
  { key: '待补充', label: '待补充' }
]

Page({
  data: {
    role: 'student',
    tabs: WORKER_TABS as TabDef[],
    active: 'all',
    issues: [] as IssueListItem[],
    loading: false
  },
  onLoad() {
    const role = getRole()
    this.setData({ role, tabs: role === 'department_staff' ? DEPT_TABS : WORKER_TABS })
  },
  onShow() {
    this.load()
  },
  async load() {
    if (!isStaff()) {
      wx.reLaunch({ url: '/pages/index/index' })
      return
    }
    this.setData({ loading: true })
    try {
      const role = getRole()
      const path = role === 'department_staff' ? '/department/issues' : '/admin/issues'
      const query = this.data.active === 'all' ? '' : `?status=${encodeURIComponent(this.data.active)}`
      const list = await get<ApiIssueListItem[]>(path + query)
      this.setData({ issues: list.map(toIssueListItem), loading: false })
    } catch (err) {
      wx.showToast({ title: (err as Error).message || '加载失败', icon: 'none' })
      this.setData({ loading: false })
    }
  },
  selectTab(event: any) {
    this.setData({ active: event.currentTarget.dataset.key }, () => this.load())
  },
  openIssue(event: any) {
    wx.navigateTo({ url: `/pages/admin/issue/issue?id=${event.currentTarget.dataset.id}` })
  }
})
