// 后端地址：
// - 已部署服务器（阿里云）：http://139.196.79.195:8080/api/v1
// - 本地开发：开发者工具模拟器可用 http://127.0.0.1:8080/api/v1；真机调试需改为开发机局域网 IP
// 开发时需要：开发者工具 -> 详情 -> 本地设置 -> 勾选「不校验合法域名…」
const BASE_URL = 'http://139.196.79.195:8080/api/v1'

const TOKEN_KEY = 'cv_token'
const ROLE_KEY = 'cv_role'
const USER_KEY = 'cv_user'

export type Role = 'student' | 'worker' | 'department_staff' | 'admin'

export interface ApiUser {
  id: string
  role: string
  name: string
  campus_id?: string | null
  department_id?: string | null
  phone_masked?: string | null
}

export interface ApiAuth {
  token: string
  user: ApiUser
}

let loginPromise: Promise<string> | null = null

function getToken(): string {
  return (wx.getStorageSync(TOKEN_KEY) as string) || ''
}

function setAuth(token: string, role: string, name: string): void {
  wx.setStorageSync(TOKEN_KEY, token)
  wx.setStorageSync(ROLE_KEY, role)
  wx.setStorageSync(USER_KEY, name)
}

export function clearToken(): void {
  wx.removeStorageSync(TOKEN_KEY)
  wx.removeStorageSync(ROLE_KEY)
  wx.removeStorageSync(USER_KEY)
}

export function getRole(): Role {
  return (wx.getStorageSync(ROLE_KEY) as Role) || 'student'
}

export function getStaffName(): string {
  return (wx.getStorageSync(USER_KEY) as string) || ''
}

export function isStaff(): boolean {
  return getRole() !== 'student'
}

/** 校园统一认证登录：学号/工号+密码，绑定当前微信 openid。 */
export function ssoLogin(campusId: string, password: string): Promise<ApiAuth> {
  return new Promise<ApiAuth>((resolve, reject) => {
    wx.login({
      success: (res) => {
        wx.request({
          url: `${BASE_URL}/auth/sso`,
          method: 'POST',
          data: { campus_id: campusId, password, code: res.code },
          success: (r) => {
            if (r.statusCode >= 200 && r.statusCode < 300) {
              const data = r.data as ApiAuth
              setAuth(data.token, data.user.role, data.user.name)
              resolve(data)
            } else {
              const errBody = r.data as { error?: { message?: string } }
              reject(
                new Error(
                  (errBody && errBody.error && errBody.error.message) || '校园登录失败'
                )
              )
            }
          },
          fail: () => reject(new Error('网络错误'))
        })
      },
      fail: () => reject(new Error('微信登录失败'))
    })
  })
}

export function ensureLogin(): Promise<string> {
  const token = getToken()
  if (token) return Promise.resolve(token)
  if (loginPromise) return loginPromise
  loginPromise = new Promise<string>((resolve, reject) => {
    wx.login({
      success: (res) => {
        wx.request({
          url: `${BASE_URL}/auth/wechat`,
          method: 'POST',
          data: { code: res.code },
          success: (r) => {
            if (r.statusCode === 200) {
              const data = r.data as ApiAuth
              setAuth(data.token, data.user.role, data.user.name)
              loginPromise = null
              resolve(data.token)
            } else {
              loginPromise = null
              reject(new Error('登录失败，请检查后端服务'))
            }
          },
          fail: () => {
            loginPromise = null
            reject(new Error('网络错误，无法连接后端'))
          }
        })
      },
      fail: () => {
        loginPromise = null
        reject(new Error('微信登录失败'))
      }
    })
  })
  return loginPromise
}

export function staffLogin(username: string, password: string): Promise<ApiAuth> {
  return new Promise<ApiAuth>((resolve, reject) => {
    wx.request({
      url: `${BASE_URL}/auth/login`,
      method: 'POST',
      data: { username, password },
      success: (res) => {
        if (res.statusCode >= 200 && res.statusCode < 300) {
          const data = res.data as ApiAuth
          setAuth(data.token, data.user.role, data.user.name)
          resolve(data)
        } else {
          const errBody = res.data as { error?: { message?: string } }
          reject(new Error((errBody && errBody.error && errBody.error.message) || '登录失败'))
        }
      },
      fail: () => reject(new Error('网络错误，无法连接后端'))
    })
  })
}

export function staffLogout(): void {
  clearToken()
}

function doRequest<T>(path: string, method: 'GET' | 'POST', data: any, token: string): Promise<T> {
  return new Promise<T>((resolve, reject) => {
    wx.request({
      url: `${BASE_URL}${path}`,
      method,
      data,
      header: { Authorization: `Bearer ${token}` },
      success: (res) => {
        if (res.statusCode >= 200 && res.statusCode < 300) {
          resolve(res.data as T)
        } else if (res.statusCode === 401) {
          clearToken()
          reject(new Error('登录已过期，请重新登录'))
        } else {
          const errBody = res.data as { error?: { message?: string } }
          const message = errBody && errBody.error ? errBody.error.message : undefined
          reject(new Error(message || `请求失败（${res.statusCode}）`))
        }
      },
      fail: () => reject(new Error('网络错误'))
    })
  })
}

export function get<T>(path: string): Promise<T> {
  return ensureLogin().then((token) => doRequest<T>(path, 'GET', undefined, token))
}

export function post<T>(path: string, data: any): Promise<T> {
  return ensureLogin().then((token) => doRequest<T>(path, 'POST', data, token))
}

export function uploadImages(issueId: string, filePaths: string[]): Promise<void> {
  if (!filePaths.length) return Promise.resolve()
  return ensureLogin().then((token) =>
    Promise.all(
      filePaths.map(
        (filePath) =>
          new Promise<void>((resolve, reject) => {
            wx.uploadFile({
              url: `${BASE_URL}/issues/${issueId}/attachments`,
              filePath,
              name: 'file',
              header: { Authorization: `Bearer ${token}` },
              success: (res) => {
                if (res.statusCode >= 200 && res.statusCode < 300) resolve()
                else reject(new Error('图片上传失败'))
              },
              fail: () => reject(new Error('网络错误'))
            })
          })
      )
    ).then(() => undefined)
  )
}

export function downloadAttachment(attachmentId: string): Promise<string> {
  return ensureLogin().then(
    (token) =>
      new Promise<string>((resolve) => {
        wx.downloadFile({
          url: `${BASE_URL}/attachments/${attachmentId}`,
          header: { Authorization: `Bearer ${token}` },
          success: (res) => resolve(res.statusCode === 200 ? res.tempFilePath : ''),
          fail: () => resolve('')
        })
      })
  )
}

export { BASE_URL }
