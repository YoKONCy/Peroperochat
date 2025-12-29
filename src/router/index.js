import { createRouter, createWebHistory } from 'vue-router'
import MobileHome from '../pages/MobileHome.vue'
import MobileSettings from '../pages/MobileSettings.vue'

export const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/', component: MobileHome },
    { path: '/settings', component: MobileSettings },
  ],
})

export default router
