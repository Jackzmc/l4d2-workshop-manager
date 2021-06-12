<template>
<div id="app">
  <div class="columns is-gapless">
    <div class="column is-3">
      <nav class="panel is-link">
        <p class="panel-heading not-rounded">
          Items
        </p>
        <a class="panel-block is-active" @click="section == $options.SECTIONS.Updateable">
          <span class="icon-text">
            <span class="icon">
              <font-awesome-icon icon="list" aria-hidden="true" />
            </span>
            <span>Updateable ({{files.updateable.length}})</span>
          </span>
        </a>
        <a class="panel-block" @click="section == $options.SECTIONS.Managed">
          <span class="icon-text">
            <span class="icon">
              <font-awesome-icon icon="list" aria-hidden="true" />
            </span>
            <span>Managed ({{files.managed.length}})</span>
          </span>
        </a>
        <a class="panel-block" @click="test">
          <span class="icon-text">
            <span class="icon">
              <font-awesome-icon icon="list" aria-hidden="true" />
            </span>
            <span>Unmanaged ({{files.unmanaged.length}})</span>
          </span>
        </a>
        <a class="panel-block">
          <span class="icon-text">
            <span class="icon">
              <font-awesome-icon icon="list" aria-hidden="true" />
            </span>
            <span>Workshop ({{files.workshop.length}})</span>
          </span>
        </a>
        <a class="panel-block bold-line">
          <span class="icon-text">
            <span class="icon">
              <font-awesome-icon icon="list" aria-hidden="true" />
            </span>
            <span>Unknown ({{files.unknown.length}})</span>
          </span>
        </a>
        <a class="panel-block">
          <span class="icon-text">
            <span class="icon">
              <font-awesome-icon icon="cog" aria-hidden="true" />
            </span>
            <span>Settings</span>
          </span>
        </a>
        <div class="panel-block">
          <button class="button is-success is-outlined is-fullwidth">
            Add New
          </button>
        </div>
      </nav>
    </div>
    <div class="column">
      <component :is="section" :items="items"/>
    </div>
  </div>
  <div class="container" v-if="loading">
    Loading Items...
  </div>
  <div class="container" v-else>
    <br>
    <article class="message is-danger" v-if="error">
      <div class="message-body">
        {{error}}
      </div>
    </article>
    <div class="box" v-if="totalItems == 0">
      No items found
    </div>
    <div class="box" v-if="updating">
      <div v-for="(update,key) in updates" :key="key">
        <b>{{update.title}}</b> <em>({{key}})</em><br>
        <div style="width:60%" class="is-inline">
          <progress class="progress is-success is-inline-block" :value="update.bytes_downloaded" :max="update.bytes_total" style="width:60%" >
            {{update.bytes_downloaded / update.bytes_total * 100}}%
          </progress>
        </div>
        <div class="is-inline" style="margin-left: 1em">
          <p v-if="update.error" class="has-text-danger">
            <b>Failed: </b>{{update.error}}
          </p>
          <template v-else-if="update.complete">
            Complete 
          </template>
          <template v-else>
            {{formatBytes(update.bytes_downloaded)}} / {{formatBytes(update.bytes_total)}}
            &nbsp; ({{Math.round(update.bytes_downloaded / update.bytes_total * 100)}}%)
          </template>
        </div>
      </div>
    </div>
    <Updateable :items="files.updateable" @refreshItems="getItems" />
    <br>
    <Managed :items="files.managed" />
    <br>
    <Unmanaged :items="files.unmanaged" />
    <br>
    <Workshop :items="files.workshop" />
    <br>
    <Unknown :items="files.unknown" />
    <br>
    <AddNew />
    <br>
  </div>
</div>
</template>

<script>
import { invoke } from '@tauri-apps/api/tauri'
import { listen } from '@tauri-apps/api/event'

import Updateable from '@/components/sections/Updateable.vue'
import Managed from '@/components/sections/Managed.vue'
import Unmanaged from '@/components/sections/Unmanaged.vue'
import Workshop from '@/components/sections/Workshop.vue'
import Unknown from '@/components/sections/Unknown.vue'
import AddNew from '@/components/sections/AddNew.vue'

import test from '@/components/sections/test.vue'

import { formatBytes, formatDate } from '@/js/utils'

const SECTIONS = {
  Updateable,
  Managed,
  Unmanaged,
  Workshop,
  Unknown,
  AddNew,
  test
}

export default {
  name: 'App',
  components: {
    ...SECTIONS
  },
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
      section: null,
      items: null
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
  },
  methods: {
    test() {
      this.items = this.files['updateable']
      this.section = SECTIONS.test
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
    await listen('progress', ({payload}) => {
      if(payload.error) {
          return console.error(`${payload.publishedfileid} -> ${payload.error}`)
      }
      this.updates[payload.publishedfileid] = {
          ...this.updates[payload.publishedfileid],
          bytes_downloaded: payload.bytes_downloaded,
          complete: payload.complete
      }
      if(payload.complete) {
          setTimeout(() => this.$delete(this.updates, payload.publishedfileid), 5000)
      }
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
/* html, body {
  background-color: #3298dc !important
} */
</style>
