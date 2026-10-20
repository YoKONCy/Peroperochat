// 仅负责文件内容的展示分类，不执行服务端文件中的脚本、宏或外部引用。
export function documentKind(name, mime = '') {
  const extension = String(name || '').split('.').pop().toLowerCase()
  if (mime === 'application/pdf' || extension === 'pdf') return 'pdf'
  if (['docx', 'xlsx', 'pptx', 'zip'].includes(extension)) return extension
  const office = { 'application/vnd.openxmlformats-officedocument.wordprocessingml.document': 'docx', 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet': 'xlsx', 'application/vnd.openxmlformats-officedocument.presentationml.presentation': 'pptx', 'application/zip': 'zip' }
  if (office[mime]) return office[mime]
  if (extension === 'csv' || mime === 'text/csv') return 'csv'
  if (mime.startsWith('text/') || /^(application\/(json|xml|javascript|x-yaml)|image\/svg\+xml)$/.test(mime) || ['txt', 'md', 'json', 'xml', 'html', 'svg', 'yaml', 'yml', 'toml', 'log', 'js', 'ts', 'py', 'rs', 'css', 'sh', 'sql', 'ini'].includes(extension)) return 'text'
  if (/^image\/(png|jpeg|webp|gif|avif|bmp|x-icon)$/.test(mime)) return 'image'
  if (/^audio\//.test(mime)) return 'audio'
  if (/^video\//.test(mime)) return 'video'
  return 'binary'
}
export function formatFileSize(size) {
  if (!Number.isFinite(size)) return '大小未知'
  if (size < 1024) return `${size} B`
  const unit = size >= 1024 ** 3 ? 3 : size >= 1024 ** 2 ? 2 : 1
  return `${(size / 1024 ** unit).toFixed(1)} ${['B', 'KiB', 'MiB', 'GiB'][unit]}`
}
const elements = (node, local) => Array.from(node.getElementsByTagNameNS('*', local))
function xml(source) {
  if (!source || /<!DOCTYPE|<!ENTITY/i.test(source)) throw new Error('此文档包含不支持的 XML 声明')
  const doc = new DOMParser().parseFromString(source, 'application/xml')
  if (doc.getElementsByTagName('parsererror').length) throw new Error('文档内容损坏，无法预览')
  return doc
}
const attribute = (node, name) => Array.from(node.attributes).find(item => item.localName === name)?.value
const textRuns = node => elements(node, 't').map(item => item.textContent).join('')
const paragraphs = doc => elements(doc, 'p').slice(0, 2000).map(node => textRuns(node)).filter(Boolean).join('\n\n').slice(0, 256 * 1024)
export function officePreview(kind, files) {
  if (kind === 'docx') return [{ title: '正文', text: paragraphs(xml(files['word/document.xml'])) || '文档中没有可预览的正文' }]
  if (kind === 'pptx') return Object.keys(files).filter(name => /^ppt\/slides\/slide\d+\.xml$/.test(name)).sort((a, b) => a.localeCompare(b, undefined, { numeric: true })).map((name, i) => ({ title: `第 ${i + 1} 页`, text: paragraphs(xml(files[name])) || '此页没有可预览的文字' }))
  const shared = files['xl/sharedStrings.xml'] ? elements(xml(files['xl/sharedStrings.xml']), 'si').slice(0, 50000).map(textRuns) : []
  const names = files['xl/workbook.xml'] ? elements(xml(files['xl/workbook.xml']), 'sheet').map(node => ({ name: attribute(node, 'name'), id: attribute(node, 'id') })) : []
  const relations = files['xl/_rels/workbook.xml.rels'] ? elements(xml(files['xl/_rels/workbook.xml.rels']), 'Relationship') : []
  return Object.keys(files).filter(name => /^xl\/worksheets\/sheet\d+\.xml$/.test(name)).sort((a, b) => a.localeCompare(b, undefined, { numeric: true })).map((name, i) => {
    const rel = relations.find(node => `xl/${attribute(node, 'Target')?.replace(/^\/?xl\//, '')}` === name)
    const title = names.find(item => item.id === attribute(rel || { attributes: [] }, 'Id'))?.name || `工作表 ${i + 1}`
    const rows = elements(xml(files[name]), 'row').slice(0, 200).map(row => {
      const values = []
      for (const cell of elements(row, 'c')) {
        const letters = (attribute(cell, 'r') || '').match(/^[A-Z]+/)?.[0]
        const index = letters ? Array.from(letters).reduce((n, c) => n * 26 + c.charCodeAt(0) - 64, 0) - 1 : values.length
        if (index >= 30) continue
        const type = attribute(cell, 't'), value = elements(cell, 'v')[0]?.textContent || ''
        values[index] = String(type === 's' ? shared[Number(value)] || '' : type === 'inlineStr' ? textRuns(cell) : type === 'b' ? value === '1' ? '是' : '否' : value).slice(0, 4000)
      }
      return Array.from({ length: Math.min(values.length, 30) }, (_, index) => values[index] || '')
    })
    return { title, rows }
  })
}
export function csvPreview(text) {
  const rows = [], row = []; let value = '', quoted = false
  for (let i = 0; i < text.length && rows.length < 200; i++) {
    const char = text[i]
    if (char === '"') { if (quoted && text[i + 1] === '"') { value += '"'; i++ } else quoted = !quoted }
    else if (!quoted && (char === ',' || char === '\n')) {
      if (row.length < 30) row.push(value.replace(/\r$/, '').slice(0, 4000)); value = ''
      if (char === '\n') { rows.push(row.splice(0)); }
    } else if (value.length < 4000) value += char
  }
  if (rows.length < 200 && (row.length || value)) { if (row.length < 30) row.push(value.replace(/\r$/, '')); rows.push(row) }
  return [{ title: '表格', rows }]
}
