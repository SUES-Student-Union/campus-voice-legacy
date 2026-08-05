import { get, getRole, getStaffName, isStaff, staffLogout } from '../../../utils/api'
import { ApiDashboard } from '../../../types/api'

Page({
  data: {
    role: 'student',
    staffName: '',
    isWorker: false,
    isDeptStaff: false,
    stats: null as ApiDashboard | null,
    loading: true
  },
  onShow() {
    if (!isStaff()) {
      wx.reLaunch({ url: '/pages/index/index' })
      return
    }
    const role = getRole()
    this.setData({
      role,
      staffName: getStaffName(),
      isWorker: role === 'worker' || role === 'admin',
      isDeptStaff: role === 'department_staff'
    })
    this.loadStats()
  },
  async loadStats() {
    if (!this.data.isWorker) {
      this.setData({ loading: false })
      return
    }
    try {
      const stats = await get<ApiDashboard>('/admin/dashboard')
      this.setData({ stats, loading: false })
    } catch (err) {
      wx.showToast({ title: (err as Error).message || '加载失败', icon: 'none' })
      this.setData({ loading: false })
    }
  },
  goIssues() {
    wx.navigateTo({ url: '/pages/admin/issues/issues' })
  },
  goReview() {
    wx.navigateTo({ url: '/pages/admin/review/review' })
  },
  goLogs() {
    wx.navigateTo({ url: '/pages/admin/logs/logs' })
  },
  logout() {
    wx.showModal({
      title: '退出登录',
      content: '确定退出管理端吗？',
      success: (res) => {
        if (res.confirm) {
          staffLogout()
          wx.reLaunch({ url: '/pages/index/index' })
        }
      }
    })
  }
})
