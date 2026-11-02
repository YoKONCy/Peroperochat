// 分区只决定入口与层级，所有操作继续使用现有服务和权限边界。
export const remoteManagementGroups = [
  { id: 'companions', title: '角色与陪伴', description: '角色资料、提示词与联系节奏', icon: 'people', tone: 'rose', items: [
    { id: 'agents', label: '角色资料', description: '称呼、人设与互动', icon: 'profile' },
    { id: 'prompts', label: '提示词', description: '编辑、整理与发布', icon: 'document' },
    { id: 'proactive', label: '主动陪伴', description: '联系的节奏', icon: 'heart' },
  ] },
  { id: 'intelligence', title: '模型与语音', description: '模型、连接、工具与声音', icon: 'model', tone: 'blue', items: [
    { id: 'models', label: '聊天模型', description: '选择模型与调整参数', icon: 'model' },
    { id: 'providers', label: '服务商连接', description: '地址、协议与密钥', icon: 'connection' },
    { id: 'assignments', label: '模型分工', description: '默认模型、角色覆盖与视觉', icon: 'route' },
    { id: 'voice', label: '语音', description: '识别与朗读', icon: 'audio' },
    { id: 'thread-tools', label: '工具权限', description: '当前会话可以使用的工具', icon: 'tools' },
  ] },
  { id: 'personal', title: '外观与偏好', description: '配色、背景和回答偏好', icon: 'palette', tone: 'violet', items: [
    { id: 'appearance', label: '外观与语言', description: '主题、背景与界面语言', icon: 'palette' },
    { id: 'user', label: '个人资料', description: '头像、称呼与回答偏好', icon: 'profile' },
    { id: 'prefill', label: '预填充模式', description: '回答开头与填充方式', icon: 'edit' },
  ] },
  { id: 'devices', title: '连接与设备', description: '配对、后台接收与运行状态', icon: 'connection', tone: 'mint', items: [
    { id: 'connections', label: '连接与模式', description: '切换设备与管理连接', icon: 'connection' },
    { id: 'usage', label: '用量统计', description: '累计用量', icon: 'chart' },
    { id: 'runtime', label: '运行概览', description: '当前在忙什么', icon: 'pulse' },
  ] },
]
