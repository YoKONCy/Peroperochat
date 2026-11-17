// 消息在文档流中按顺序排列，二分查找首条可见消息，减少滚动时的逐条坐标读取。
export function findReadingAnchor(list, selector) {
  const rows = list.querySelectorAll(selector)
  const top = list.getBoundingClientRect().top
  let low = 0
  let high = rows.length
  while (low < high) {
    const middle = (low + high) >>> 1
    if (rows[middle].getBoundingClientRect().bottom <= top + 4) low = middle + 1
    else high = middle
  }
  const element = rows[low]
  return element instanceof HTMLElement ? { element, offset: element.getBoundingClientRect().top - top } : null
}
