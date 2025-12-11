<script setup lang="ts">
import MainLayout from './components/layout/MainLayout.vue';
</script>

<template>
  <MainLayout>
    <router-view />
  </MainLayout>
</template>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}
</style>
