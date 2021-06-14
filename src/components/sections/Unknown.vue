<template>
<div>
    <p class="ml-5">
        <font-awesome-icon icon="exclamation-triangle" class="has-text-warning" />
        These items do not include a publishedfileid, therefore cannot be managed by this tool.
    </p>
    <table class="table is-fullwidth">
        <thead>
            <tr>
                <th>Item Name</th>
                <th>File Size</th>
                <th>Last Updated</th>
            </tr>
        </thead>
        <tbody>
            <tr v-for="item in items" :key="item.publishedfileid" >
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
    <hr>
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