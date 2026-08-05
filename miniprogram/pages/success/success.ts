Page({
  data: { id: '' },
  onLoad(options: Record<string, string>) {
    this.setData({ id: options.id || '' })
  },
  copyId() {
    wx.setClipboardData({ data: this.data.id })
  },
  viewIssue() {
    wx.redirectTo({ url: `/pages/issue-detail/issue-detail?id=${this.data.id}` })
  },
  backHome() {
    wx.switchTab({ url: '/pages/index/index' })
  }
})
