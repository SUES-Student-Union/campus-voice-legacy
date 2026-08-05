import { downloadAttachment, get, post } from '../../utils/api'
import { ApiIssueDetail, IssueDetail, toIssueDetail } from '../../types/api'

Page({
  data: {
    id: '',
    issue: null as IssueDetail | null,
    ratings: ['满意', '一般', '不满意']
  },
  onLoad(options: Record<string, string>) {
    this.setData({ id: options.id || '' })
    this.loadIssue(options.id || '')
  },
  onShow() {
    if (this.data.id) this.loadIssue(this.data.id)
  },
  async loadIssue(id: string) {
    try {
      const api = await get<ApiIssueDetail>(`/my/issues/${id}`)
      const issue = toIssueDetail(api)
      if (api.attachments.length) {
        const paths = await Promise.all(api.attachments.map((a) => downloadAttachment(a.id)))
        issue.images = paths.filter((p) => p !== '')
      }
      this.setData({ issue })
    } catch (err) {
      wx.showToast({ title: (err as Error).message || '加载失败', icon: 'none' })
    }
  },
  copyId() {
    if (this.data.issue) wx.setClipboardData({ data: this.data.issue.no })
  },
  previewImage(event: any) {
    const issue = this.data.issue
    if (issue && issue.images.length) {
      wx.previewImage({ current: event.currentTarget.dataset.src, urls: issue.images })
    }
  },
  async selectRating(event: any) {
    const issue = this.data.issue
    if (!issue) return
    const rating = event.currentTarget.dataset.rating
    try {
      const updated = await post<ApiIssueDetail>(`/my/issues/${issue.id}/rating`, { rating, comment: '' })
      this.setData({ issue: toIssueDetail(updated) })
      wx.showToast({ title: '感谢你的评价', icon: 'success' })
    } catch (err) {
      wx.showToast({ title: (err as Error).message || '评价失败', icon: 'none' })
    }
  },
  addMaterial() {
    const issue = this.data.issue
    if (issue) wx.navigateTo({ url: `/pages/supplement/supplement?id=${issue.id}` })
  }
})
