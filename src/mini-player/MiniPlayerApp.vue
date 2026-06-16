<script setup lang="ts">
import MiniPlayer from './MiniPlayer.vue'
</script>

<template>
  <v-app>
    <MiniPlayer />
  </v-app>
</template>

<style>
html, body, #app {
  margin: 0;
  padding: 0;
  width: 100%;
  height: 100%;
  overflow: hidden;
  background: rgb(var(--v-theme-surface));
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
}
</style>
