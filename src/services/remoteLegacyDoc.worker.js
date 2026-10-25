// 只读取 Word 97–2003 正文片段，不执行宏、对象、脚本或文档外部引用。
self.onmessage = ({ data }) => {
  try {
    const bytes = new Uint8Array(data), view = new DataView(data)
    const u16 = at => { if (at < 0 || at + 2 > bytes.length) throw new Error('文档结构损坏'); return view.getUint16(at, true) }
    const u32 = at => { if (at < 0 || at + 4 > bytes.length) throw new Error('文档结构损坏'); return view.getUint32(at, true) }
    if (bytes.length > 8 * 1024 * 1024 || u32(0) !== 0xe011cfd0 || u32(4) !== 0xe11ab1a1) throw new Error('此文件不是可速览的旧 Word 文档，请保存原件查看')
    const shift = u16(30), miniShift = u16(32)
    if (![9, 12].includes(shift) || miniShift !== 6 || u16(28) !== 0xfffe) throw new Error('此文档的存储格式暂不支持')
    const sectorSize = 1 << shift, maxSector = Math.floor(bytes.length / sectorSize) - 1
    const sector = id => { if (id > maxSector || id < 0) throw new Error('文档扇区无效'); return bytes.subarray((id + 1) * sectorSize, (id + 2) * sectorSize) }
    const ints = buffer => Array.from({ length: buffer.byteLength / 4 }, (_, index) => new DataView(buffer.buffer, buffer.byteOffset, buffer.byteLength).getUint32(index * 4, true))
    const fatIds = []
    for (let i = 0; i < 109; i++) { const id = u32(76 + i * 4); if (id < 0xfffffffa) fatIds.push(id) }
    let difat = u32(68); const seenDifat = new Set()
    for (let i = 0; i < Math.min(u32(72), 128); i++) {
      if (seenDifat.has(difat)) throw new Error('文档索引循环'); seenDifat.add(difat)
      const ids = ints(sector(difat)); difat = ids.pop(); fatIds.push(...ids.filter(id => id < 0xfffffffa))
    }
    if (fatIds.length > 2048 || u32(44) > fatIds.length) throw new Error('文档索引过大或不完整')
    const fat = fatIds.slice(0, u32(44)).flatMap(id => ints(sector(id)))
    function chain(start, table, read, unit, limit = 8 * 1024 * 1024) {
      const chunks = [], visited = new Set(); let id = start, length = 0
      while (id < 0xfffffffa) {
        if (visited.has(id) || id >= table.length || length + unit > limit) throw new Error('文档数据链损坏或过大')
        visited.add(id); chunks.push(read(id)); length += unit; id = table[id]
      }
      if (id !== 0xfffffffe && id !== 0xffffffff) throw new Error('文档数据链结束标记无效')
      const result = new Uint8Array(length); chunks.forEach((part, index) => result.set(part, index * unit)); return result
    }
    const directory = chain(u32(48), fat, sector, sectorSize), entries = []
    for (let i = 0; i + 128 <= directory.length; i += 128) {
      const item = new DataView(directory.buffer, i, 128), size = item.getUint16(64, true), type = directory[i + 66]
      if (![2, 5].includes(type)) continue
      if (size < 2 || size > 64 || item.getUint32(124, true)) throw new Error('文档目录无效')
      entries.push({ name: new TextDecoder('utf-16le').decode(directory.subarray(i, i + size - 2)), type, start: item.getUint32(116, true), size: item.getUint32(120, true) })
    }
    const root = entries.find(entry => entry.type === 5)
    if (!root) throw new Error('文档目录缺少根节点')
    const mini = root.size ? chain(root.start, fat, sector, sectorSize).subarray(0, root.size) : new Uint8Array()
    const miniFat = u32(64) ? ints(chain(u32(60), fat, sector, sectorSize)) : []
    function stream(name) {
      const item = entries.find(entry => entry.name === name)
      if (!item || item.size > 8 * 1024 * 1024) throw new Error('文档缺少正文或正文过大')
      if (!item.size) return new Uint8Array()
      if (item.size >= u32(56)) return chain(item.start, fat, sector, sectorSize).subarray(0, item.size)
      return chain(item.start, miniFat, id => { const start = id * 64; if (start + 64 > mini.length) throw new Error('文档小数据链无效'); return mini.subarray(start, start + 64) }, 64).subarray(0, item.size)
    }
    const word = stream('WordDocument'), fib = new DataView(word.buffer, word.byteOffset, word.length)
    if (word.length < 100 || fib.getUint16(0, true) !== 0xa5ec || fib.getUint16(2, true) < 0xc1) throw new Error('此旧版 Word 暂不支持正文速览，请保存原件')
    const flags = fib.getUint16(10, true)
    if (flags & 0x8100) throw new Error('此 Word 文档已加密，请保存后输入密码查看')
    const table = stream(flags & 0x200 ? '1Table' : '0Table')
    let pos = 32; const csw = fib.getUint16(pos, true); pos += 2 + csw * 2
    const cslw = fib.getUint16(pos, true), lw = pos + 2; pos += 2 + cslw * 4
    const count = fib.getUint16(pos, true); pos += 2
    if (count < 34 || pos + 34 * 8 > word.length || cslw < 4) throw new Error('文档正文索引不完整')
    const fc = fib.getUint32(pos + 33 * 8, true), length = fib.getUint32(pos + 33 * 8 + 4, true), mainChars = fib.getUint32(lw + 12, true)
    if (fc + length > table.length) throw new Error('文档正文片段超出范围')
    const piece = new DataView(table.buffer, table.byteOffset, table.length)
    let cursor = fc
    while (cursor < fc + length && table[cursor] === 1) cursor += 3 + piece.getUint16(cursor + 1, true)
    if (table[cursor] !== 2) throw new Error('文档没有可读取的正文片段')
    const pieceSize = piece.getUint32(cursor + 1, true), start = cursor + 5, pieces = (pieceSize - 4) / 12
    if (!Number.isInteger(pieces) || pieces > 20000 || start + pieceSize > table.length) throw new Error('文档正文片段无效或过多')
    const output = []; let total = 0
    for (let i = 0; i < pieces && total < 256 * 1024; i++) {
      const begin = piece.getUint32(start + i * 4, true), end = Math.min(mainChars, piece.getUint32(start + (i + 1) * 4, true))
      if (begin >= mainChars) break
      const chars = Math.min(end - begin, 256 * 1024 - total)
      if (chars < 0) throw new Error('文档字符范围无效')
      const location = piece.getUint32(start + (pieces + 1) * 4 + i * 8 + 2, true), compressed = !!(location & 0x40000000), at = (location & 0x3fffffff) / (compressed ? 2 : 1), size = chars * (compressed ? 1 : 2)
      if (!Number.isInteger(at) || at + size > word.length) throw new Error('文档字符范围超出正文')
      output.push(new TextDecoder(compressed ? 'windows-1252' : 'utf-16le').decode(word.subarray(at, at + size))); total += chars
    }
    // eslint-disable-next-line no-control-regex -- 旧 Word 正文使用控制字符标记单元格，需要转为制表符并移除其他控制字符。
    const text = output.join('').replace(/\r/g, '\n').replace(/\x07/g, '\t').replace(/[\x00-\x06\x08-\x0c\x0e-\x1f]/g, '').trim()
    if (!text) throw new Error('未找到可速览的正文，请保存原件查看')
    self.postMessage({ text })
  } catch (error) { self.postMessage({ error: error.message || '旧 Word 文档无法预览，请保存原件' }) }
}
