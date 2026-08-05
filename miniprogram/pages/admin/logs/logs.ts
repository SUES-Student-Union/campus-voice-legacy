import { get } from '../../../utils/api'
import { ApiAuditLog, AuditLogItem, toAuditLogItem } from '../../../types/api'

Page({
  data: {
    logs: [] as AuditLogItem[],
    loading: false
  },
  onShow() {
    this.load()
  },
  async load() {
    this.setData({ loading: true })
    try {
      const list = await get<ApiAuditLog[]>('/admin/audit-logs')
      this.setData({ logs: list.map(toAuditLogItem), loading: false })
    } catch (err) {
      wx.showToast({ title: (err as Error).message || '加载失败', icon: 'none' })
      this.setData({ loading: false })
    }
  }
})
