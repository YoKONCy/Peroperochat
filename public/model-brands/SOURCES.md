# 模型与服务商品牌图形

来源：Lobe Icons，静态资源包 `@lobehub/icons-static-svg@1.95.1`。以下 SVG 保留原始内容，随应用本地提供。

项目采用 MIT 许可，许可副本见 `LICENSE.lobe-icons.txt`。品牌图形属于相应品牌，用于标识模型或服务，不表示其对 PeroperoChat 的背书。

许可来源：[https://raw.githubusercontent.com/lobehub/lobe-icons/bcdb7c82792a854c2967f7328e11a30205ea584d/LICENSE](https://raw.githubusercontent.com/lobehub/lobe-icons/bcdb7c82792a854c2967f7328e11a30205ea584d/LICENSE)

核对来源：[Lobe Icons](https://github.com/lobehub/lobe-icons)、[Agnes 官方模型资产](https://huggingface.co/Agnes-AI/Agnes-3.0-Flash/tree/main/assets)、[OpenAI 品牌规范](https://openai.com/brand/)。

彩色图形使用原色；单色图形用遮罩呈现，采用当前界面的中性文字色。沿用陪伴界面的紧凑尺寸，不增加额外装饰底座。

Kimi、OpenRouter、Yi、LongCat 的小尺寸展示采用库提供的单色版本，以保证深浅背景上的辨识度；仅随应用提供当前映射使用的素材。

## 接入方式

- `src/utils/modelBrands.js` 统一管理图形、模型家族与服务商域名映射。
- `src/components/companion/BrandIcon.vue` 呈现图形；`ChoicePicker.vue` 通过图标插槽复用选择行为。模型输入框和模型列表使用同一组件。
- 模型按 `modelId` 识别，服务商按预设和显式 API 地址识别；二者独立。用户昵称不参与识别，未知模型或服务使用通用图标。
- 新品牌先添加本地 SVG 和来源记录，再补充映射；兼容协议类型不代表实际服务商品牌。

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
