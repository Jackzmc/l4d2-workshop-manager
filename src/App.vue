<template>
<div>
  <TitleBar />
  <br>
  <div id="app">
    <div class="columns is-gapless">
      <div class="column is-3 panel-container" >
        <nav class="panel is-info" style="height:650px">
          <p class="panel-heading not-rounded">
            Items
          </p>
          <a v-for="(key, index) in Object.keys($options.MAIN_SECTIONS)" :key="key"
            :class="['panel-block', {'panel-active': section.id == key, 'bold-line': index == Object.keys($options.MAIN_SECTIONS).length - 1}]" 
            @click="openSection(key)"
          >
            <span class="icon-text">
              <span class="icon">
                <font-awesome-icon icon="list" aria-hidden="true" />
              </span>
              <span>{{key}} <span class="tag is-info is-light">{{files[key.toLowerCase()].length}}</span></span>
            </span>
          </a>
          <a :class="['panel-block', {'panel-active': section.id == 'Settings'}]" @click="openSection('Settings')">
            <span class="icon-text">
              <span class="icon">
                <font-awesome-icon icon="cog" aria-hidden="true" />
              </span>
              <span>Settings</span>
            </span>
          </a>
          <a :class="['panel-block', {'panel-active': section.id == 'AddNew'}]" @click="openSection('AddNew')">
            <span class="icon-text">
              <span class="icon has-text-success">
                <font-awesome-icon icon="plus" aria-hidden="true" />
              </span>
              <span>Add New</span>
            </span>
          </a>
          <div class="panel-block" @click="getItems">
            <b-button type="is-info" outlined expanded :disabled="loading" :loading="loading">
              Refresh
            </b-button>
          </div>
        </nav>
        <p class="has-text-centered mt-1"><em>V{{$VERSION}} Build #{{$BUILD}}</em></p>
      </div>
      <div class="column mt-3 section-component" id="section">
        <component 
          v-if="section.component"
          :is="section.component" 
          :items="items"
          v-bind="section.props"
          @refreshItems="getItems"  
        />
        <p v-else class="title is-4 has-text-centered mt-5">Select an item on the left to begin</p>
        <br><br>
      </div>
    </div>
    
  </div>
</div>
</template>

<script>
import { invoke } from '@tauri-apps/api/tauri'

import Updateable from '@/components/sections/Updateable.vue'
import Managed from '@/components/sections/Managed.vue'
import Unmanaged from '@/components/sections/Unmanaged.vue'
import Workshop from '@/components/sections/Workshop.vue'
import Unknown from '@/components/sections/Unknown.vue'
import AddNew from '@/components/sections/AddNew.vue'
import Settings from '@/components/sections/Settings.vue'

import TitleBar from '@/components/Titlebar.vue'


import { formatBytes, formatDate } from '@/js/utils'

const MAIN_SECTIONS = {
  Updateable,
  Managed,
  Unmanaged,
  Workshop,
  Unknown,
}

const SECTIONS = {
  ...MAIN_SECTIONS,
  AddNew,
  Settings
}

export default {
  name: 'App',
  components: {
    ...SECTIONS,
    TitleBar
  },
  MAIN_SECTIONS,
  SECTIONS,
  data() {
    return {
      error: null,
      settings: null,
      updates: {},
      updating: false,
      loading: false,
      files: {
        updateable: [],
        managed: [],
        unmanaged: [],
        workshop: [],
        unknown: [],
      },
      total_bytes: {},
      section: {
        component: null,
        props: null,
        id: null
      },
    }
  },
  computed: {
    totalItems() {
      let count = 0;
      for(const category in this.files) {
        count += this.files[category].length
      }
      return count
    },
    items() {
      return this.section.id ? this.files[this.section.id.toLowerCase()] : []
    }
  },
  methods: {
    openSection(name) {
      let sectionProps = {}
      if(name === "Settings") {
        sectionProps = {
          settings: this.settings
        }
      }
      this.section.component = SECTIONS[name]
      this.section.id = name
      this.section.props = sectionProps
    },
    formatBytes, 
    formatDate,
    //Updates all selected managed
    
    async getItems() {
      this.loading = true
      for(const category in this.files) {
        this.files[category] = []
      }
      try {
        const items = await invoke('get_items')
        for(const file of items) {
          let category = "unmanaged"
          switch(file.item_type) {
            case "Updateable": 
              category = "updateable";
              break
            case "Managed": 
              category = "managed";
              break
            case "Workshop": 
              category = "workshop";
              break;
            case "Unknown":
              category = "unknown"
              break
            default: 
              category = "unmanaged";
          }
          this.files[category].push(file.item)
        }
      }catch(error) {
        this.error = error.message
      }
      this.loading = false
    }
  },
  async created() {
    try {
      await this.getItems() 
    } catch(err) {
      this.error = err
    }
    try {
      this.settings = await invoke('get_settings')
      console.log('settings', Object.assign({}, this.settings))
    } catch(err) {
      console.error('Could not get config: ', err)
    }
    
    document.addEventListener("resize", () => {
      document.getElementById("section").style.height = window.innerHeight 
    })
  },
  async mounted() {
    await invoke('close_splashscreen')
  }
}
</script>

<style>
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
  border-bottom: 1px solid rgba(53, 51, 51, 0.336)!important
}
.panel:not(:last-child) {
  margin-bottom: 0 !important;
}
.panel {
  background-color: white
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
html, body {
  overflow-y: hidden !important;
  background-color: rgba(255, 255, 255, 0.667);
}
.section-component {
  height: 720px !important;
  overflow: auto !important;
}

</style>
