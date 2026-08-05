import { get } from '../../utils/api'
import { ApiPublicCase, PublicCaseItem, toPublicCaseItem } from '../../types/api'

Page({
  data: {
    publicCase: null as PublicCaseItem | null
  },
  onLoad(options: Record<string, string>) {
    if (options.id) {
      get<ApiPublicCase>(`/public/cases/${options.id}`)
        .then((data) => this.setData({ publicCase: toPublicCaseItem(data) }))
        .catch((err) => {
          wx.showToast({ title: (err as Error).message || '加载失败', icon: 'none' })
        })
    }
  }
})
