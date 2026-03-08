import { createRouter, createWebHistory } from 'vue-router'
import MobileHome from '../pages/MobileHome.vue'
import MobileSettings from '../pages/MobileSettings.vue'
import MobileGroupChat from '../pages/MobileGroupChat.vue'
import MobileGameBlackjack from '../pages/MobileGameBlackjack.vue'
import RoleManager from '../pages/RoleManager.vue'

export const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/', component: MobileHome },
    { path: '/settings', component: MobileSettings },
    { path: '/roles', component: RoleManager },
    { path: '/group', component: MobileGroupChat },
    { path: '/game/blackjack', component: MobileGameBlackjack },
  ],
})

export default router
