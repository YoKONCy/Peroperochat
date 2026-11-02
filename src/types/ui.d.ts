// 服务端动态字段由原生协议层校验，表单和展示组件保留可扩展的字段结构。
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type DataRecord = Record<string, any>
export interface ChoiceOption {
  value: string | number | boolean | null
  label: string
  shortLabel?: string
  description?: string
  disabled?: boolean
}
export interface FormField {
  key: string
  label?: string
  type: string
  help?: string
  rows?: number
  min?: number
  max?: number
  required?: boolean
  nullable?: boolean
  placeholder?: string
  choices?: (ChoiceOption | string)[]
  itemChoices?: ChoiceOption[]
  itemDefault?: unknown
  translateChoices?: boolean
  fields?: FormField[]
  dictionary?: boolean
  open?: boolean
}
export interface ToolItem { id: string; name: string; status: string; summary?: string }
export interface ChatMessage extends DataRecord {
  id: number
  role: string
  content: string | { type: string; text?: string; image_url?: { url: string } }[]
  reasoning?: string
  status?: string
  error?: string
  toolTimeline?: ToolItem[]
  usage?: DataRecord
}
export interface NamedRecord extends DataRecord { id?: string; name?: string; description?: string; tags?: string[] }
export interface ManagementCommand extends DataRecord { label: string; op: string; fields: FormField[]; defaults: DataRecord }
export interface ManagementSection extends DataRecord { id: string; label: string; fields?: FormField[]; keys?: Record<string, string> }
export interface BackgroundScope { kind: string; agentId?: string }
