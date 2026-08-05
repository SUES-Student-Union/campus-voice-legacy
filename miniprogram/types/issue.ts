export type IssueStatusKey =
  | 'pending'
  | 'supplement'
  | 'accepted'
  | 'processing'
  | 'review'
  | 'completed'
  | 'closed'

export interface IssueProgress {
  status: string
  time: string
  operator: string
  description: string
  current?: boolean
}

export interface Issue {
  id: string
  type: string
  category: string
  campus: string
  location: string
  title: string
  description: string
  images: string[]
  status: string
  statusKey: IssueStatusKey
  createdAt: string
  updatedAt: string
  isPublic: boolean
  contact: string
  department: string
  requestedDepartment?: string
  latestProgress: string
  progress: IssueProgress[]
  reply?: string
  rating?: string
}

export interface PublicCase {
  id: string
  category: string
  campus: string
  location: string
  title: string
  summary: string
  status: string
  statusKey: IssueStatusKey
  submittedAt: string
  completedAt?: string
  department: string
  reply: string
  progress: IssueProgress[]
}
