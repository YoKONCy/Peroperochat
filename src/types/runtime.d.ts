import type { DataRecord } from './ui'
export {}
declare global {
  interface Error { code?: string; outcome?: string; requestId?: string; superseded?: boolean }
  interface Window {
    PPC_LIVE2D_ENABLED?: boolean
    PPC_LIVE2D_ACTIVE?: boolean
    __PPC_LIVE2D_READY?: Promise<unknown>
    __PPC_LIVE2D_SCRIPT?: Promise<unknown>
    PPC_LIVE2D_CONFIG?: DataRecord
    WAIFU_TEXTS?: DataRecord
    WAIFU_INTERACTION_LINES?: DataRecord
    WAIFU_GET_AGENT_ID?: () => string
    PPC_SAVE_MODEL_SETTINGS?: (modelId: number, textureId: number, agentId?: string) => Promise<void>
    WaifuWidget?: {
      loadModel(modelId: number, textureId: number, message?: string, save?: boolean): Promise<unknown>
      loadOtherModel(): Promise<unknown>
    }
    webkitAudioContext?: typeof AudioContext
  }
}
