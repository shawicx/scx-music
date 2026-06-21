<script setup lang="ts">
import MiniPlayer from './MiniPlayer.vue'
</script>

<template>
  <v-app>
    <MiniPlayer />
  </v-app>
</template>

<style src="../styles/window-reset.css"></style>
<style>
html, body, #app {
  background: rgb(var(--v-theme-surface));
}
</style>
