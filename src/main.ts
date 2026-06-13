import { createApp } from 'vue'
import { createPinia } from 'pinia'
import vuetify from './plugins/vuetify'
import i18n from './i18n'

async function bootstrap() {
  const app = createApp(
    window.location.hash === '#desktop-lyrics'
      ? (await import('./desktop-lyrics/DesktopLyricsApp.vue')).default
      : (await import('./App.vue')).default,
  )
  const pinia = createPinia()

  app.use(pinia)
  app.use(vuetify)
  app.use(i18n)
  app.mount('#app')
}

bootstrap()
