import { get } from '../../utils/api'
import { ApiPublicCase, PublicCaseItem, toPublicCaseItem } from '../../types/api'

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
    cases: [] as PublicCaseItem[],
    filteredCases: [] as PublicCaseItem[]
  },
  pageLifetimes: {
    show() {
      this.loadCases()
    }
  },
  methods: {
    async loadCases() {
      try {
        const list = await get<ApiPublicCase[]>('/public/cases')
        this.setData({ cases: list.map(toPublicCaseItem) }, () => this.applyFilter())
      } catch (err) {
        wx.showToast({ title: (err as Error).message || '加载失败', icon: 'none' })
        this.setData({ cases: [], filteredCases: [] })
      }
    },
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
