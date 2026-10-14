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
        <button class="action-btn primary" @click="startGame" :disabled="busy">开始发牌</button>
      </template>
      
      <template v-if="gameState === 'playing'">
        <button class="action-btn hit" @click="hit" :disabled="busy">要牌 (Hit)</button>
        <button class="action-btn stand" @click="stand" :disabled="busy">停牌 (Stand)</button>
      </template>

      <template v-if="gameState === 'finished'">
        <button class="action-btn primary" @click="resetGame" :disabled="busy">再来一局</button>
      </template>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onBeforeUnmount } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import { AGENTS, getActiveAgentId } from '../api'
import { localInvoke, localChat } from '../services/localChat'
import { projectNativeTurn } from '../stores/conversations'
import { visibleMessage } from '../utils/messageContent'

const router = useRouter()
const activeAgentId = getActiveAgentId()
const agentName = computed(() => AGENTS[activeAgentId]?.name || 'Pero')
const game = ref(null)
const busy = ref(true)
const settlementText = ref('')
const gameState = computed(() => game.value?.gameState || 'ready')
const playerHand = computed(() => game.value?.playerHand || [])
const dealerHand = computed(() => game.value?.dealerHand || [])
const playerScore = computed(() => game.value?.playerScore || 0)
const dealerScore = computed(() => game.value?.dealerScore || 0)
const statusMessage = computed(() => game.value?.statusMessage || '')
let disposed = false
onBeforeUnmount(() => { disposed = true })

onMounted(async () => {
  try { game.value = await localInvoke('local_blackjack_snapshot', { agentId: activeAgentId }) }
  catch (error) { if (!disposed) ElMessage.error(error.message) }
  finally { busy.value = false }
})

function getCardDisplay(card) { return `${card.suit}${card.value}` }
function goHome() { router.push('/') }
function closeSettlement() { settlementText.value = '' }

async function act(action) {
  if (busy.value) return
  busy.value = true
  settlementText.value = ''
  try {
    const result = await localChat.turn({
      threadId: `desktop:${activeAgentId}`, agentId: activeAgentId,
      gameId: action === 'start' ? crypto.randomUUID() : game.value.id,
      action, revision: game.value?.revision || 0,
    }, {
      command: 'local_blackjack_play',
      onEvent(event) {
        if (disposed) return
        if (event.kind === 'game') game.value = JSON.parse(event.content)
        if (event.kind === 'started') settlementText.value = `${agentName.value} 正在思考点评...`
      },
    })
    projectNativeTurn(result.turn)
    if (disposed) return
    game.value = result.game
    settlementText.value = result.commentError?.message || (result.turn ? visibleMessage(result.turn.message.content) : '')
  } catch (error) {
    if (!disposed) {
      settlementText.value = error.message || '操作暂时失败，请稍后重试'
      try { game.value = await localInvoke('local_blackjack_snapshot', { agentId: activeAgentId }) }
      catch (refreshError) { console.error('恢复牌局失败：', refreshError) }
    }
  } finally { busy.value = false }
}
const startGame = () => act('start')
const hit = () => act('hit')
const stand = () => act('stand')
const resetGame = startGame
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