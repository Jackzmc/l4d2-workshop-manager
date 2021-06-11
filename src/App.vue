<template>
<div id="app">
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
    <Updateable :items="files.updateable.items" @refreshItems="getItems" />
    <br>
    <Managed :items="files.managed.items" />
    <br>
    <Unmanaged :items="files.unmanaged.items" />
    <br>
    <Workshop :items="files.workshop.items" />
    <br>
    <Unknown :items="files.unknown.items" />
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

import { formatBytes, formatDate } from '@/js/utils'


export default {
  name: 'App',
  components: {
    Updateable,
    Managed,
    Unmanaged,
    Workshop,
    Unknown
  },
  data() {
    return {
      error: null,
      settings: null,
      updates: {},
      updating: false,
      loading: false,
      files: {
        updateable: {
          total_bytes: 0, 
          items: [],
          selected: {},
          title: 'Update Pending Items',
          active: true
        },
        managed: { 
          total_bytes: 0, 
          items: [],
          selected: {},
          title: "Managed Items",
          active: true
        },
        unmanaged: { 
          total_bytes: 0, 
          items: [],
          selected: {},
          title: "Unmanaged Items",
          active: false
        },
        workshop: {
          total_bytes: 0, 
          items: [],
          selected: {},
          title: "Workshop Items",
          active: false
        },
        unknown: {
          items: [],
          selected: {},
          title: "Unknown Items",
          active: false
        }
      },
      total_bytes: {}
    }
  },
  computed: {
    totalItems() {
      let count = 0;
      for(const category in this.files) {
        count += this.files[category].items.length
      }
      return count
    }
  },
  methods: {
    formatBytes, 
    formatDate,
    //Updates all selected managed
    
    async getItems() {
      this.loading = true
      for(const category in this.files) {
        this.files[category].items = []
        this.files[category].total_bytes = 0
        this.files[category].selected = {}
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
          this.files[category].total_bytes += file.item.file_size
          this.files[category].items.push(file.item)
          this.$set(this.files[category].selected, file.item.publishedfileid, false)
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
html, body {
  background-color: #3298dc !important
}
</style>
