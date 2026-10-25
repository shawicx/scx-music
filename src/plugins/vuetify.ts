import { createVuetify } from 'vuetify'
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
  teal:       { base: '#009688', darken1: '#00796B', lighten1: '#26A69A', lighten2: '#4DB6AC' },
  indigo:     { base: '#3F51B5', darken1: '#3949AB', lighten1: '#5C6BC0', lighten2: '#7986CB' },
  blue:       { base: '#2196F3', darken1: '#1E88E5', lighten1: '#42A5F5', lighten2: '#64B5F6' },
  deepPurple: { base: '#673AB7', darken1: '#5E35B1', lighten1: '#7E57C2', lighten2: '#9575CD' },
  red:        { base: '#E53935', darken1: '#C62828', lighten1: '#EF5350', lighten2: '#E57373' },
  amber:      { base: '#F2A900', darken1: '#D39100', lighten1: '#F5B841', lighten2: '#F8CB6C' },
}

/**
 * 中性表面分层（对应 tokens.css 的 --surface-0..3）：
 * 暗色 #131318/#1b1b23/#23232e/#2a2a37，浅色 #ececea/#f5f5f4/#e7e7e5/#ffffff。
 * Material 渐变与发光变量已移除，强调效果统一走 --accent CSS 变量与 accent-bg。
 */
function createTheme(c: MaterialColor, dark: boolean = true) {
  const [r, g, b] = [parseInt(c.base.slice(1, 3), 16), parseInt(c.base.slice(3, 5), 16), parseInt(c.base.slice(5, 7), 16)]
  return {
    dark: dark,
    colors: {
      background: dark ? '#1b1b23' : '#f5f5f4',
      surface: dark ? '#1b1b23' : '#f5f5f4',
      'surface-variant': dark ? '#23232e' : '#e7e7e5',
      'surface-bright': dark ? '#131318' : '#ececea',
      'surface-light': dark ? '#2a2a37' : '#ffffff',
      primary: c.base,
      'primary-darken-1': c.darken1,
      secondary: c.lighten1,
      'secondary-lighten-1': c.lighten2,
      error: '#E0524D',
      info: '#009688',
      success: '#4CAF50',
      warning: '#F2A900',
    },
    variables: {
      'border-color': dark ? 'rgba(255,255,255,0.085)' : 'rgba(0,0,0,0.09)',
      'text-secondary': dark ? 'rgba(242,242,245,0.58)' : 'rgba(29,29,31,0.64)',
      'text-muted': dark ? 'rgba(242,242,245,0.4)' : 'rgba(29,29,31,0.45)',
      'accent-bg': `rgba(${r}, ${g}, ${b}, 0.14)`,
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

export type ThemeColor = keyof typeof themeMeta
export type ThemeMode = 'light' | 'dark' | 'system'
export type ThemeName = `${ThemeColor}-${ThemeMode}` | ThemeColor

// Generate all theme combinations (both light and dark variants)
const themes: Record<string, any> = {}
for (const [colorName, colorData] of Object.entries(materialColors)) {
  themes[`${colorName}-light`] = createTheme(colorData, false)
  themes[`${colorName}-dark`] = createTheme(colorData, true)
  // Keep legacy theme names for backward compatibility
  themes[colorName] = createTheme(colorData, true)
}

export default createVuetify({
  icons: {
    defaultSet: 'mdi',
    aliases,
    sets: { mdi },
  },
  theme: {
    defaultTheme: 'teal-dark',
    themes,
  },
})
