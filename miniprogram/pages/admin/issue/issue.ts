import { get, post, getRole } from '../../../utils/api'
import {
  AdminIssueDetail,
  ApiAdminIssueDetail,
  ApiDepartment,
  toAdminIssueDetail
} from '../../../types/api'

Page({
  data: {
    id: '',
    role: 'student',
    issue: null as AdminIssueDetail | null,
    departments: [] as ApiDepartment[],
    categories: ['宿舍', '食堂', '教学', '交通', '校园环境', '网络', '医疗健康', '学生服务', '其他'],
    priorities: ['高', '中', '低'],
    loading: true,
    canAccept: false,
    canAssign: false,
    canSupplement: false,
    canClose: false,
    canReview: false,
    canDeptProgress: false,
    canReply: false,
    action: '',
    actionTitle: '',
    formCategoryIndex: 0,
    formPriorityIndex: 0,
    formPublic: false,
    formDeptIndex: -1,
    formNote: '',
    submitting: false
  },
  onLoad(options: Record<string, string>) {
    const role = getRole()
    if (role === 'student') {
      wx.reLaunch({ url: '/pages/index/index' })
      return
    }
    this.setData({ id: options.id || '', role })
    if (role !== 'department_staff') this.loadDepartments()
  },
  onShow() {
    if (this.data.id) this.load()
  },
  async load() {
    try {
      const role = getRole()
      const path = role === 'department_staff'
        ? `/department/issues/${this.data.id}`
        : `/admin/issues/${this.data.id}`
      const api = await get<ApiAdminIssueDetail>(path)
      const status = api.status
      const isWorker = role !== 'department_staff'
      const terminal = status === '已办结' || status === '已关闭'
      this.setData({
        issue: toAdminIssueDetail(api),
        loading: false,
        canAccept: isWorker && status === '待受理',
        canAssign: isWorker && !terminal && status !== '待受理',
        canSupplement: isWorker && !terminal,
        canClose: isWorker && !terminal,
        canReview: isWorker && status === '待审核',
        canDeptProgress: !isWorker && status === '部门处理中',
        canReply: !isWorker && status === '部门处理中'
      })
    } catch (err) {
      wx.showToast({ title: (err as Error).message || '加载失败', icon: 'none' })
      this.setData({ loading: false })
    }
  },
  async loadDepartments() {
    try {
      this.setData({ departments: await get<ApiDepartment[]>('/departments') })
    } catch (err) {
      wx.showToast({ title: (err as Error).message || '部门加载失败', icon: 'none' })
    }
  },
  openAction(event: any) {
    const action = event.currentTarget.dataset.action
    const titles: Record<string, string> = {
      accept: '受理诉求',
      assign: '指派部门',
      supplement: '请求补充信息',
      close: '关闭诉求',
      review: '公开审核',
      progress: '更新进展',
      reply: '提交正式回复'
    }
    this.setData({
      action,
      actionTitle: titles[action] || '',
      formCategoryIndex: 0,
      formPriorityIndex: 0,
      formPublic: false,
      formDeptIndex: -1,
      formNote: ''
    })
  },
  closeAction() {
    this.setData({ action: '' })
  },
  noop() {},
  onCat(event: any) { this.setData({ formCategoryIndex: Number(event.detail.value) }) },
  onPriority(event: any) { this.setData({ formPriorityIndex: Number(event.detail.value) }) },
  onPublic(event: any) { this.setData({ formPublic: event.detail.value }) },
  onDept(event: any) { this.setData({ formDeptIndex: Number(event.detail.value) }) },
  onNote(event: any) { this.setData({ formNote: event.detail.value }) },
  async submitAction() {
    const { action, id, issue, submitting } = this.data
    if (!issue || submitting) return
    const note = this.data.formNote.trim()
    let path = ''
    let body: any = {}

    if (action === 'accept') {
      path = `/admin/issues/${id}/accept`
      body = {
        category: this.data.categories[this.data.formCategoryIndex],
        priority: this.data.priorities[this.data.formPriorityIndex],
        is_public: this.data.formPublic
      }
    } else if (action === 'assign') {
      if (this.data.formDeptIndex < 0) return this.toast('请选择部门')
      path = `/admin/issues/${id}/assign`
      body = { department_id: this.data.departments[this.data.formDeptIndex].id, assignee_id: null }
    } else if (action === 'supplement') {
      if (!note) return this.toast('请填写需补充的说明')
      path = `/admin/issues/${id}/request-supplement`
      body = { note }
    } else if (action === 'close') {
      if (!note) return this.toast('请填写关闭原因')
      path = `/admin/issues/${id}/close`
      body = { note }
    } else if (action === 'progress') {
      if (!note) return this.toast('请填写进展说明')
      path = `/department/issues/${id}/progress`
      body = { note }
    } else if (action === 'reply') {
      if (!note) return this.toast('请填写正式回复内容')
      path = `/department/issues/${id}/reply`
      body = { content: note }
    }

    this.setData({ submitting: true })
    try {
      await post<any>(path, body)
      wx.showToast({ title: '操作成功', icon: 'success' })
      this.setData({ action: '' })
      this.load()
    } catch (err) {
      wx.showToast({ title: (err as Error).message || '操作失败', icon: 'none' })
    }
    this.setData({ submitting: false })
  },
  goReview() {
    wx.navigateTo({ url: `/pages/admin/review/review?id=${this.data.id}` })
  },
  toast(message: string) {
    wx.showToast({ title: message, icon: 'none' })
  }
})
