import { createApp } from 'vue'
import { createPinia } from 'pinia'
import vuetify from './plugins/vuetify'
import i18n from './i18n'

async function bootstrap() {
  let component
  if (window.location.hash === '#desktop-lyrics') {
    component = (await import('./desktop-lyrics/DesktopLyricsApp.vue')).default
  } else if (window.location.hash === '#desktop-lyrics-lock') {
    component = (await import('./desktop-lyrics/DesktopLyricsLockApp.vue')).default
  } else {
    component = (await import('./App.vue')).default
  }
  const app = createApp(component)
  const pinia = createPinia()

  app.use(pinia)
  app.use(vuetify)
  app.use(i18n)
  app.mount('#app')
}

bootstrap()
