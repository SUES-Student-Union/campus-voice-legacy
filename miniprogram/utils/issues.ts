import { mockIssues } from '../data/mock'
import { Issue } from '../types/issue'

const STORAGE_KEY = 'campus_voice_issues'

export function getIssues(): Issue[] {
  const saved = wx.getStorageSync(STORAGE_KEY) as Issue[] | string
  const localIssues = Array.isArray(saved) ? saved : []
  const localIds = localIssues.map((issue) => issue.id)
  return localIssues.concat(mockIssues.filter((issue) => !localIds.includes(issue.id)))
}

export function getIssueById(id: string): Issue | undefined {
  return getIssues().find((issue) => issue.id === id)
}

export function saveIssue(issue: Issue): void {
  const saved = wx.getStorageSync(STORAGE_KEY) as Issue[] | string
  const localIssues = Array.isArray(saved) ? saved : []
  const nextIssues = [issue].concat(localIssues.filter((item) => item.id !== issue.id))
  wx.setStorageSync(STORAGE_KEY, nextIssues)
}

export function updateIssueRating(id: string, rating: string): Issue | undefined {
  const issue = getIssueById(id)
  if (!issue) return undefined
  const updated = { ...issue, rating }
  saveIssue(updated)
  return updated
}

export function generateIssueId(): string {
  const now = new Date()
  const year = now.getFullYear().toString()
  const month = String(now.getMonth() + 1).padStart(2, '0')
  const day = String(now.getDate()).padStart(2, '0')
  const base = `CV${year}${month}${day}`
  const existing = getIssues().map((issue) => issue.id)
  let suffix = String(now.getTime()).slice(-4)
  let id = `${base}${suffix}`
  while (existing.includes(id)) {
    suffix = String(Number(suffix) + 1).padStart(4, '0')
    id = `${base}${suffix}`
  }
  return id
}

export function formatDateTime(date: Date): string {
  const year = date.getFullYear()
  const month = String(date.getMonth() + 1).padStart(2, '0')
  const day = String(date.getDate()).padStart(2, '0')
  const hour = String(date.getHours()).padStart(2, '0')
  const minute = String(date.getMinutes()).padStart(2, '0')
  return `${year}-${month}-${day} ${hour}:${minute}`
}
