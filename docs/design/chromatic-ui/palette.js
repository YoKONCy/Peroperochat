// 主色延续现有八套配色；辅助色在同一处定义，组件只读取用途令牌。
(function (root) {
  const packs = [
    ['sky', '晴空蓝', '#467fc5', '#8bbcf0'], ['night', '月夜紫', '#8966b7', '#c2a7ef'],
    ['sakura', '樱花粉', '#bb5c84', '#eea8c7'], ['mint', '薄荷青', '#307e74', '#8dd9c4'],
    ['amber', '琥珀金', '#9c7127', '#e5c17a'], ['crimson', '绯红', '#b74a5e', '#f19aab'],
    ['silver', '银雾', '#5c738e', '#b4c9df'], ['peach', '桃橙', '#b46643', '#f1b396'],
  ];
  function mix(a, b, weight) {
    const color = value => value.match(/[a-f\d]{2}/gi).map(part => parseInt(part, 16));
    return '#' + color(a).map((v, i) => Math.round(v * weight + color(b)[i] * (1 - weight)).toString(16).padStart(2, '0')).join('');
  }
  function palette(id, dark) {
    const pack = packs.find(item => item[0] === id) || packs[0];
    const paper = dark ? '#302d3d' : '#ffffff', ink = dark ? '#f2edf7' : '#36354c';
    const accent = pack[dark ? 3 : 2];
    const result = { paper, ink, accent, on: dark ? '#252231' : '#ffffff', page: dark ? '#211f2d' : '#f5f4f9', muted: dark ? '#b3aabe' : '#817e94', line: dark ? '#53495f' : '#e3deec', shadow: dark ? '#15131f' : '#ded9e8' };
    const seeds = dark ? { primary: accent, rose: '#f1a4ba', mint: '#8cd2b9', violet: '#c2abed', amber: '#edc182', blue: '#97c3ee' } : { primary: accent, rose: '#b8587d', mint: '#307f70', violet: '#8664b1', amber: '#a17536', blue: '#4d7fae' };
    const lights = { primary: '#e6effb', rose: '#fbe3eb', mint: '#e0f3eb', violet: '#eee5fc', amber: '#fff0d7', blue: '#e2effa' };
    for (const [key, value] of Object.entries(seeds)) {
      result[key] = key === 'primary' ? accent : mix(value, accent, .92);
      result[key + 'Soft'] = dark ? mix(result[key], paper, .2) : key === 'primary' ? mix(accent, paper, .13) : mix(lights[key], accent, .97);
      result[key + 'Edge'] = mix(result[key], paper, dark ? .5 : .3);
    }
    result.raised = dark ? '#3c354c' : '#fffdfd';
    return result;
  }
  const roles = [ ['primary', '当前选择', '主要操作'], ['rose', '一起聊天', '群聊与角色'], ['mint', '正在推进', '任务与完成'], ['violet', '慢慢积累', '记忆与思考'], ['amber', '需要留意', '待确认与提醒'], ['blue', '连接与文件', '模型与资料'] ];
  const api = { packs, palette, roles, mix };
  root.Chromatic = api;
  if (typeof module !== 'undefined') module.exports = api;
})(globalThis);
