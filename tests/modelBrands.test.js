import test from 'node:test'
import assert from 'node:assert/strict'
import { existsSync } from 'node:fs'
import { MODEL_BRANDS, resolveModelBrand, resolveProviderBrand } from '../src/utils/modelBrands.js'

test('代理服务与模型品牌独立识别，兼容协议不代表服务商', () => {
  assert.equal(resolveProviderBrand('openai', 'https://proxy.example.com/v1'), undefined)
  assert.equal(resolveProviderBrand('custom', 'https://api.siliconflow.cn/v1'), 'siliconflow')
  assert.equal(resolveModelBrand('anthropic/claude-sonnet-4-5'), 'claude')
  assert.equal(resolveModelBrand('Qwen/Qwen3-32B'), 'qwen')
  assert.equal(resolveModelBrand('us.anthropic.claude-3-7-sonnet-20250219-v1:0'), 'claude')
  assert.equal(resolveModelBrand('nvidia/Llama-3.1-Nemotron-70B-Instruct'), 'nvidia')
})

test('服务商只匹配真实主机，拒绝伪装域名、路径和非网络地址', () => {
  for (const base of ['https://api.openai.com.attacker.example/v1', 'https://example.com/openai.com', 'https://openai.com@example.com', 'file:///openai.com', 'javascript:openai.com', 'https://']) {
    assert.equal(resolveProviderBrand('openai', base), undefined, base)
  }
  assert.equal(resolveProviderBrand('custom', 'https://API.OPENAI.COM./v1'), 'openai')
  assert.equal(resolveProviderBrand('anthropic'), 'anthropic')
})

test('未知值与空值保留通用图标，不从不完整模型名称误猜品牌', () => {
  for (const id of ['', '模型昵称', 'gptish', 'claudette', 'my-qwen-service', undefined]) assert.equal(resolveModelBrand(id), undefined)
  assert.equal(resolveProviderBrand('__proto__'), undefined)
  assert.equal(resolveProviderBrand('custom', 'http://localhost:11434/v1'), undefined)
  assert.equal(resolveProviderBrand('ollama', 'http://localhost:11434/v1'), 'ollama')
})

test('所有品牌映射都使用随应用提供的本地 SVG，并附带许可', () => {
  for (const { asset } of Object.values(MODEL_BRANDS)) {
    assert.match(asset, /^[a-z0-9-]+\.svg$/)
    assert.ok(existsSync(new URL(`../public/model-brands/${asset}`, import.meta.url)), asset)
  }
  assert.ok(existsSync(new URL('../public/model-brands/LICENSE.lobe-icons.txt', import.meta.url)))
})
