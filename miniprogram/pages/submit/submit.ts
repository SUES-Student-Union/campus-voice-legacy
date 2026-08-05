import { post, uploadImages } from '../../utils/api'
import { ApiIssueDetail } from '../../types/api'

Component({
  data: {
    types: ['意见建议', '问题咨询', '投诉反馈', '设施报修建议', '其他'],
    categories: ['宿舍', '食堂', '教学', '交通', '校园环境', '网络', '医疗健康', '学生服务', '其他'],
    campuses: ['松江校区', '长宁校区', '虹口校区'],
    typeIndex: -1,
    categoryIndex: -1,
    campusIndex: -1,
    location: '',
    title: '',
    description: '',
    contact: '',
    images: [] as string[],
    isPublic: false,
    agreed: false,
    isSubmitting: false
  },
  methods: {
    onTypeChange(event: any) { this.setData({ typeIndex: Number(event.detail.value) }) },
    onCategoryChange(event: any) { this.setData({ categoryIndex: Number(event.detail.value) }) },
    onCampusChange(event: any) { this.setData({ campusIndex: Number(event.detail.value) }) },
    onLocationInput(event: any) { this.setData({ location: event.detail.value }) },
    onTitleInput(event: any) { this.setData({ title: event.detail.value }) },
    onDescriptionInput(event: any) { this.setData({ description: event.detail.value }) },
    onContactInput(event: any) { this.setData({ contact: event.detail.value }) },
    onPublicChange(event: any) { this.setData({ isPublic: event.detail.value }) },
    toggleAgreement() { this.setData({ agreed: !this.data.agreed }) },
    chooseImage() {
      const remain = 3 - this.data.images.length
      if (remain <= 0) {
        wx.showToast({ title: '最多上传 3 张图片', icon: 'none' })
        return
      }
      wx.chooseMedia({
        count: remain,
        mediaType: ['image'],
        sizeType: ['compressed'],
        sourceType: ['album', 'camera'],
        success: (result) => {
          const MAX_SIZE = 5 * 1024 * 1024
          const oversized = result.tempFiles.filter((file) => file.size > MAX_SIZE).length
          const selected = result.tempFiles.filter((file) => file.size <= MAX_SIZE).map((file) => file.tempFilePath)
          if (oversized > 0) {
            wx.showToast({ title: `已忽略 ${oversized} 张超过 5MB 的图片`, icon: 'none' })
          }
          this.setData({ images: this.data.images.concat(selected).slice(0, 3) })
        }
      })
    },
    previewImage(event: any) {
      const current = event.currentTarget.dataset.src
      wx.previewImage({ current, urls: this.data.images })
    },
    removeImage(event: any) {
      const index = Number(event.currentTarget.dataset.index)
      this.setData({ images: this.data.images.filter((_item, itemIndex) => itemIndex !== index) })
    },
    submitIssue() {
      if (this.data.typeIndex < 0) return this.showError('请选择诉求类型')
      if (this.data.categoryIndex < 0) return this.showError('请选择所属类别')
      if (this.data.campusIndex < 0) return this.showError('请选择校区')
      if (!this.data.location.trim()) return this.showError('请填写具体地点')
      if (!this.data.title.trim()) return this.showError('请填写诉求标题')
      if (this.data.description.trim().length < 10) return this.showError('详细描述至少填写 10 个字')
      if (!this.data.agreed) return this.showError('请阅读并同意隐私与诉求处理说明')
      if (this.data.isSubmitting) return

      this.setData({ isSubmitting: true })
      post<ApiIssueDetail>('/issues', {
        kind: this.data.types[this.data.typeIndex],
        category: this.data.categories[this.data.categoryIndex],
        campus: this.data.campuses[this.data.campusIndex],
        location: this.data.location.trim(),
        title: this.data.title.trim(),
        description: this.data.description.trim(),
        public_will: this.data.isPublic,
        contact: this.data.contact.trim()
      })
        .then(async (issue) => {
          if (this.data.images.length) {
            await uploadImages(issue.id, this.data.images).catch(() => {
              wx.showToast({ title: '诉求已提交，但图片上传失败', icon: 'none' })
            })
          }
          wx.redirectTo({ url: `/pages/success/success?id=${issue.id}&no=${issue.issue_no}` })
        })
        .catch((err) => {
          this.showError((err as Error).message || '提交失败')
          this.setData({ isSubmitting: false })
        })
    },
    showError(message: string) {
      wx.showToast({ title: message, icon: 'none', duration: 2200 })
    }
  }
})
