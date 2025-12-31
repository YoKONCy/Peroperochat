;(function(){
  const fallback = {
    WAIFU_TEXTS: {
      idleMessages: [
        "主人～Pio在这儿等你拥抱呀！",
        "今天也要元气满满，小心被Pio的可爱击中喽！"
      ],
      visibilityBack: "嗷！主人回来啦～抱抱我！",
      welcome: {
        timeRanges: {
          morningEarly: "早安主人～Pio为你准备了一份元气早餐式灵感！",
          morning: "上午好～让我们像小太阳一样发光发热吧！",
          noon: "午休时间到～睡眠是最好的充电方式哦！",
          afternoon: "午后容易犯困～让Pio撒点萌把主人叫醒！",
          eveningSunset: "黄昏好美～把这份颜色存进灵感便签吧！",
          night: "晚上好～我来做你的小陪伴和灵感灯塔！",
          lateNight: ["夜深了，Pio要被主人温柔守护～", "主人困困？一起来深呼吸，然后去睡觉吧！"],
          midnight: "夜猫子模式启动～Pio守护你的创作！"
        }
      },
      randTexturesNoClothes: "诶～衣柜里还空空的呢，等Pio长大就有啦",
      randTexturesSuccess: "新衣服出场！快夸我可爱～我会更努力的"
    },
    WAIFU_CONFIG: {
      click: [
        { selector: "#live2d", text: [
          "不要戳啦～Pio会害羞的…但也有点开心就是啦…",
          "主人想要灵感吗？摸摸Pio的头，灵感说不定就来啦!",
          "贴贴！今天也要被主人温柔拥抱(*^▽^*)"
        ] }
      ]
    }
  }
  window.refreshWaifuTexts = async (newData) => {
    try {
      let v = newData
      if (!v) {
        const saved = localStorage.getItem('ppc.waifu.texts')
        if (saved) v = JSON.parse(saved)
        else {
          const r = await fetch('/live2d-widget/waifu-texts.json')
          v = await r.json()
        }
      }
      const ln = [v.lateNight_01, v.lateNight_02].filter(Boolean)
      window.WAIFU_TEXTS = {
        idleMessages: [v.idleMessages_01, v.idleMessages_02].filter(Boolean),
        visibilityBack: v.visibilityBack || fallback.WAIFU_TEXTS.visibilityBack,
        welcome: {
          timeRanges: {
            morningEarly: v.welcome_timeRanges_morningEarly || fallback.WAIFU_TEXTS.welcome.timeRanges.morningEarly,
            morning: v.welcome_timeRanges_morning || fallback.WAIFU_TEXTS.welcome.timeRanges.morning,
            noon: v.welcome_timeRanges_noon || fallback.WAIFU_TEXTS.welcome.timeRanges.noon,
            afternoon: v.welcome_timeRanges_afternoon || fallback.WAIFU_TEXTS.welcome.timeRanges.afternoon,
            eveningSunset: v.welcome_timeRanges_eveningSunset || fallback.WAIFU_TEXTS.welcome.timeRanges.eveningSunset,
            night: v.welcome_timeRanges_night || fallback.WAIFU_TEXTS.welcome.timeRanges.night,
            lateNight: ln.length ? ln : fallback.WAIFU_TEXTS.welcome.timeRanges.lateNight,
            midnight: v.welcome_timeRanges_midnight || fallback.WAIFU_TEXTS.welcome.timeRanges.midnight
          }
        },
        randTexturesNoClothes: v.randTexturesNoClothes || fallback.WAIFU_TEXTS.randTexturesNoClothes,
        randTexturesSuccess: v.randTexturesSuccess || fallback.WAIFU_TEXTS.randTexturesSuccess
      }
      window.WAIFU_CONFIG = {
        click: [
          { selector: '#live2d', text: [
            v.click_messages_01, v.click_messages_02, v.click_messages_03
          ].filter(Boolean) }
        ]
      }
    } catch (_) {
      window.WAIFU_TEXTS = fallback.WAIFU_TEXTS
      window.WAIFU_CONFIG = fallback.WAIFU_CONFIG
    }
  }

  window.addEventListener('ppc:waifu-texts-updated', (e) => {
    window.refreshWaifuTexts(e.detail)
  })

  window.__waifuTextsReady = window.refreshWaifuTexts()
})()
