import test from 'node:test'
import assert from 'node:assert/strict'
import { readdirSync, readFileSync } from 'node:fs'
import { join, relative } from 'node:path'
import { fileURLToPath } from 'node:url'
import { parse, compileScript, compileTemplate } from 'vue/compiler-sfc'

const root = fileURLToPath(new URL('../src', import.meta.url))
function components(directory) {
  return readdirSync(directory, { withFileTypes: true }).flatMap(entry => {
    const path = join(directory, entry.name)
    return entry.isDirectory() ? components(path) : entry.name.endsWith('.vue') ? [path] : []
  })
}

// 只在内存中校验组件与模板，不启动 Vite、不生成应用构建产物。
for (const filename of components(root)) {
  test(`Vue 组件与模板可解析：${relative(root, filename)}`, () => {
    const { descriptor, errors } = parse(readFileSync(filename, 'utf8'), { filename })
    assert.deepEqual(errors, [])
    const script = descriptor.scriptSetup || descriptor.script ? compileScript(descriptor, { id: filename }) : null
    if (descriptor.template) {
      const result = compileTemplate({ source: descriptor.template.content, filename, id: filename, compilerOptions: { bindingMetadata: script?.bindings } })
      assert.deepEqual(result.errors, [])
    }
  })
}
