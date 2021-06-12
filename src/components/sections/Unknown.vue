<template>
<div class="card">
    <header class="card-header" @click="toggle">
    <p class="card-header-title" :style="canOpen ? 'cursor: pointer' : ''">
        Unknown Items ({{items.length}})
    </p>
    <a class="card-header-icon" aria-label="more options" v-if="canOpen">
        <font-awesome-icon :icon="active ? 'angle-up' : 'angle-down'" size="lg" aria-hidden="true" />
    </a>
    </header>
    <div class="card-content" v-if="active">
    <div class="content">
        These items do not include a publishedfileid, therefore cannot be managed by this tool.
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
            <td @click="selected[item.publishedfileid] = !selected[item.publishedfileid]">{{item.title || item.publishedfileid}}.vpk</td>
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
    </div>
    </div>
</div>
</template>

<script>
import { formatBytes, formatDate } from '@/js/utils'
export default {
    props: ['items'],
    data() {
        return {
            active: false,
            selected: {}
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
        canOpen() {
            return this.items.length > 0
        }
    },
    methods: {
        formatBytes, 
        formatDate,
        toggle() {
            if(this.items.length == 0) return this.active = false
            this.active = !this.active
        }
    }
}
</script>