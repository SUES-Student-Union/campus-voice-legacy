Page({
  data: { id: '', no: '' },
  onLoad(options: Record<string, string>) {
    this.setData({ id: options.id || '', no: options.no || options.id || '' })
  },
  copyId() {
    wx.setClipboardData({ data: this.data.no })
  },
  viewIssue() {
    wx.redirectTo({ url: `/pages/issue-detail/issue-detail?id=${this.data.id}` })
  },
  backHome() {
    wx.switchTab({ url: '/pages/index/index' })
  }
})
