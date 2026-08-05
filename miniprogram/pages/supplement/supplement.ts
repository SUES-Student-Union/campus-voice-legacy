import { get, post } from '../../utils/api'
import { ApiIssueDetail } from '../../types/api'

Page({
  data: {
    id: '',
    issueNo: '',
    content: '',
    isSubmitting: false
  },
  onLoad(options: Record<string, string>) {
    this.setData({ id: options.id || '' })
    if (options.id) {
      get<ApiIssueDetail>(`/my/issues/${options.id}`)
        .then((detail) => this.setData({ issueNo: detail.issue_no }))
        .catch(() => undefined)
    }
  },
  onContentInput(event: any) {
    this.setData({ content: event.detail.value })
  },
  async submit() {
    const content = this.data.content.trim()
    if (content.length < 2) {
      wx.showToast({ title: '请填写补充内容', icon: 'none' })
      return
    }
    if (this.data.isSubmitting) return
    this.setData({ isSubmitting: true })
    try {
      await post<ApiIssueDetail>(`/my/issues/${this.data.id}/supplement`, { content })
      wx.showToast({ title: '补充成功', icon: 'success' })
      setTimeout(() => wx.navigateBack(), 800)
    } catch (err) {
      wx.showToast({ title: (err as Error).message || '提交失败', icon: 'none' })
      this.setData({ isSubmitting: false })
    }
  }
})
