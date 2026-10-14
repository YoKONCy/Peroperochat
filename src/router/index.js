import { createRouter, createWebHistory } from 'vue-router'

export const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/', component: () => import('../pages/MobileHome.vue') },
    { path: '/settings', component: () => import('../pages/MobileSettings.vue') },
    { path: '/roles', component: () => import('../pages/RoleManager.vue') },
    { path: '/group', component: () => import('../pages/MobileGroupChat.vue') },
    { path: '/game/blackjack', component: () => import('../pages/MobileGameBlackjack.vue') },
  ],
})

export default router
