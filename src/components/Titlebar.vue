<template>
<div class="titlebar" data-tauri-drag-region>
    <h1 class="titlebar-title is-pulled-left">L4D2 Workshop Manager</h1>
    <div class="titlebar-button"  @click="$options.appWindow.minimize()">
        <font-awesome-icon icon="window-minimize" aria-hidden="true" />
    </div>
    <div class="titlebar-button" @click="$options.appWindow.maximize()">
        <font-awesome-icon :icon="['far', 'window-maximize']" aria-hidden="true" />
    </div>
    <div class="titlebar-button" @click="$options.appWindow.close()">
        <font-awesome-icon icon="times" aria-hidden="true" />
    </div>
</div>
</template>

<style scoped>
.titlebar-title {
  display: block;
  position: fixed;
  margin-left: 5px;
  left: 0;
  width: 30px;
  height: 30px;
  white-space: nowrap;
}
h1 { 
    color: #ffffff; 
    font-weight: 800; 
}
.titlebar {
  height: 30px;
  background: #030303;
  user-select: none;
  display: flex;
  justify-content: flex-end;
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  color: white;
}
.titlebar-button {
  display: inline-flex;
  justify-content: center;
  align-items: center;
  width: 30px;
  height: 30px;
}
.titlebar-button:hover {
  background: #5bbec3;
}
</style>

<script>
import { appWindow } from '@tauri-apps/api/window'

export default {
    appWindow,
}
</script>