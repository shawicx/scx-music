import { createVuetify } from 'vuetify'
import * as components from 'vuetify/components'
import * as directives from 'vuetify/directives'
import { aliases, mdi } from 'vuetify/iconsets/mdi'
import 'vuetify/styles'
import '@mdi/font/css/materialdesignicons.css'

interface MaterialColor {
  base: string
  darken1: string
  lighten1: string
  lighten2: string
}

const materialColors: Record<string, MaterialColor> = {
  teal:        { base: '#009688', darken1: '#00897B', lighten1: '#26A69A', lighten2: '#4DB6AC' },
  indigo:      { base: '#3F51B5', darken1: '#3949AB', lighten1: '#5C6BC0', lighten2: '#7986CB' },
  blue:        { base: '#2196F3', darken1: '#1E88E5', lighten1: '#42A5F5', lighten2: '#64B5F6' },
  deepPurple:  { base: '#673AB7', darken1: '#5E35B1', lighten1: '#7E57C2', lighten2: '#9575CD' },
  red:         { base: '#F44336', darken1: '#E53935', lighten1: '#EF5350', lighten2: '#E57373' },
  amber:       { base: '#FFC107', darken1: '#FFB300', lighten1: '#FFD54F', lighten2: '#FFE082' },
}

function createTheme(c: MaterialColor) {
  const [r, g, b] = [parseInt(c.base.slice(1, 3), 16), parseInt(c.base.slice(3, 5), 16), parseInt(c.base.slice(5, 7), 16)]
  return {
    dark: true,
    colors: {
      background: '#1a1a2e',
      surface: '#1e1e32',
      'surface-variant': '#2a2a3e',
      'surface-bright': '#0f0f1a',
      primary: c.base,
      'primary-darken-1': c.darken1,
      secondary: c.lighten1,
      'secondary-lighten-1': c.lighten2,
      error: '#F44336',
      info: '#009688',
      success: '#4CAF50',
      warning: '#FFC107',
    },
    variables: {
      'border-color': '#2a2a3e',
      'text-secondary': '#888888',
      'text-muted': '#555555',
      'accent-bg': `rgba(${r}, ${g}, ${b}, 0.12)`,
      'accent-glow': `rgba(${r}, ${g}, ${b}, 0.25)`,
      'accent-shadow': `rgba(${r}, ${g}, ${b}, 0.4)`,
      'gradient-brand': `linear-gradient(135deg, ${c.base}, ${c.darken1})`,
      'gradient-brand-text': `linear-gradient(135deg, ${c.lighten1}, ${c.base})`,
      'gradient-progress': `linear-gradient(90deg, ${c.base}, ${c.lighten1})`,
    },
  }
}

export const themeMeta: Record<string, { label: string; color: string }> = {
  teal:       { label: '青色',   color: materialColors.teal.base },
  indigo:     { label: '靛蓝',   color: materialColors.indigo.base },
  blue:       { label: '蓝色',   color: materialColors.blue.base },
  deepPurple: { label: '深紫',   color: materialColors.deepPurple.base },
  red:        { label: '红色',   color: materialColors.red.base },
  amber:      { label: '琥珀',   color: materialColors.amber.base },
}

export type ThemeName = keyof typeof themeMeta

export default createVuetify({
  components,
  directives,
  icons: {
    defaultSet: 'mdi',
    aliases,
    sets: { mdi },
  },
  theme: {
    defaultTheme: 'teal',
    themes: Object.fromEntries(
      Object.entries(materialColors).map(([name, c]) => [name, createTheme(c)]),
    ),
  },
})
