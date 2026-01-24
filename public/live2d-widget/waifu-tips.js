function loadWidget(config) {
  let { waifuPath, apiPath, cdnPath } = config
  let useCDN = false, modelList
  if (typeof cdnPath === "string") { useCDN = true; if (!cdnPath.endsWith("/")) cdnPath += "/" }
  else if (typeof apiPath === "string") { if (!apiPath.endsWith("/")) apiPath += "/" }
  else { console.error("Invalid initWidget argument!"); return }
  localStorage.removeItem("waifu-display")
  sessionStorage.removeItem("waifu-text")
  document.body.insertAdjacentHTML("beforeend", `<div id="waifu">
      <div id="waifu-tips"></div>
      <canvas id="live2d" width="800" height="800"></canvas>
      <div id="waifu-tool">
        <span class="fa fa-lg fa-comment"></span>
        <span class="fa fa-lg fa-paper-plane"></span>
        <span class="fa fa-lg fa-user-circle"></span>
        <span class="fa fa-lg fa-street-view"></span>
        <span class="fa fa-lg fa-camera-retro"></span>
        <span class="fa fa-lg fa-info-circle"></span>
        <span class="fa fa-lg fa-times"></span>
      </div>
    </div>`)
  setTimeout(() => { document.getElementById("waifu").style.bottom = 0 }, 0)

  function randomSelection(obj) { return Array.isArray(obj) ? obj[Math.floor(Math.random() * obj.length)] : obj }
  let userAction = false, userActionTimer, messageTimer
  window.addEventListener("mousemove", () => userAction = true)
  window.addEventListener("keydown", () => userAction = true)
  setInterval(() => {
    if (userAction) { userAction = false; clearInterval(userActionTimer); userActionTimer = null }
    else if (!userActionTimer) { 
      userActionTimer = setInterval(() => { 
        const T = window.WAIFU_TEXTS || {}
        const idleArray = []
        for (let i = 1; i <= 10; i++) {
          const key = `idleMessages_${String(i).padStart(2, '0')}`
          if (T[key]) idleArray.push(T[key])
          else break
        }
        
        // 动态回退逻辑：根据 agentId 选择默认台词
        if (idleArray.length === 0) {
            const agentId = localStorage.getItem('ppc.activeAgent') || 'pero'
            const defaultIdle = {
                pero: ["好久不见，日子过得好快呢……", "大坏蛋！你都多久没理人家了呀，嘤嘤嘤～", "嗨～快来逗我玩吧！", "拿小拳拳锤你胸口！"],
                nana: ["杂鱼主人死到哪里去了？怎么还不回来...呜...", "喂！你是不是把我忘了？真是无可救药的笨蛋！", "好无聊啊...这种时候要是有人来给我骂一下就好了...", "哈？你以为Nana在想你吗？别自作多情了，杂鱼！♡"]
            }
            const messageArray = defaultIdle[agentId] || defaultIdle.pero
            showMessage(randomSelection(messageArray), 6000, 9)
        } else {
            showMessage(randomSelection(idleArray), 6000, 9)
        }
      }, 120000) 
    }
  }, 1000)

  ;(function registerEventListener() {
    document.querySelector("#waifu-tool .fa-paper-plane").addEventListener("click", () => {
      if (window.Asteroids) { if (!window.ASTEROIDSPLAYERS) window.ASTEROIDSPLAYERS = []; window.ASTEROIDSPLAYERS.push(new Asteroids()) }
      else { const script = document.createElement("script"); script.src = "https://fastly.jsdelivr.net/gh/stevenjoezhang/asteroids/asteroids.js"; document.head.appendChild(script) }
    })
    document.querySelector("#waifu-tool .fa-user-circle").addEventListener("click", loadOtherModel)
    document.querySelector("#waifu-tool .fa-street-view").addEventListener("click", loadRandModel)
    document.querySelector("#waifu-tool .fa-camera-retro").addEventListener("click", () => { Live2D.captureName = "photo.png"; Live2D.captureFrame = true })
    document.querySelector("#waifu-tool .fa-info-circle").addEventListener("click", () => { open("https://github.com/stevenjoezhang/live2d-widget") })
    document.querySelector("#waifu-tool .fa-times").addEventListener("click", () => { localStorage.setItem("waifu-display", Date.now()); document.getElementById("waifu").style.bottom = "-500px"; setTimeout(() => { document.getElementById("waifu").style.display = "none"; const t = document.getElementById("waifu-toggle"); if (t) t.classList.add("waifu-toggle-active") }, 3000) })
    const devtools = () => {}
    console.log("%c", devtools)
    window.addEventListener("visibilitychange", () => { 
      const T = window.WAIFU_TEXTS || {}
      if (!document.hidden) {
        let agentId = localStorage.getItem('ppc.activeAgent') || 'pero'
        const allLines = window.WAIFU_INTERACTION_LINES
        let text = T.visibilityBack
        if (!text && allLines && allLines[agentId] && allLines[agentId].visibility) {
          text = randomSelection(allLines[agentId].visibility)
        }
        showMessage(text || "哇，你终于回来了～", 6000, 9) 
      }
    })
  })()

  ;(function welcomeMessage() {
    if (location.pathname === "/") {
      const T = window.WAIFU_TEXTS || {}
      const W = (T.welcome || {})
      const TR = (W.timeRanges || {})
      
      let agentId = localStorage.getItem('ppc.activeAgent') || 'pero'
      const allLines = window.WAIFU_INTERACTION_LINES
      const fallbackTR = (allLines && allLines[agentId] && allLines[agentId].welcome) || {}
      
      let text
      const now = new Date().getHours()
      if (now > 5 && now <= 7) text = TR.morningEarly || fallbackTR.morningEarly || "早上好！一日之计在于晨，美好的一天就要开始了。"
      else if (now > 7 && now <= 11) text = TR.morning || fallbackTR.morning || "上午好！工作顺利嘛，不要久坐，多起来走动走动哦！"
      else if (now > 11 && now <= 13) text = TR.noon || fallbackTR.noon || "中午了，工作了一个上午，现在是午餐时间！"
      else if (now > 13 && now <= 17) text = TR.afternoon || fallbackTR.afternoon || "午后很容易犯困呢，今天的运动目标完成了吗？"
      else if (now > 17 && now <= 19) text = TR.eveningSunset || fallbackTR.eveningSunset || "傍晚了！窗外夕阳的景色很美丽呢，最美不过夕阳红～"
      else if (now > 19 && now <= 21) text = TR.night || fallbackTR.night || "晚上好，今天过得怎么样？"
      else if (now > 21 && now <= 23) text = TR.lateNight || fallbackTR.lateNight || ["已经这么晚了呀，早点休息吧，晚安～", "深夜时要爱护眼睛呀！"]
      else text = TR.midnight || fallbackTR.midnight || "你是夜猫子呀？这么晚还不睡觉，明天起的来嘛？"
      
      showMessage(text, 7000, 8)
    }
  })()

  function showMessage(text, timeout, priority) {
    if (!text || (sessionStorage.getItem("waifu-text") && sessionStorage.getItem("waifu-text") > priority)) return
    if (messageTimer) { clearTimeout(messageTimer); messageTimer = null }
    text = randomSelection(text)
    
    // 触发自定义事件，允许外部（如 Vue）拦截消息显示
    window.dispatchEvent(new CustomEvent('waifu-message', { 
      detail: { text, timeout, priority } 
    }))

    sessionStorage.setItem("waifu-text", priority)
    const tips = document.getElementById("waifu-tips")
    if (!tips) return
    tips.innerHTML = text
    tips.classList.remove("waifu-tips-active")
    void tips.offsetWidth // trigger reflow
    tips.classList.add("waifu-tips-active")
    messageTimer = setTimeout(() => { sessionStorage.removeItem("waifu-text"); tips.classList.remove("waifu-tips-active") }, timeout)
  }

  // 监听角色切换事件并同步台词
  window.addEventListener("ppc:agent-switched", (e) => {
    const agentId = e.detail && e.detail.agentId
    if (agentId) {
        console.log(`[WaifuTips] Agent switched to: ${agentId}, reloading custom texts.`)
        
        // 1. 从本地存储重新加载该角色的自定义台词，实现真正的动态切换
        try {
          const saved = localStorage.getItem(`ppc.${agentId}.waifu.texts`)
          if (saved) {
            window.WAIFU_TEXTS = JSON.parse(saved)
          } else {
            window.WAIFU_TEXTS = {} // 如果没有自定义，重置为空，触发回退机制
          }
        } catch (err) {
          window.WAIFU_TEXTS = {}
        }

        // 2. 显示欢迎语
        const T = window.WAIFU_TEXTS || {}
        const welcome = (agentId === 'nana') ? "哈？杂鱼主人，你怎么又把我换回来了？" : "锵锵！Pero 酱回来啦！"
        showMessage(welcome, 4000, 10)
    }
  })

  ;(function initModel() {
    let modelId = localStorage.getItem("modelId"), modelTexturesId = localStorage.getItem("modelTexturesId")
    if (modelId === null) {
      modelId = 0; modelTexturesId = 0
    } else {
      modelId = parseInt(modelId); modelTexturesId = parseInt(modelTexturesId)
    }
    if (useCDN) {
      if (!modelList) {
        loadModelList().then(() => {
          if (isNaN(modelId) || modelId >= modelList.models.length) {
            modelId = 0; modelTexturesId = 0
          } else {
            const maxTextures = modelList.model_textures[modelId];
            if (isNaN(modelTexturesId) || modelTexturesId >= maxTextures) {
              modelTexturesId = 0
            }
          }
          loadModel(modelId, modelTexturesId)
        })
      } else {
        if (isNaN(modelId) || modelId >= modelList.models.length) {
          modelId = 0; modelTexturesId = 0
        }
        loadModel(modelId, modelTexturesId)
      }
    } else {
      if (isNaN(modelId)) { modelId = 1; modelTexturesId = 53 }
      loadModel(modelId, modelTexturesId)
    }
    window.addEventListener("click", event => {
      const C = window.WAIFU_CONFIG || { click: [] }
      for (let config of (C.click || [])) {
        if (!event.target.matches(config.selector)) continue;
        
        // --- CUSTOM INTERACTION LOGIC START ---
        // 拦截 Live2D 点击，使用自定义的部位判定和台词系统
        if (config.selector === '#live2d' || config.selector === '#waifu #live2d') {
             try {
                 const rect = event.target.getBoundingClientRect()
                 // 针对 1 头身 Live2D 优化后的判定比例
                   const relativeY = (event.clientY - rect.top) / rect.height
                   let area = 'body'
                   if (relativeY < 0.60) area = 'head'       // 头部范围大幅增大 (原 0.45 -> 0.60)
                   else if (relativeY < 0.80) area = 'chest' // 胸部范围下移 (原 0.65 -> 0.80)
                   
                   console.log(`[WaifuTips] Clicked area: ${area} (Y: ${relativeY.toFixed(2)})`)

                 let agentId = 'pero'
                 if (typeof window.WAIFU_GET_AGENT_ID === 'function') {
                     agentId = window.WAIFU_GET_AGENT_ID()
                 } else {
                     agentId = localStorage.getItem('ppc.activeAgent') || 'pero'
                 }

                 const allLines = window.WAIFU_INTERACTION_LINES
                 const T = window.WAIFU_TEXTS || {}
                 
                 // 1. 优先尝试从动态解析的 WAIFU_TEXTS 中获取该部位台词
                 if (area && T[`click_${area}_01`]) {
                     const areaTexts = []
                     for (let i = 1; i <= 20; i++) {
                         const key = `click_${area}_${String(i).padStart(2, '0')}`
                         if (T[key]) areaTexts.push(T[key])
                         else break
                     }
                     if (areaTexts.length > 0) {
                         showMessage(randomSelection(areaTexts), 4000, 9)
                         return
                     }
                 }

                 // 2. 尝试从动态解析的通用点击台词中获取 (click_messages_0x)
                 if (T.click_messages_01) {
                     const generalTexts = []
                     for (let i = 1; i <= 20; i++) {
                         const key = `click_messages_${String(i).padStart(2, '0')}`
                         if (T[key]) generalTexts.push(T[key])
                         else break
                     }
                     if (generalTexts.length > 0) {
                         showMessage(randomSelection(generalTexts), 4000, 9)
                         return
                     }
                 }

                 // 3. 回退到预定义的互动台词库
                 if (allLines) {
                     const roleLines = allLines[agentId] || allLines['pero']
                     const areaLines = roleLines[area] || roleLines['body']
                     if (areaLines && areaLines.length > 0) {
                         const text = Array.isArray(areaLines) ? areaLines[Math.floor(Math.random() * areaLines.length)] : areaLines
                         showMessage(text, 4000, 9)
                         return // 拦截成功，不再执行后续默认逻辑
                     }
                 }
             } catch (e) {
                 console.error('[WaifuTips] Interaction error:', e)
             }
        }
        // --- CUSTOM INTERACTION LOGIC END ---

        let t;
        if (Array.isArray(config.text)) {
          // Sequential playback
          if (config.nextIndex === undefined) config.nextIndex = 0;
          t = config.text[config.nextIndex];
          config.nextIndex = (config.nextIndex + 1) % config.text.length;
        } else {
          t = config.text;
        }
        
        t = String(t).replace("{text}", event.target.innerText);
        showMessage(t, 4000, 8);
        return
      }
    })

    // 监听来自 Vue 层的手动点击触发
    window.addEventListener("ppc:waifu-click", (e) => {
      const C = window.WAIFU_CONFIG || { click: [] }
      const area = e.detail && e.detail.area // 可能包含 'head', 'chest', 'body'
      
      const config = C.click.find(c => c.selector === "#live2d")
      if (config && config.text) {
        let t;
        const T = window.WAIFU_TEXTS || {}
        
        // 如果指定了部位，尝试从 WAIFU_TEXTS 中找对应部位的台词
        if (area && T[`click_${area}_01`]) {
          const areaTexts = []
          // 尝试读取多条台词，直到没有为止（假设最大 10 条）
          for (let i = 1; i <= 10; i++) {
            const key = `click_${area}_${String(i).padStart(2, '0')}`
            if (T[key]) areaTexts.push(T[key])
            else break
          }
          
          if (areaTexts.length > 0) {
            t = randomSelection(areaTexts)
          }
        }

        // 如果没找到部位台词，回退到通用逻辑
        if (!t) {
          if (Array.isArray(config.text)) {
            if (config.nextIndex === undefined) config.nextIndex = 0;
            t = config.text[config.nextIndex];
            config.nextIndex = (config.nextIndex + 1) % config.text.length;
          } else {
            t = config.text;
          }
        }
        
        showMessage(t, 4000, 8);
      }
    })
  })()

  async function loadModelList() { const response = await fetch(`${cdnPath}model_list.json`); modelList = await response.json() }

  async function ensureLoadFn() {
    if (typeof window.loadlive2d === "function") return true
    return await new Promise(resolve => {
      const start = Date.now()
      const timer = setInterval(() => {
        if (typeof window.loadlive2d === "function") { clearInterval(timer); resolve(true) }
        else if (Date.now() - start > 8000) { clearInterval(timer); resolve(false) }
      }, 60)
    })
  }

  async function loadModel(modelId, modelTexturesId, message) {
    localStorage.setItem("modelId", modelId)
    localStorage.setItem("modelTexturesId", modelTexturesId)
    showMessage(message, 4000, 10)
    const ok = await ensureLoadFn()
    if (!ok || typeof window.loadlive2d !== "function") { try { console.warn("loadlive2d unavailable") } catch (_) {}; return }
    if (useCDN) {
      if (!modelList) await loadModelList();
      const target = modelList.models[modelId];
      const indexName = modelTexturesId > 0 ? `index_${modelTexturesId}.json` : "index.json";
      window.loadlive2d("live2d", `${cdnPath}model/${target}/${indexName}`);
      console.log(`Live2D 模型 ${modelId} 纹理 ${modelTexturesId} 加载完成`);
    }
    else { window.loadlive2d("live2d", `${apiPath}get/?id=${modelId}-${modelTexturesId}`); console.log(`Live2D 模型 ${modelId}-${modelTexturesId} 加载完成`) }
  }

  async function loadRandModel() {
    const T = window.WAIFU_TEXTS || {}
    if (useCDN && !modelList) await loadModelList();
    
    let modelId = parseInt(localStorage.getItem("modelId") || "0")
    let modelTexturesId = parseInt(localStorage.getItem("modelTexturesId") || "0")
    
    // 获取当前角色的 fallback 文本
    let agentId = localStorage.getItem('ppc.activeAgent') || 'pero'
    const allLines = window.WAIFU_INTERACTION_LINES
    const fallbackOther = (allLines && allLines[agentId] && allLines[agentId].other) || {}
    const successText = T.randTexturesSuccess || fallbackOther.randTexturesSuccess || "我的新衣服好看嘛？"
    const failText = T.randTexturesNoClothes || fallbackOther.randTexturesNoClothes || "我还没有其他衣服呢！"
    
    if (useCDN) {
      if (isNaN(modelId) || modelId >= modelList.models.length) modelId = 0;
      const maxTextures = modelList.model_textures[modelId] || 1;
      modelTexturesId++;
      if (modelTexturesId >= maxTextures) modelTexturesId = 0;
      loadModel(modelId, modelTexturesId, successText)
    }
    else { fetch(`${apiPath}rand_textures/?id=${modelId}-${modelTexturesId}`).then(response => response.json()).then(result => { if (result.textures.id === 1 && (modelTexturesId === 1 || modelTexturesId === 0)) showMessage(failText, 4000, 10); else loadModel(modelId, result.textures.id, successText) }) }
  }

  async function loadOtherModel() {
    let modelId = parseInt(localStorage.getItem("modelId") || "0")
    if (useCDN) { 
      if (!modelList) await loadModelList(); 
      if (isNaN(modelId) || modelId >= modelList.models.length) modelId = 0;
      const index = (++modelId >= modelList.models.length) ? 0 : modelId; 
      loadModel(index, 0, modelList.messages[index]) 
    }
    else { fetch(`${apiPath}switch/?id=${modelId}`).then(response => response.json()).then(result => { loadModel(result.model.id, 0, result.model.message) }) }
  }
  window.WaifuWidget = { loadOtherModel, loadRandModel }
}

function initWidget(config, apiPath) {
  if (typeof config === "string") { config = { waifuPath: config, apiPath } }
  loadWidget(config)
}
