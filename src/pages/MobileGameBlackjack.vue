<template>
  <div class="blackjack-container">
    <div class="header">
      <div class="back-btn" @click="goHome">
        <i class="fas fa-chevron-left"></i>
      </div>
      <div class="header-title">
        <h2>21 点</h2>
        <span>BLACKJACK</span>
      </div>
      <div class="placeholder"></div>
    </div>

    <!-- 游戏区域 -->
    <div class="game-area">
      <!-- 庄家 (AI) 区域 -->
      <div class="dealer-area">
        <div class="agent-info">
          <div class="agent-name">{{ agentName }}</div>
        </div>
        <div class="cards-row">
          <div 
            v-for="(card, index) in dealerHand" 
            :key="index" 
            :class="['poker-card', { 'card-hidden': card.hidden }]"
          >
            <span v-if="!card.hidden">{{ getCardDisplay(card) }}</span>
            <span v-else>?</span>
          </div>
        </div>
        <div class="score-tag" v-if="gameState !== 'playing'">{{ dealerScore }}</div>
      </div>

      <!-- 游戏状态提示 -->
      <div class="game-status-message" v-if="statusMessage">
        {{ statusMessage }}
      </div>

      <!-- 玩家区域 -->
      <div class="player-area">
        <div class="score-tag">{{ playerScore }}</div>
        <div class="cards-row">
          <div 
            v-for="(card, index) in playerHand" 
            :key="index" 
            class="poker-card"
          >
            {{ getCardDisplay(card) }}
          </div>
        </div>
        <div class="player-label">YOU</div>
      </div>
    </div>

    <!-- 结算气泡 -->
    <Transition name="bounce">
      <div class="settlement-bubble" v-if="settlementText" @click="closeSettlement">
        <div class="bubble-content">
          <p>{{ settlementText }}</p>
        </div>
      </div>
    </Transition>

    <!-- 底部操作栏 -->
    <div class="action-bar">
      <template v-if="gameState === 'ready'">
        <button class="action-btn primary" @click="startGame">开始发牌</button>
      </template>
      
      <template v-if="gameState === 'playing'">
        <button class="action-btn hit" @click="hit">要牌 (Hit)</button>
        <button class="action-btn stand" @click="stand">停牌 (Stand)</button>
      </template>

      <template v-if="gameState === 'finished'">
        <button class="action-btn primary" @click="resetGame">再来一局</button>
      </template>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { AGENTS, getActiveAgentId, chat } from '../api'

const router = useRouter()
const activeAgentId = ref(getActiveAgentId())
const agentName = computed(() => AGENTS[activeAgentId.value]?.name || 'Pero')

// 游戏状态: ready, playing, finished
const gameState = ref('ready')
const deck = ref([])
const playerHand = ref([])
const dealerHand = ref([])
const statusMessage = ref('')
const settlementText = ref('')

// 牌面显示映射
const suits = ['♠', '♥', '♣', '♦']
const values = ['A', '2', '3', '4', '5', '6', '7', '8', '9', '10', 'J', 'Q', 'K']

function createDeck() {
  const d = []
  for (const suit of suits) {
    for (const value of values) {
      d.push({ suit, value, hidden: false })
    }
  }
  return d.sort(() => Math.random() - 0.5)
}

function getCardValue(card) {
  if (['J', 'Q', 'K'].includes(card.value)) return 10
  if (card.value === 'A') return 11
  return parseInt(card.value)
}

function calculateScore(hand) {
  let score = 0
  let aces = 0
  for (const card of hand) {
    if (card.hidden) continue
    score += getCardValue(card)
    if (card.value === 'A') aces += 1
  }
  while (score > 21 && aces > 0) {
    score -= 10
    aces -= 1
  }
  return score
}

const playerScore = computed(() => calculateScore(playerHand.value))
const dealerScore = computed(() => calculateScore(dealerHand.value))

function getCardDisplay(card) {
  return `${card.suit}${card.value}`
}

function goHome() {
  router.push('/')
}

function startGame() {
  deck.value = createDeck()
  playerHand.value = [deck.value.pop(), deck.value.pop()]
  dealerHand.value = [deck.value.pop(), { ...deck.value.pop(), hidden: true }]
  gameState.value = 'playing'
  statusMessage.value = ''
  settlementText.value = ''
  
  // 检查开局 BlackJack
  if (playerScore.value === 21) {
    stand()
  }
}

function hit() {
  playerHand.value.push(deck.value.pop())
  if (playerScore.value > 21) {
    endGame('bust')
  }
}

async function stand() {
  // 庄家回合
  dealerHand.value[1].hidden = false // 翻开底牌
  
  while (dealerScore.value < 17) {
    dealerHand.value.push(deck.value.pop())
    await new Promise(r => setTimeout(r, 500)) // 模拟发牌延迟
  }

  if (dealerScore.value > 21) {
    endGame('win', '庄家爆牌了！')
  } else if (dealerScore.value > playerScore.value) {
    endGame('lose')
  } else if (dealerScore.value < playerScore.value) {
    endGame('win')
  } else {
    endGame('draw')
  }
}

async function endGame(result, reason = '') {
  gameState.value = 'finished'
  let prompt = ''
  
  if (result === 'bust') {
    statusMessage.value = '你爆牌了！(>_<)'
    prompt = `[System]: 用户在21点游戏中爆牌输了（点数 ${playerScore.value}）。请用${agentName.value}的语气嘲讽或安慰用户。`
  } else if (result === 'win') {
    statusMessage.value = reason || '你赢了！🎉'
    prompt = `[System]: 用户在21点游戏中赢了（用户 ${playerScore.value} vs 你 ${dealerScore.value}）。请用${agentName.value}的语气表示不甘心或夸奖用户。`
  } else if (result === 'lose') {
    statusMessage.value = '你输了...'
    prompt = `[System]: 用户在21点游戏中输了（用户 ${playerScore.value} vs 你 ${dealerScore.value}）。请用${agentName.value}的语气通过这次胜利来调侃用户。`
  } else {
    statusMessage.value = '平局'
    prompt = `[System]: 21点游戏平局（${playerScore.value}）。请用${agentName.value}的语气评价这场势均力敌的对决。`
  }

  // 触发 AI 点评
  try {
    const agentNameVal = agentName.value
    const lsGet = (key, fallback) => {
      try { const v = localStorage.getItem(key); if (v===null||v===undefined) return fallback; try { return JSON.parse(v) } catch(_) { return v } } catch(_) { return fallback }
    }
    const lsSet = (key, value) => {
      try { const v = typeof value === 'string' ? value : JSON.stringify(value); localStorage.setItem(key, v) } catch(_) {}
    }
    const getAgentStoreKey = (type) => `ppc.${activeAgentId.value}.${type}`

    // 模拟用户输入，触发主对话逻辑
    const gameResultMsg = `【管理系统提醒：${agentNameVal}，用户刚刚和你玩了一局21点。结果是：${statusMessage.value}（用户点数：${playerScore.value}，你的点数：${dealerScore.value}）。请以此为契机，用你一贯的语气和用户聊两句。】`
    
    // 获取当前消息列表
    const savedMessages = lsGet(getAgentStoreKey('messages'), [])
    const now = Date.now()
    
    // 检查最后一条消息是否已经是这个提醒，避免重复触发
    const lastMsg = savedMessages[savedMessages.length - 1]
    if (lastMsg && lastMsg.content === gameResultMsg) return

    const userMsg = { role: 'user', content: gameResultMsg, timestamp: now }
    
    // 我们不需要在这里等待回复，因为主对话逻辑是在 MobileHome 中处理的
    // 但为了让用户在回到主页时能看到回复，我们需要更新存储并触发事件
    const newMessages = [...savedMessages, userMsg]
    lsSet(getAgentStoreKey('messages'), newMessages)
    
    // 发送全局事件，通知 MobileHome 有新消息需要处理
    window.dispatchEvent(new CustomEvent('ppc:trigger-chat', { 
      detail: { 
        systemMsg: gameResultMsg,
        agentId: activeAgentId.value
      } 
    }))

    // 监听 AI 的回复
    const handleAiReply = (event) => {
      const reply = event.detail
      if (reply && !reply.includes('正在思考中')) {
        settlementText.value = reply
        window.removeEventListener('ppc:chat', handleAiReply)
      }
    }
    window.addEventListener('ppc:chat', handleAiReply)

    // 为了在游戏界面也有即时反馈，我们可以展示一个简单的提示
    settlementText.value = `${agentNameVal} 正在思考点评...`
  } catch (e) {
    console.error('Failed to trigger game comment', e)
  }
}

function resetGame() {
  startGame()
}

function closeSettlement() {
  settlementText.value = ''
}
</script>

<style scoped>
.blackjack-container {
  min-height: 100vh;
  background: linear-gradient(135deg, #1e293b 0%, #0f172a 100%);
  display: flex;
  flex-direction: column;
  color: white;
  padding-bottom: env(safe-area-inset-bottom);
}

.header {
  height: 60px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 16px;
  background: rgba(0, 0, 0, 0.2);
  backdrop-filter: blur(10px);
}

.back-btn {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 12px;
  background: rgba(255, 255, 255, 0.1);
  cursor: pointer;
}

.header-title { text-align: center; }
.header-title h2 { margin: 0; font-size: 16px; }
.header-title span { font-size: 10px; color: #94a3b8; letter-spacing: 2px; }
.placeholder { width: 40px; }

.game-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  padding: 20px;
  position: relative;
}

.dealer-area, .player-area {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
}

.agent-info {
  display: flex;
  flex-direction: column;
  align-items: center;
  margin-bottom: 5px;
  padding: 8px 16px;
  background: rgba(255, 255, 255, 0.05);
  border-radius: 20px;
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.agent-name {
  font-size: 14px;
  font-weight: 600;
  color: #f1f5f9;
}

.cards-row {
  display: flex;
  gap: 8px;
  justify-content: center;
  height: 90px;
}

.poker-card {
  width: 60px;
  height: 84px;
  background: white;
  border-radius: 8px;
  color: #1e293b;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 18px;
  font-weight: bold;
  box-shadow: 0 4px 6px rgba(0,0,0,0.3);
  transition: all 0.3s;
}

.poker-card.card-hidden {
  background: #3b82f6;
  background-image: repeating-linear-gradient(
    45deg,
    rgba(255,255,255,0.1) 0px,
    rgba(255,255,255,0.1) 10px,
    transparent 10px,
    transparent 20px
  );
  color: rgba(255,255,255,0.5);
}

.score-tag {
  background: rgba(0, 0, 0, 0.4);
  padding: 4px 12px;
  border-radius: 12px;
  font-size: 14px;
  font-weight: bold;
  font-family: monospace;
}

.game-status-message {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  background: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(4px);
  padding: 10px 20px;
  border-radius: 20px;
  font-weight: bold;
  pointer-events: none;
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.action-bar {
  height: 80px;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 16px;
  padding: 0 20px;
  background: rgba(0, 0, 0, 0.2);
}

.action-btn {
  padding: 12px 24px;
  border: none;
  border-radius: 16px;
  font-weight: 600;
  font-size: 15px;
  cursor: pointer;
  transition: transform 0.1s;
}

.action-btn:active { transform: scale(0.95); }

.action-btn.primary { background: #3b82f6; color: white; width: 100%; max-width: 200px; }
.action-btn.hit { background: #10b981; color: white; flex: 1; }
.action-btn.stand { background: #ef4444; color: white; flex: 1; }

/* 结算气泡样式 */
.settlement-bubble {
  position: absolute;
  bottom: 100px;
  left: 50%;
  transform: translateX(-50%);
  width: fit-content;
  max-width: 85%;
  min-width: 120px;
  background: rgba(255, 255, 255, 0.95);
  backdrop-filter: blur(10px);
  border-radius: 20px;
  padding: 16px 20px;
  color: #1e293b;
  box-shadow: 0 10px 30px rgba(0,0,0,0.3);
  z-index: 100;
  border: 1px solid rgba(255, 255, 255, 0.8);
  display: flex;
  justify-content: center;
}

.bubble-content {
  width: 100%;
}

.settlement-bubble p {
  margin: 0;
  font-size: 15px;
  line-height: 1.6;
  color: #1e293b;
  white-space: pre-wrap;
  word-wrap: break-word;
  word-break: break-word;
  text-align: left;
}

/* 弹跳动画 */
.bounce-enter-active {
  animation: bounce-in 0.5s cubic-bezier(0.175, 0.885, 0.32, 1.275);
}
.bounce-leave-active {
  animation: bounce-in 0.3s reverse ease-in;
}
@keyframes bounce-in {
  0% { transform: translate(-50%, 20px) scale(0.8); opacity: 0; }
  100% { transform: translate(-50%, 0) scale(1); opacity: 1; }
}
</style>