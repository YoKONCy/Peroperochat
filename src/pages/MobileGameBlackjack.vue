<template>
  <div class="blackjack-container">
    <CompanionHeader title="21 点" :subtitle="`和 ${agentName} 玩一局`" @back="goHome" />

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
import CompanionHeader from '../components/companion/CompanionHeader.vue'
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
.blackjack-container { min-height: 100dvh; display: flex; flex-direction: column; background: var(--room-page); padding-bottom: env(safe-area-inset-bottom); }
.game-area { flex: 1; display: flex; flex-direction: column; justify-content: space-between; gap: 24px; width: min(calc(100% - 32px), 800px); min-height: 480px; margin: 28px auto; padding: 32px 22px; background: var(--room-accent-soft); border: 1px solid var(--room-line); border-radius: 20px; }
.dealer-area, .player-area { display: flex; flex-direction: column; align-items: center; gap: 16px; }
.agent-name { font-size: 15px; font-weight: 600; color: var(--room-ink); }
.cards-row { display: flex; justify-content: center; flex-wrap: wrap; gap: 10px; }
.poker-card { display: grid; place-items: center; width: 64px; height: 92px; border: 1px solid #d4dde6; border-radius: 10px; background: var(--room-surface); color: var(--room-ink); box-shadow: 0 3px 9px #253b4c0c; font-size: 21px; font-weight: 500; }
.poker-card.card-hidden { background: repeating-linear-gradient(45deg, var(--room-accent-soft), var(--room-accent-soft) 6px, var(--room-surface) 6px, var(--room-surface) 8px); color: var(--room-accent); }
.score-tag { min-width: 35px; padding: 5px 10px; border-radius: 8px; background: var(--room-surface); font-size: 13px; color: var(--room-accent); text-align: center; }
.game-status-message { text-align: center; color: var(--room-muted); font-size: 14px; line-height: 1.7; }
.player-label { font-size: 11px; color: var(--room-muted); letter-spacing: .08em; }
.action-bar { display: flex; justify-content: center; gap: 12px; padding: 4px 20px 28px; }
.action-btn { min-width: 120px; min-height: 44px; border: 1px solid var(--room-line); border-radius: 11px; padding: 12px 20px; background: var(--room-surface); color: var(--room-ink); font-weight: 500; }
.action-btn.primary, .action-btn.hit { border-color: transparent; background: var(--room-accent); color: white !important; }
.settlement-bubble { width: min(calc(100% - 32px), 800px); margin: 0 auto 20px; border: 1px solid var(--room-line); border-radius: 15px; padding: 16px 20px; background: var(--room-surface); cursor: pointer; }
.bubble-content p { margin: 0; font-size: 14px; line-height: 1.75; white-space: pre-wrap; overflow-wrap: anywhere; }
.bounce-enter-active, .bounce-leave-active { transition: opacity .2s; }
.bounce-enter-from, .bounce-leave-to { opacity: 0; }
@media (max-width: 400px) { .poker-card { width: 52px; height: 78px; font-size: 18px; } .game-area { padding: 26px 14px; } .cards-row { gap: 7px; } }
</style>
