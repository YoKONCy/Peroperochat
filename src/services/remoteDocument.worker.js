import { unzipSync, strFromU8 } from 'fflate'

self.onmessage = ({ data: { bytes, kind } }) => {
  try {
    let count = 0, expanded = 0, selected = 0, truncated = false
    const entries = []
    const files = unzipSync(new Uint8Array(bytes), { filter(file) {
      if (++count > 10000 || file.name.length > 1024) throw new Error('压缩包目录过大，请下载后查看')
      if (entries.length < 600) entries.push({ name: file.name, size: file.originalSize, directory: file.name.endsWith('/') })
      const wanted = kind === 'docx' ? file.name === 'word/document.xml'
        : kind === 'xlsx' ? /^(xl\/(sharedStrings|workbook)\.xml|xl\/_rels\/workbook\.xml\.rels|xl\/worksheets\/sheet\d+\.xml)$/.test(file.name)
          : kind === 'pptx' ? /^ppt\/slides\/slide\d+\.xml$/.test(file.name)
            : ['odt', 'ods', 'odp'].includes(kind) ? file.name === 'content.xml'
              : kind === 'epub' ? /\.(xhtml|html|htm)$/i.test(file.name) : false
      if (!wanted) return false
      if (++selected > 50) { truncated = true; return false }
      expanded += file.originalSize
      if (!Number.isSafeInteger(file.originalSize) || file.originalSize < 0 || file.originalSize > 2 * 1024 * 1024 || file.size > 2 * 1024 * 1024 || expanded > 8 * 1024 * 1024) throw new Error('文档内容过大，请下载后查看')
      return true
    } })
    self.postMessage({ entries, count, truncated, files: Object.fromEntries(Object.entries(files).map(([name, bytes]) => [name, strFromU8(bytes)])) })
  } catch (error) { self.postMessage({ error: error.message || '文件损坏或格式不受支持' }) }
}
