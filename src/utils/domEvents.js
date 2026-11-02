// 模板事件先缩小原生元素类型，避免把其他事件目标当作表单控件。
/** @param {Event} event */
export function inputValue(event) {
  const target = event.target
  return target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement || target instanceof HTMLSelectElement ? target.value : ''
}
/** @param {Event} event */
export function inputChecked(event) { return event.target instanceof HTMLInputElement && event.target.checked }
/** @param {Event} event */
export function detailsOpen(event) { return event.target instanceof HTMLDetailsElement && event.target.open }
