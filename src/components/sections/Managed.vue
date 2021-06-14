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
                <a class="button is-success">Enable</a>
                <a class="button is-danger">Disable</a>
            </div>
        </div>
    </template>
</div>
</template>

<script>
import { formatBytes, formatDate } from '@/js/utils'
export default {
    name: "Workshop",
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
        toggle() {
            if(this.items.length == 0) return this.active = false
            this.active = !this.active
        }
    }
}
</script>