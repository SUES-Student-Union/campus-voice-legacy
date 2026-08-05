import { publicCases } from '../../data/mock'
import { PublicCase } from '../../types/issue'

Page({
  data: {
    publicCase: null as PublicCase | null
  },
  onLoad(options: Record<string, string>) {
    const publicCase = publicCases.find((item) => item.id === options.id)
    if (!publicCase) {
      wx.showToast({ title: '未找到公开反馈', icon: 'none' })
      return
    }
    this.setData({ publicCase })
  }
})
