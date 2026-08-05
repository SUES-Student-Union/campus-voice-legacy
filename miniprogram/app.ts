import { ensureLogin } from './utils/api'

App<IAppOption>({
  globalData: {},
  onLaunch() {
    ensureLogin()
      .then(() => {
        console.log('后端登录成功')
      })
      .catch((err) => {
        console.error('后端登录失败', err)
      })
  }
})
