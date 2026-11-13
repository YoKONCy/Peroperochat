# 模型与服务商品牌图标

本目录内置来自 Lobe Icons（`@lobehub/icons-static-svg@1.95.1`）的原始 SVG 图标，按 MIT 协议随应用本地分发，许可副本见 [LICENSE.lobe-icons.txt](LICENSE.lobe-icons.txt)（[上游 LICENSE](https://raw.githubusercontent.com/lobehub/lobe-icons/bcdb7c82792a854c2967f7328e11a30205ea584d/LICENSE)）。各品牌商标与图形归对应版权方所有，仅作模型及服务商识别用途。

核对参考：[Lobe Icons](https://github.com/lobehub/lobe-icons)、[Agnes 官方模型资产](https://huggingface.co/Agnes-AI/Agnes-3.0-Flash/tree/main/assets)、[OpenAI 品牌规范](https://openai.com/brand/)。

## 展示与接入规范

- **渲染策略**：彩色图标直接渲染原色；单色图标（含小尺寸优化的 Kimi、OpenRouter、Yi、LongCat 等）使用 CSS 遮罩跟随当前主题中性文字色，保持统一尺寸而不额外叠加装饰底座。
- **代码入口**：`src/utils/modelBrands.js` 维护图标、模型家族与服务商域名映射；`src/components/companion/BrandIcon.vue` 与 `src/components/companion/ChoicePicker.vue` 统一在输入框和选项列表渲染图标。
- **匹配规则**：模型按 `modelId` 匹配，服务商按预设与显式 API 域名匹配（兼容协议类型不等同于具体品牌），二者互不干扰；未知品牌统一回退为通用图标。新增品牌需先在本目录补齐本地 SVG 与来源记录后再添加映射。

## 资源清单

- `openai.svg`：[https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/openai.svg](https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/openai.svg)
- `anthropic.svg`：[https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/anthropic.svg](https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/anthropic.svg)
- `google-color.svg`：[https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/google-color.svg](https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/google-color.svg)
- `gemini-color.svg`：[https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/gemini-color.svg](https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/gemini-color.svg)
- `deepseek-color.svg`：[https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/deepseek-color.svg](https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/deepseek-color.svg)
- `zhipu-color.svg`：[https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/zhipu-color.svg](https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/zhipu-color.svg)
- `zai.svg`：[https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/zai.svg](https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/zai.svg)
- `moonshot.svg`：[https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/moonshot.svg](https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/moonshot.svg)
- `minimax-color.svg`：[https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/minimax-color.svg](https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/minimax-color.svg)
- `grok.svg`：[https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/grok.svg](https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/grok.svg)
- `xai.svg`：[https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/xai.svg](https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/xai.svg)
- `agnesai.svg`：[https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/agnesai.svg](https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/agnesai.svg)
- `qwen-color.svg`：[https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/qwen-color.svg](https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/qwen-color.svg)
- `doubao-color.svg`：[https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/doubao-color.svg](https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/doubao-color.svg)
- `hunyuan-color.svg`：[https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/hunyuan-color.svg](https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/hunyuan-color.svg)
- `mistral-color.svg`：[https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/mistral-color.svg](https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/mistral-color.svg)
- `groq.svg`：[https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/groq.svg](https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/groq.svg)
- `siliconcloud-color.svg`：[https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/siliconcloud-color.svg](https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/siliconcloud-color.svg)
- `bailian-color.svg`：[https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/bailian-color.svg](https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/bailian-color.svg)
- `volcengine-color.svg`：[https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/volcengine-color.svg](https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/volcengine-color.svg)
- `ollama.svg`：[https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/ollama.svg](https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/ollama.svg)
- `stepfun-color.svg`：[https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/stepfun-color.svg](https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/stepfun-color.svg)
- `meta-color.svg`：[https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/meta-color.svg](https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/meta-color.svg)
- `gemma-color.svg`：[https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/gemma-color.svg](https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/gemma-color.svg)
- `cohere-color.svg`：[https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/cohere-color.svg](https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/cohere-color.svg)
- `baai.svg`：[https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/baai.svg](https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/baai.svg)
- `jina.svg`：[https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/jina.svg](https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/jina.svg)
- `nvidia-color.svg`：[https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/nvidia-color.svg](https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/nvidia-color.svg)
- `baichuan-color.svg`：[https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/baichuan-color.svg](https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/baichuan-color.svg)
- `internlm-color.svg`：[https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/internlm-color.svg](https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/internlm-color.svg)
- `wenxin-color.svg`：[https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/wenxin-color.svg](https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/wenxin-color.svg)
- `spark-color.svg`：[https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/spark-color.svg](https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/spark-color.svg)
- `claude-color.svg`：[https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/claude-color.svg](https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/claude-color.svg)
- `kimi.svg`：[https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/kimi.svg](https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/kimi.svg)
- `openrouter.svg`：[https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/openrouter.svg](https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/openrouter.svg)
- `yi.svg`：[https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/yi.svg](https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/yi.svg)
- `longcat.svg`：[https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/longcat.svg](https://unpkg.com/@lobehub/icons-static-svg@1.95.1/icons/longcat.svg)
