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
        const messageArray = (Array.isArray(T.idleMessages) && T.idleMessages.length ? T.idleMessages : ["好久不见，日子过得好快呢……", "大坏蛋！你都多久没理人家了呀，嘤嘤嘤～", "嗨～快来逗我玩吧！", "拿小拳拳锤你胸口！"])
        showMessage(randomSelection(messageArray), 6000, 9) 
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
      if (!document.hidden) showMessage(T.visibilityBack || "哇，你终于回来了～", 6000, 9) 
    })
  })()

  ;(function welcomeMessage() {
    if (location.pathname === "/") {
      const T = window.WAIFU_TEXTS || {}
      const W = (T.welcome || {})
      const TR = (W.timeRanges || {})
      let text
      const now = new Date().getHours()
      if (now > 5 && now <= 7) text = TR.morningEarly || "早上好！一日之计在于晨，美好的一天就要开始了。"
      else if (now > 7 && now <= 11) text = TR.morning || "上午好！工作顺利嘛，不要久坐，多起来走动走动哦！"
      else if (now > 11 && now <= 13) text = TR.noon || "中午了，工作了一个上午，现在是午餐时间！"
      else if (now > 13 && now <= 17) text = TR.afternoon || "午后很容易犯困呢，今天的运动目标完成了吗？"
      else if (now > 17 && now <= 19) text = TR.eveningSunset || "傍晚了！窗外夕阳的景色很美丽呢，最美不过夕阳红～"
      else if (now > 19 && now <= 21) text = TR.night || "晚上好，今天过得怎么样？"
      else if (now > 21 && now <= 23) text = TR.lateNight || ["已经这么晚了呀，早点休息吧，晚安～", "深夜时要爱护眼睛呀！"]
      else text = TR.midnight || "你是夜猫子呀？这么晚还不睡觉，明天起的来嘛？"
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
          const areaTexts = [T[`click_${area}_01`], T[`click_${area}_02`]].filter(Boolean)
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
    
    if (useCDN) {
      if (isNaN(modelId) || modelId >= modelList.models.length) modelId = 0;
      const maxTextures = modelList.model_textures[modelId] || 1;
      modelTexturesId++;
      if (modelTexturesId >= maxTextures) modelTexturesId = 0;
      loadModel(modelId, modelTexturesId, (T.randTexturesSuccess || "我的新衣服好看嘛？"))
    }
    else { fetch(`${apiPath}rand_textures/?id=${modelId}-${modelTexturesId}`).then(response => response.json()).then(result => { if (result.textures.id === 1 && (modelTexturesId === 1 || modelTexturesId === 0)) showMessage((T.randTexturesNoClothes || "我还没有其他衣服呢！"), 4000, 10); else loadModel(modelId, result.textures.id, (T.randTexturesSuccess || "我的新衣服好看嘛？")) }) }
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
