<template>
<div>
  <TitleBar />
  <br>
  <div id="app">
    <div class="columns is-gapless">
      <div class="column is-3 panel-container">
        <nav class="panel is-info">
          <p class="panel-heading not-rounded">
            Items
          </p>
          <a v-for=" data in SIDEBAR_SECTIONS " :key="data.id"
            :class="['panel-block', 'is-block', { 'panel-active': selected?.id === data.id, 'bold-line': selected?.id == data.id }]"
            @click="openSection( data )">
            <span class="icon-text" v-if="data.icon">
              <span class="icon">
                <font-awesome-icon :icon="data.icon" aria-hidden="true" />
              </span>
              <span>{{ data.title }}</span>
            </span>
            <span class="tag is-white has-text-black ml-2 is-pulled-right" v-if="files[data.id]">{{ files[data.id].length }}</span>
          </a>
         <p class="has-text-centered mt-1"><em>v{{ APP_VERSION }} build #{{ BUILD_NUMBER }}</em></p>
        </nav>
      </div>
      <div class="column mt-3 section-component" id="section">
        <component v-if="selected " :is="selected.component" :items="selectedFiles"
          :key="selected?.id" 
          :settings="settings"
          @refresh="onItems"
          @saved="newSettings => settings = newSettings"
        />
        <p v-else class="title is-4 has-text-centered mt-5">Select an item on the left to begin</p>
        <br><br>
      </div>
    </div>

  </div>
</div>
</template>

<script setup lang="ts">
const APP_VERSION = __APP_VERSION__
const BUILD_NUMBER = __BUILD_NUMBER__

import { invoke } from '@tauri-apps/api/tauri'
import TitleBar from '@/components/Titlebar.vue'
import Managed from '@/components/sections/Managed.vue'
import Workshop from '@/components/sections/Workshop.vue'
import Settings from '@/components/sections/Settings.vue';
import AddContent from '@/components/sections/AddContent.vue'
import { markRaw, ref, onMounted, computed, onBeforeMount } from 'vue'

const SIDEBAR_SECTIONS = [
    {
    id: "managed",
    title: "My Addons",
    component: markRaw( Managed ),
    icon: "list"
  },
  {
    id: "workshop",
    title: "Workshop Addons",
    component: markRaw( Workshop ),
    icon: "list"
  },
  {
    id: "settings",
    title: "Settings",
    component: markRaw( Settings ),
    icon: "cog"
  },
  {
    id: "add-content",
    title: "Add Content",
    component: markRaw( AddContent ),
    icon: "plus"
  }
]
let files = ref<Record<string, any>>({
  workshop: [],
  managed: []
})

let selected = ref<{ id: string, component: any }>()
let loading = ref(false)
let error = ref( null )
let settings = ref<Record<string, any>>()

const selectedFiles = computed( {
  get() {
    if(!selected.value) return null
    return files.value[selected.value.id]
  },
  set( value ) {
    if(selected.value)
      files.value[selected.value.id] = value
  }
})

function openSection(data: any) {
  selected.value = data
}

function onItems( entries: any[] ) {
  if ( !selected.value ) return
  console.debug( "got items for", selected.value?.id )
  selectedFiles.value = entries
}

onBeforeMount( async () => {
  settings.value = await invoke( "get_settings" )
  const fileCache = window.localStorage['files_cache']
  if ( fileCache ) {
    files.value = JSON.parse(fileCache)
  }
})

onMounted( async () => {
  files.value.managed = await invoke( "get_my_addons" )
  files.value.workshop = await invoke( "get_workshop_addons" )
  window.localStorage['files_cache'] = JSON.stringify(files.value)
})
</script>

<style scoped>
#app {
  font-family: Avenir, Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  color: #2c3e50;
}

.not-rounded {
  border-radius: 0 !important;
}

.bold-line {
  border-bottom: 1px solid rgba(53, 51, 51, 0.336) !important
}

.panel:not(:last-child) {
  margin-bottom: 0 !important;
}

.panel-container {
}

.panel {
  background-color: white;
  border-right: 2px solid rgba(53, 51, 51, 0.336);
  border-bottom: 2px solid rgba(53, 51, 51, 0.336);
  height: 100%;
}

.panel-active {
  background-color: #1176dbce;
  color: #F7F6F6 !important;
  font-weight: 700;
}

.panel-active:hover {
  background-color: #1176dbce !important;
  color: #F7F6F6
}

html,
body {
  overflow-y: hidden !important;
  background-color: rgba(255, 255, 255, 0.667);
}

.section-component {
  height: 720px !important;
  overflow: auto !important;
}
</style>
