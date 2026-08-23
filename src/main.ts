import { createApp } from 'vue'
import App from './App.vue'
import './style.css'
import { initializeTheme } from './lib/theme'

initializeTheme()
createApp(App).mount('#app')
