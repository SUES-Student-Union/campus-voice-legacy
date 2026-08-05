import { Issue } from '../../types/issue'
import { getIssueById, updateIssueRating } from '../../utils/issues'

Page({
  data: {
    id: '',
    issue: null as Issue | null,
    ratings: ['满意', '一般', '不满意']
  },
  onLoad(options: Record<string, string>) {
    this.setData({ id: options.id || '' })
    this.loadIssue(options.id || '')
  },
  onShow() {
    if (this.data.id) this.loadIssue(this.data.id)
  },
  loadIssue(id: string) {
    const issue = getIssueById(id)
    if (!issue) {
      wx.showToast({ title: '未找到该诉求', icon: 'none' })
      return
    }
    this.setData({ issue })
  },
  copyId() {
    if (this.data.issue) wx.setClipboardData({ data: this.data.issue.id })
  },
  previewImage(event: any) {
    const issue = this.data.issue
    if (!issue) return
    wx.previewImage({ current: event.currentTarget.dataset.src, urls: issue.images })
  },
  selectRating(event: any) {
    const issue = this.data.issue
    if (!issue) return
    const rating = event.currentTarget.dataset.rating
    const updated = updateIssueRating(issue.id, rating)
    if (updated) {
      this.setData({ issue: updated })
      wx.showToast({ title: '感谢你的评价', icon: 'success' })
    }
  },
  addMaterial() {
    wx.showToast({ title: 'Demo 暂不上传补充材料', icon: 'none' })
  }
})
