import { get, post } from '../../../utils/api'
import {
  AdminIssueDetail,
  ApiAdminIssueDetail,
  ApiIssueListItem,
  IssueListItem,
  toAdminIssueDetail,
  toIssueListItem
} from '../../../types/api'

Page({
  data: {
    mode: 'list',
    issues: [] as IssueListItem[],
    id: '',
    issue: null as AdminIssueDetail | null,
    title: '',
    location: '',
    summary: '',
    reply: '',
    note: '',
    submitting: false
  },
  onLoad(options: Record<string, string>) {
    if (options.id) {
      this.setData({ mode: 'edit', id: options.id })
      this.loadEdit(options.id)
    } else {
      this.loadList()
    }
  },
  async loadList() {
    try {
      const list = await get<ApiIssueListItem[]>(
        '/admin/issues?status=' + encodeURIComponent('待审核')
      )
      this.setData({ issues: list.map(toIssueListItem) })
    } catch (err) {
      wx.showToast({ title: (err as Error).message || '加载失败', icon: 'none' })
    }
  },
  async loadEdit(id: string) {
    try {
      const api = await get<ApiAdminIssueDetail>(`/admin/issues/${id}`)
      const issue = toAdminIssueDetail(api)
      const draft = api.review_draft
      this.setData({
        issue,
        title: draft ? draft.title : issue.title,
        location: draft ? draft.location : issue.location,
        summary: draft ? draft.summary : '',
        reply: draft ? draft.reply : issue.reply
      })
    } catch (err) {
      wx.showToast({ title: (err as Error).message || '加载失败', icon: 'none' })
    }
  },
  openEdit(event: any) {
    wx.navigateTo({ url: `/pages/admin/review/review?id=${event.currentTarget.dataset.id}` })
  },
  onTitle(event: any) { this.setData({ title: event.detail.value }) },
  onLocation(event: any) { this.setData({ location: event.detail.value }) },
  onSummary(event: any) { this.setData({ summary: event.detail.value }) },
  onReply(event: any) { this.setData({ reply: event.detail.value }) },
  onNote(event: any) { this.setData({ note: event.detail.value }) },
  saveDraft() { this.submit('draft') },
  approve() { this.submit('approve') },
  reject() { this.submit('reject') },
  async submit(action: string) {
    const { id, title, location, summary, reply, note, submitting } = this.data
    if (submitting) return
    if (action !== 'reject' && (!title.trim() || !location.trim() || !summary.trim() || !reply.trim())) {
      return this.toast('请填写完整的脱敏稿')
    }
    if (action === 'reject' && !note.trim()) {
      return this.toast('请填写打回原因')
    }
    this.setData({ submitting: true })
    try {
      await post(`/admin/issues/${id}/review`, { action, title, location, summary, reply, note })
      wx.showToast({
        title: action === 'approve' ? '已通过并公开' : action === 'reject' ? '已打回' : '草稿已保存',
        icon: 'success'
      })
      setTimeout(() => wx.navigateBack(), 800)
    } catch (err) {
      wx.showToast({ title: (err as Error).message || '操作失败', icon: 'none' })
      this.setData({ submitting: false })
    }
  },
  toast(message: string) {
    wx.showToast({ title: message, icon: 'none' })
  }
})
