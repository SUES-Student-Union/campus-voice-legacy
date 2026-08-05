import { publicCases } from '../../data/mock'
import { PublicCase } from '../../types/issue'

Component({
  data: {
    keyword: '',
    categories: ['全部', '食堂', '校园环境', '学生服务'],
    activeCategory: '全部',
    statuses: [
      { key: 'all', label: '全部状态' },
      { key: 'completed', label: '已办结' },
      { key: 'processing', label: '处理中' }
    ],
    activeStatus: 'all',
    cases: publicCases,
    filteredCases: publicCases as PublicCase[]
  },
  methods: {
    onKeywordInput(event: any) {
      this.setData({ keyword: event.detail.value }, () => this.applyFilter())
    },
    clearKeyword() {
      this.setData({ keyword: '' }, () => this.applyFilter())
    },
    selectCategory(event: any) {
      this.setData({ activeCategory: event.currentTarget.dataset.category }, () => this.applyFilter())
    },
    selectStatus(event: any) {
      this.setData({ activeStatus: event.currentTarget.dataset.status }, () => this.applyFilter())
    },
    applyFilter() {
      const keyword = this.data.keyword.trim().toLowerCase()
      const filtered = this.data.cases.filter((item) => {
        const matchCategory = this.data.activeCategory === '全部' || item.category === this.data.activeCategory
        const matchStatus = this.data.activeStatus === 'all' || item.statusKey === this.data.activeStatus
        const haystack = `${item.title}${item.summary}${item.category}${item.campus}${item.department}`.toLowerCase()
        return matchCategory && matchStatus && (!keyword || haystack.includes(keyword))
      })
      this.setData({ filteredCases: filtered })
    },
    openCase(event: any) {
      wx.navigateTo({ url: `/pages/public-detail/public-detail?id=${event.currentTarget.dataset.id}` })
    }
  }
})
