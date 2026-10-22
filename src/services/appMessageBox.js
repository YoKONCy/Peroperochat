import { ElMessageBox } from 'element-plus'

// 确认框挂到应用主题根节点，继续使用原有的焦点管理与确认、取消语义。
// 不挂到抽屉内，避免与抽屉的键盘焦点范围互相抢占。
function options(value) {
  const root = document.querySelector('.app[data-ui="soft"]')
  return { ...value, ...(root ? { appendTo: root } : {}) }
}
export const appMessageBox = {
  confirm: (message, title, value) => ElMessageBox.confirm(message, title, options(value)),
  prompt: (message, title, value) => ElMessageBox.prompt(message, title, options(value)),
}
