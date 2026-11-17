const journalReading = (function reading(D) {
  const {avatar, button, esc, head, icon, state:s, tabs}=D
  const imagePath='art/willow-bridge.webp'
  const intro='<p>从<strong>东岸步道</strong>走到柳桥，树荫正好盖住半条路。到了桥头再决定，要不要绕去旧书店。</p>'
  const quote='<blockquote>想歇脚的时候，桥下有一张朝着河面的长椅。</blockquote>'
  const formats=['图文手记','路线对照','代码与公式']
  function illustration(){
    return `<figure class="sketch-figure"><button data-do="sheet:bridge" aria-label="放大柳桥速写"><img src="${imagePath}" width="800" height="534" alt="河边的柳桥，右侧是红篷旧书店"></button><figcaption>柳桥与旧书店 · 河畔速写</figcaption></figure>`
  }
  function excerpt(name){
    return `<div class="prose rich-prose"><h3>沿河走到柳桥</h3>${intro}${quote}${button('展开路线手记 '+icon('next'),'rich:'+name,'read-note')}</div>`
  }
  function journal(mode){
    const attachment=mode==='remote'?'<p class="source-link"><a href="#remote/resource/map" data-do="resource:map">河畔散步路线.md</a></p>':''
    return `<h3>沿河走到柳桥</h3>${intro}${illustration()}${quote}
      <h4>出门前</h4><ul><li>带上水和折叠伞。</li><li>喜欢拍照的话，<em>把相机也捎上</em>。</li></ul>${attachment}`
  }
  function routes(){
    return `<h3>今天想走多远？</h3><div class="route-table" tabindex="0" role="region" aria-label="步行路线对照">
      <table><caption>按每小时 4 公里估算</caption><thead><tr><th scope="col">走法</th><th scope="col">全程</th><th scope="col">步行</th></tr></thead><tbody>
      <tr><th scope="row">到桥折返</th><td>1.6 km</td><td>24 分钟</td></tr><tr><th scope="row">绕去书店</th><td>2.6 km</td><td>39 分钟</td></tr></tbody></table></div>
      <h4>到了桥头</h4><figure class="route-diagram" aria-label="从东岸步道走到柳桥，可以原路折返，也可以绕去旧书店"><span>东岸步道</span><b aria-hidden="true">↓</b><strong>柳桥</strong><div><span>原路折返</span><span>绕去旧书店</span></div></figure>
      <ol><li>沿东岸步道往北，走到石桥。</li><li>要去书店，就过桥往右，在红篷下停一停。</li></ol>`
  }
  const cardMarkup='<article>\n  <h4>柳桥折返</h4>\n  <p>1.6 公里 · 约 24 分钟</p>\n</article>'
  function calculation(){
    return `<h3>算算步行时间</h3><p><code>km</code> 是全程公里数。按每小时 4 公里计算，去书店这条线约走 <strong>39 分钟</strong>。</p>
      <div class="math-note"><math display="block" aria-label="二点六除以四，乘以六十，等于三十九分钟"><mrow><mfrac><mn>2.6</mn><mn>4</mn></mfrac><mo>×</mo><mn>60</mn><mo>=</mo><mn>39</mn><mtext> 分钟</mtext></mrow></math></div>
      <div class="code-note"><header><b>JavaScript</b><button data-do="richwrap" aria-pressed="${s.richWrap}">换行</button></header>
      <pre tabindex="0" class="${s.richWrap?'wrapped':''}"><code><span class="syntax-key">const</span> minutes = (km, speed = <span class="syntax-num">4</span>) =&gt;\n  Math.round(km / speed * <span class="syntax-num">60</span>)\n\nminutes(<span class="syntax-num">2.6</span>)  <span class="syntax-comment">// 39 分钟</span></code></pre></div>
      <div class="code-note"><header><b>HTML · 行程卡</b><button data-do="richsource" aria-pressed="${s.richSource}">${s.richSource?'看预览':'看源码'}</button></header>
      ${s.richSource?`<pre tabindex="0"><code>${esc(cardMarkup)}</code></pre>`:`<div class="trip-card">${cardMarkup}</div>`}</div>`
  }
  function page(mode,name){
    const tab=s.richTab
    const body=tab===formats[0]?journal(mode):tab===formats[1]?routes():calculation()
    return {header:head('河畔散步手记','',true),
      body:`<div class="reading-author">${avatar(name)}<b>${esc(name)}</b><span>会话消息</span></div>${tabs(formats,tab,'richtab:')}<article class="prose rich-prose reading-page">${body}</article>`}
  }
  return {page,excerpt,illustration,imagePath}
})(ThemeStudy)
