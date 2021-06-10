<template>
<div id="app" class=" has-background-info">
  <div class="container">
    <br>
    <article class="message is-danger" v-if="error">
      <div class="message-body">
        {{error}}
      </div>
    </article>
    <div class="box">
      <p v-for="(update,key) in updates" :key="key">
        <u>{{key}}</u><br>
        Downloaded: {{update.bytes_downloaded}}<br>
        total: {{update.bytes_total}}<br>
        left: {{update.bytes_total - update.bytes_downloaded}}<br>
        %: {{update.bytes_downloaded / update.bytes_total * 100}}<br>
        complete: {{update.complete ? 'yes' : 'no'}}<br>
      </p>
    </div>
    <div v-for="(category, key) in files" :key="key">
      <template v-if="category.items.length > 0">
        <div class="card">
          <header class="card-header" @click="category.active = !category.active">
            <p class="card-header-title"  style="cursor: pointer">
              {{category.title}} ({{category.items.length}})
            </p>
            <a class="card-header-icon" aria-label="more options" >
              <font-awesome-icon :icon="category.active ? 'angle-up' : 'angle-down'" size="lg" aria-hidden="true" />
            </a>
          </header>
          <div class="card-content" v-if="category.active">
            <div class="content">
              <table class="table is-fullwidth ">
                <thead>
                  <tr>
                    <th style="width: 40px"></th>
                    <th>Item Name</th>
                    <th>File Size</th>
                    <th>Last Updated</th>
                  </tr>
                </thead>
                <tbody>
                  <tr v-for="item in category.items" :key="item.publishedfileid" >
                    <td><input v-model="category.selected[item.publishedfileid]" class="checkbox is-large" type="checkbox" /></td>
                    <td @click="category.selected[item.publishedfileid] = !category.selected[item.publishedfileid]">{{item.title || item.publishedfileid}}</td>
                    <td>{{formatBytes(item.file_size)}}</td>
                    <td>{{formatDate(item.time_updated)}}</td>
                  </tr>
                </tbody>
                <tfoot>
                  <tr>
                    <td></td>
                    <th>Total File Size: </th>
                    <th>{{formatBytes(category.total_bytes)}}</th>
                    <td></td>
                  </tr>
                </tfoot>
              </table>
              <hr>
              <b>Action for selected</b><br>
              <div class="buttons">
                <a class="button is-primary" @click="update()">Update</a>
                <a class="button is-success">Enable</a>
                <a class="button is-danger">Disable</a>
              </div>
            </div>
          </div>
        </div>
        <br>
      </template>
    </div>
  </div>
</div>
</template>

<script>
import { invoke } from '@tauri-apps/api/tauri'
import { listen } from '@tauri-apps/api/event'

export default {
  name: 'App',
  data() {
    return {
      error: null,
      settings: null,
      updates: {},
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
  components: {
  },
  methods: {
    formatBytes(bytes) {
      if(bytes > 1000000000) {
        return `${Math.round(bytes / 1000000000.0)} GB`
      }else if (bytes > 1000000) {
        return `${Math.round(bytes / 1000000.0)} MB`
      } else if (bytes > 1000) {
        return `${Math.round(bytes / 1000.0)} KB`
      } else {
        return `${Math.round(bytes)} B`
      }
    },
    formatDate(date) {
      const d = new Date(date * 1000)
      return `${d.toLocaleDateString()}`
    },
    //Updates all selected managed
    async update() {
      const items = this.files.updateable.items.filter(file => this.files.updateable.selected[file.publishedfileid])
      for(const item of items) {
        this.$set(this.updates, item.publishedfileid, {
          bytes_total: item.file_size,
          bytes_downloaded: 0,
          complete: false
        })
      }
      console.log('updating', items)
      try {
        await listen('progress', ({payload}) => {
          if(!payload.error) {
            this.updates[payload.publishedfileid].bytes_downloaded = payload.bytes_downloaded
            this.updates[payload.publishedfileid].complete = payload.complete
            
          }else{
            console.log('complete', payload)
          }
        })

        await invoke("download_addons", { items })
      } catch (err) {
        alert("Failure: " + err)
      }
    },
    async getItems() {
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
    }
  },
  async mounted() {
    this.getItems() 
    try {
      this.settings = await invoke('get_config')
      console.log('settings', Object.assign({}, this.settings))
    } catch(err) {
      console.error('Could not get config: ', err)
    }
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
</style>
