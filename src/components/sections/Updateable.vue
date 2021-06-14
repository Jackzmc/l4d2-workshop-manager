<template>
<div>
    <table class="table is-fullwidth">
        <thead>
            <tr>
                <th style="width: 40px"></th>
                <th>Item Name</th>
                <th>File Size</th>
                <th>Last Updated</th>
            </tr>
        </thead>
        <tbody>
            <tr v-for="item in items" :key="item.publishedfileid" >
                <td><b-checkbox v-model="selected[item.publishedfileid]" /></td>
                <td @click="selected[item.publishedfileid] = !selected[item.publishedfileid]">
                    <a target="_blank" :href="'https://steamcommunity.com/sharedfiles/filedetails/?id=' + item.publishedfileid">
                        {{item.title || item.publishedfileid}}
                    </a>
                </td>
                <td>{{formatBytes(item.file_size)}}</td>
                <td>{{formatDate(item.time_updated)}}</td>
            </tr>
        </tbody>
        <tfoot>
            <tr>
                <td></td>
                <th>Total File Size: </th>
                <th>{{formatBytes(total_bytes)}}</th>
                <td></td>
            </tr>
        </tfoot>
    </table>
    <template v-if="hasItemSelected">
        <hr>
        <div class="container ml-5" v-if="hasItemSelected">
            <b>Action for selected items</b><br>
            <div class="buttons">
                <b-button class="button is-info" @click="update" :disabled="updating" :loading="updating">Update</b-button>
                <a class="button is-success">Enable</a>
                <a class="button is-danger">Disable</a>
            </div>
        </div>
    </template>
</div>
</template>

<script>
import { invoke } from '@tauri-apps/api/tauri'

import { formatBytes, formatDate } from '@/js/utils'
const CONCURRENT_DOWNLOADS = 3

export default {
    props: ['items'],
    data() {
        return {
            active: false,
            selected: {},
            updates: {},
            updating: false
        }
    },
    computed: {
        total_bytes() {
            let bytes = 0;
            for(const item in this.items) {
                bytes += this.items[item].file_size
            }
            return bytes
        },
        hasItemSelected() {
           for(const item in this.selected) {
               if(this.selected[item] === true) return true
           } 
           return false;
        }
    },
    methods: {
        formatBytes, 
        formatDate,
        async update() {
            let items = this.items.filter(item => this.selected[item.publishedfileid])
            if(items.length == 0) return
            this.updating = true;
            items.forEach(item => {
                this.$set(this.updates, item.publishedfileid, {
                    bytes_total: item.file_size,
                    bytes_downloaded: 0,
                    complete: false,
                    title: item.title
                })
                //TODO: Add back
                this.selected[item.publishedfileid] = false;
            })
            let running = 0;
            let timer = setInterval(async() => {
                if(items.length == 0 && running == 0) {
                    this.$emit('refreshItems')
                    this.updating = false
                    for(const item in items) {
                        this.$delete(this.updates, item.publishedfileid)
                    }
                    return clearInterval(timer)
                }else if(running < CONCURRENT_DOWNLOADS) {
                    let item = items.shift();
                    running++
                    await invoke("download_addon", { item })
                    .catch(err => this.updates[item.publishedfileid].error = err)
                    running--
                }
            }, 1000)
        },
    },
    async created() {
        
    },
}
</script>