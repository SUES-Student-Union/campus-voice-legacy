export interface ApiProgress {
  status: string
  operator_name: string
  note: string
  created_at: string
}

export interface ApiReply {
  id: string
  department_name: string | null
  content: string
  created_at: string
}

export interface ApiAttachment {
  id: string
  original_name: string
  size: number
  mime: string
  created_at: string
}

export interface ApiIssueListItem {
  id: string
  issue_no: string
  title: string
  category: string
  campus: string
  department_name: string | null
  status: string
  priority: string
  latest_progress: string
  created_at: string
  updated_at: string
}

export interface ApiIssueDetail {
  id: string
  issue_no: string
  kind: string
  category: string
  campus: string
  location: string
  title: string
  description: string
  status: string
  priority: string
  is_public: boolean
  public_will: boolean
  contact: string
  department_name: string | null
  latest_progress: string
  rating: string | null
  created_at: string
  updated_at: string
  progress: ApiProgress[]
  replies: ApiReply[]
  attachments: ApiAttachment[]
}

export interface ApiPublicCase {
  id: string
  category: string
  campus: string
  location: string
  title: string
  summary: string
  reply: string
  status: string
  department_name: string | null
  submitted_at: string
  completed_at: string | null
  progress: ApiProgress[]
}

export type StatusKey =
  | 'pending'
  | 'supplement'
  | 'accepted'
  | 'processing'
  | 'review'
  | 'completed'
  | 'closed'

const STATUS_KEY_MAP: Record<string, StatusKey> = {
  '待受理': 'pending',
  '待补充': 'supplement',
  '已受理': 'accepted',
  '部门处理中': 'processing',
  '待审核': 'review',
  '已办结': 'completed',
  '已关闭': 'closed'
}

export function statusKeyOf(status: string): StatusKey {
  return STATUS_KEY_MAP[status] || 'pending'
}

function pad(n: number): string {
  return String(n).padStart(2, '0')
}

export function formatDateTime(iso: string): string {
  const d = new Date(iso)
  if (isNaN(d.getTime())) return iso
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`
}

export function formatTimeShort(iso: string): string {
  const d = new Date(iso)
  if (isNaN(d.getTime())) return iso
  return `${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`
}

export function formatDate(iso: string): string {
  const d = new Date(iso)
  if (isNaN(d.getTime())) return iso
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`
}

// ---------- 页面展示模型 ----------

export interface TimelineItem {
  status: string
  time: string
  operator: string
  description: string
  current: boolean
}

export interface IssueListItem {
  id: string
  no: string
  title: string
  category: string
  campus: string
  department: string
  status: string
  statusKey: StatusKey
  priority: string
  latestProgress: string
  createdAt: string
  updatedAt: string
}

export function toIssueListItem(a: ApiIssueListItem): IssueListItem {
  return {
    id: a.id,
    no: a.issue_no,
    title: a.title,
    category: a.category,
    campus: a.campus,
    department: a.department_name || '待分派',
    status: a.status,
    statusKey: statusKeyOf(a.status),
    priority: a.priority || '中',
    latestProgress: a.latest_progress,
    createdAt: formatDateTime(a.created_at),
    updatedAt: formatDateTime(a.updated_at)
  }
}

export interface IssueDetail {
  id: string
  no: string
  title: string
  status: string
  statusKey: StatusKey
  latestProgress: string
  department: string
  reply: string
  type: string
  category: string
  campus: string
  location: string
  createdAt: string
  updatedAt: string
  isPublic: boolean
  description: string
  images: string[]
  rating: string
  progress: TimelineItem[]
}

export function toIssueDetail(a: ApiIssueDetail): IssueDetail {
  const lastReply = a.replies[a.replies.length - 1]
  const lastIndex = a.progress.length - 1
  return {
    id: a.id,
    no: a.issue_no,
    title: a.title,
    status: a.status,
    statusKey: statusKeyOf(a.status),
    latestProgress: a.latest_progress,
    department: a.department_name || '待分派',
    reply: lastReply ? lastReply.content : '',
    type: a.kind,
    category: a.category,
    campus: a.campus,
    location: a.location,
    createdAt: formatDateTime(a.created_at),
    updatedAt: formatDateTime(a.updated_at),
    isPublic: a.is_public,
    description: a.description,
    images: [],
    rating: a.rating || '',
    progress: a.progress.map((p, index) => ({
      status: p.status,
      time: formatTimeShort(p.created_at),
      operator: p.operator_name,
      description: p.note,
      current: index === lastIndex
    }))
  }
}

export interface PublicCaseItem {
  id: string
  category: string
  status: string
  statusKey: StatusKey
  title: string
  summary: string
  campus: string
  location: string
  submittedAt: string
  completedAt: string
  reply: string
  department: string
  progress: TimelineItem[]
}

export function toPublicCaseItem(a: ApiPublicCase): PublicCaseItem {
  const lastIndex = a.progress.length - 1
  return {
    id: a.id,
    category: a.category,
    status: a.status,
    statusKey: statusKeyOf(a.status),
    title: a.title,
    summary: a.summary,
    campus: a.campus,
    location: a.location,
    submittedAt: formatDate(a.submitted_at),
    completedAt: a.completed_at ? formatDate(a.completed_at) : '',
    reply: a.reply,
    department: a.department_name || '',
    progress: a.progress.map((p, index) => ({
      status: p.status,
      time: formatDate(p.created_at),
      operator: p.operator_name,
      description: p.note,
      current: index === lastIndex
    }))
  }
}

// ---------- 管理端模型 ----------

export interface ApiDashboard {
  month_total: number
  all_total: number
  completion_rate: number
  by_category: [string, number][]
  department_pending: [string, number][]
  avg_response_hours: number | null
  avg_completion_hours: number | null
  satisfaction_avg: number | null
}

export interface ApiReviewDraft {
  title: string
  location: string
  summary: string
  reply: string
}

export interface ApiAdminIssueDetail extends ApiIssueDetail {
  student_name: string | null
  deadline: string | null
  assignee_name: string | null
  review_draft: ApiReviewDraft | null
  review_published: boolean
}

export interface ApiAuditLog {
  id: string
  operator_name: string | null
  action: string
  target_type: string
  target_id: string | null
  before: string | null
  after: string | null
  detail: string
  ip: string | null
  created_at: string
}

export interface ApiDepartment {
  id: string
  name: string
  code: string
  description: string | null
}

export interface AdminIssueDetail extends IssueDetail {
  studentName: string
  deadline: string
  assigneeName: string
  reviewDraft: ApiReviewDraft | null
  reviewPublished: boolean
}

export function toAdminIssueDetail(a: ApiAdminIssueDetail): AdminIssueDetail {
  const base = toIssueDetail(a)
  return {
    ...base,
    studentName: a.student_name || '未知学生',
    deadline: a.deadline ? formatDateTime(a.deadline) : '',
    assigneeName: a.assignee_name || '',
    reviewDraft: a.review_draft,
    reviewPublished: a.review_published
  }
}

export interface AuditLogItem {
  time: string
  operator: string
  action: string
  target: string
  before: string
  after: string
  detail: string
}

export function toAuditLogItem(a: ApiAuditLog): AuditLogItem {
  return {
    time: formatDateTime(a.created_at),
    operator: a.operator_name || '系统',
    action: a.action,
    target: a.target_id || '',
    before: a.before || '',
    after: a.after || '',
    detail: a.detail
  }
}
