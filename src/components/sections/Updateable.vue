<template>
<div class="card" v-if="items.length > 0">
    <header class="card-header" @click="active = !active">
    <p class="card-header-title" style="cursor: pointer">
        Updateable Items ({{items.length}})
    </p>
    <a class="card-header-icon" aria-label="more options">
        <font-awesome-icon :icon="active ? 'angle-up' : 'angle-down'" size="lg" aria-hidden="true" />
    </a>
    </header>
    <div class="card-content" v-if="active">
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
            <tr v-for="item in items" :key="item.publishedfileid" >
            <td><input v-model="selected[item.publishedfileid]" class="checkbox is-large" type="checkbox" /></td>
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
        <hr>
        <b>Action for selected</b><br>
        <div class="buttons">
            <a class="button is-primary" @click="update">Update</a>
            <a class="button is-success">Enable</a>
            <a class="button is-danger">Disable</a>
        </div>
    </div>
    </div>
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
            selected: [],
            updates: {}
        }
    },
    computed: {
        total_bytes() {
            let bytes = 0;
            for(const item in this.items) {
                bytes += this.items[item].file_size
            }
            return bytes
        }
    },
    methods: {
        formatBytes, 
        formatDate,
        async update() {
            let items = this.files.updateable.items.filter(item => this.files.updateable.selected[item.publishedfileid])
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
                this.files.updateable.selected[item.publishedfileid] = false;
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
                }
                if(running < CONCURRENT_DOWNLOADS) {
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