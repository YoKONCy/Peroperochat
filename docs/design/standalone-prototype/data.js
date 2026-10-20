(() => {
  // 角色资源和示例文案集中配置，组件不把主题、身份和业务状态绑定在一起。
  window.LocalDesignData = {
    character: { id: 'pero', name: 'Pero', avatar: 'assets/pero-avatar.jpg', sprite: 'assets/pero-standing.svg', room: 'assets/sunlit-room.png', greeting: '回来啦，今天过得怎么样？', acknowledgement: '听到啦，今天的事慢慢说，我陪着你。' },
    palettes: { sky: { name: '晴空蓝', accent: '#467fc5', wash: '#eaf4ff' }, sakura: { name: '樱花粉', accent: '#b85d85', wash: '#fff0f5' }, mint: { name: '薄荷青', accent: '#398b7b', wash: '#eaf8f2' }, violet: { name: '月夜紫 · 浅色', accent: '#8967ba', wash: '#f3edfc' } },
    pages: [{ id: 'room', title: '房间', note: '房间、角色与一句熟悉的问候' }, { id: 'chat', title: '对话', note: '展开聊天，保留一点房间的温度' }, { id: 'wardrobe', title: '衣柜', note: '服装缩略图与轻薄的半屏抽屉' }, { id: 'settings', title: '设置', note: '统一的玻璃层次与清楚的分组' }],
    outfits: [
      { id: 'maid', name: '日常女仆', texture: 'maid-costume.png' },
      { id: 'summer', name: '晴空夏日', texture: 'summer-dress-costume-blue.png' },
      { id: 'school', name: '蓝白校服', texture: 'school-costume.png' },
      { id: 'winter', name: '冬日暖意', texture: 'winter-costume-white.png' },
    ],
    messages: [{ id: 'm1', role: 'user', text: '今天有一点累。', time: '19:42' }, { id: 'm2', role: 'assistant', text: '那就先休息一会儿，我陪着你。', time: '19:42' }, { id: 'm3', role: 'user', text: '晚一点提醒我喝水。', time: '19:43' }, { id: 'm4', role: 'assistant', text: '好，记住啦。', time: '19:43' }],
    journals: [{ id: 'j1', date: '今天 · 19:43', title: '忙碌之后的一点休息', text: '今天聊起有一点疲惫。我们决定把节奏放慢，先歇一会儿，也别忘记喝水。' }, { id: 'j2', date: '昨天 · 21:10', title: '窗边的好天气', text: '说起了周末想去散步的地方。小小的期待，让普通的一天也有了不一样的颜色。' }],
  }
})()
