(() => {
  const roles = {
    pero: {
      id: 'pero', name: 'Pero', signature: 'Pero', avatar: 'assets/pero-avatar.jpg', cg: 'assets/pero-cg.png', cgPosition: '54% 25%',
      subtitle: '晴空与你，日常也闪闪发亮。', greeting: '回来啦，', greetingNext: '继续刚才的旅程吗？',
      introduction: '今天也一起，把小小的想法变成现实。', reply: '好呀！我先看看天气，再帮你选一条轻松的路线。',
      completion: '我整理好了一份轻松的一日游方案，包含交通、景点和美食推荐。你可以先看看这份计划～',
      final: '行程方案已经整理好啦！需要我再调整什么吗？', sampleReply: '听到啦！这段消息会留在当前的设计演示中。',
      empty: '还没有资料呢，之后的发现就收在这里吧。', recommendedPalette: 'sky', recommendedTheme: 'light',
    },
    nana: {
      id: 'nana', name: 'Nana', signature: 'Nana', avatar: 'assets/nana-avatar.jpg', cg: 'assets/nana-cg.png', cgPosition: '53% 27%',
      subtitle: '与你的日常，也有星月相伴。', greeting: '回来啦，', greetingNext: '今天想做点什么？',
      introduction: '有想法就说出来，本小姐陪你一起。', reply: '交给我吧。路线别排太满，留点时间慢慢逛。',
      completion: '已经整理好一份一日游计划，你可以先看看，有想调整的地方随时告诉我。',
      final: '行程已经整理完成了！希望能让你的周末更轻松愉快～', sampleReply: '嗯哼，听到了。这次先记在设计演示里吧。',
      empty: '这里还空着呢，要不要一起攒点新发现？', recommendedPalette: 'night', recommendedTheme: 'dark', stylePack: '星纹缎带',
    },
  };
  // 配色包只描述视觉变量，不携带业务逻辑或可执行脚本。
  const palettes = [
    { id: 'sky', name: '晴空蓝', light: ['#467fc5', '#eff8ff', '#96c7e8'], dark: ['#8bbcf0', '#171f2d', '#b9d8ed'], mood: '清透 · 元气' },
    { id: 'night', name: '月夜紫', light: ['#8966b7', '#f7f2ff', '#cb9dbb'], dark: ['#c2a7ef', '#1d1b2b', '#d79bb8'], mood: '幻想 · 俏皮' },
    { id: 'sakura', name: '樱花粉', light: ['#bb5c84', '#fff3f7', '#e6b4ca'], dark: ['#eea8c7', '#281d29', '#c98ba8'], mood: '甜美 · 温柔' },
    { id: 'mint', name: '薄荷青', light: ['#307e74', '#effaf6', '#8dcabb'], dark: ['#8dd9c4', '#192824', '#b5cec5'], mood: '轻盈 · 治愈' },
    { id: 'amber', name: '琥珀金', light: ['#9c7127', '#fff9eb', '#dfba75'], dark: ['#e5c17a', '#28231c', '#d0a86a'], mood: '温暖 · 活泼' },
    { id: 'crimson', name: '绯红', light: ['#b74a5e', '#fff2f4', '#d78b9b'], dark: ['#f19aab', '#2a1b24', '#d18399'], mood: '鲜明 · 利落' },
    { id: 'silver', name: '银雾', light: ['#5c738e', '#f0f5fa', '#b0c2d3'], dark: ['#b4c9df', '#1d232d', '#a7b6cc'], mood: '清冷 · 通透' },
    { id: 'peach', name: '桃橙', light: ['#b46643', '#fff5ee', '#e9b29d'], dark: ['#f1b396', '#2b211f', '#dc9c8e'], mood: '明快 · 亲近' },
  ];
  const files = [
    { id: 'trip', name: '周末出行计划.pdf', kind: 'PDF', size: '640 KB', updated: '今天 10:12', title: '轻松一点，出发吧。', subtitle: '周末一日旅行计划', tone: 'blue', icon: 'files' },
    { id: 'budget', name: '旅行预算.xlsx', kind: 'XLSX', size: '28 KB', updated: '今天 10:10', title: '留一点小预算', subtitle: '交通 · 餐饮 · 其他', tone: 'green', icon: 'tasks' },
    { id: 'list', name: '出行清单.md', kind: 'MD', size: '6 KB', updated: '昨天 18:32', title: '出门前，再看一眼', subtitle: '把期待装进口袋', tone: 'purple', icon: 'memory' },
  ];
  window.ProtoData = { roles, palettes, files, conversation: '周末去哪里玩', instance: '家庭 infOS' };
})();
