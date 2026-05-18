<template>
  <main v-if="profile?.ready" v-bind:[kindName]="profile.name.trim()" @click.stop="save(value)">
    <Panel v-slot:[slotName]="slotProps" v-model="value">{{ slotProps.title ?? title }}</Panel>
  </main>
</template>

<script setup lang="ts">
const ready = true;
</script>

<style scoped>
.root { color: red; }
</style>
