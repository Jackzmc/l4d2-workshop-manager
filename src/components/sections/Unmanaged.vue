<template>
<div>
    <MenuItem ref="menu" :item="menuItem" 
        @enable="$emit('enable', 'unmanaged', menuItem.publishedfileid)" 
        @disable="$emit('disable', 'unmanaged', menuItem.publishedfileid)" 
        @uninstall="$emit('uninstall', 'unmanaged',  menuItem.publishedfileid)"
        @select="selected[menuItem.publishedfileid] = !selected[menuItem.publishedfileid]" 
    />
    <table class="table is-fullwidth hoverable">
        <thead>
            <tr>
                <th style="width: 80px">
                    <b-checkbox @input="onSelectAll" />
                </th>
                <th>Item Name</th>
                <th>File Size</th>
                <th>Last Updated</th>
            </tr>
        </thead>
        <tbody>
            <tr v-for="item in items" :key="item.publishedfileid" >
                <td>
                    <b-checkbox v-model="selected[item.publishedfileid]" />
                    <a class="is-pulled-right" style="color:black"><font-awesome-icon icon="ellipsis-v" @click="openMenu(item)"/></a>
                </td>
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
                <b-button type="is-info" 
                    @click="importAddons"
                    :disabled="loading"
                    :loading="loading"
                >
                    Import Addons
                </b-button>
            </div>
        </div>
    </template>
</div>
</template>

<script>
import { invoke } from '@tauri-apps/api/tauri'
import MenuItem from '@/components/MenuItem'
import { formatBytes, formatDate } from '@/js/utils'

export default {
    props: ['items'],
    components: {
        MenuItem
    },
    data() {
        return {
            active: false,
            selected: {},
            loading: false,
            menuItem: null
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
        },
        importAddons() {
            this.loading = true
            let items = this.items.filter(item => this.selected[item.publishedfileid] && item)
            let running = 0;
            let timer = setInterval(async() => {
                if(items.length == 0 && running == 0) {
                    this.$emit('refreshItems')
                    this.loading = false
                    return clearInterval(timer)
                }else if(running < 6) {
                    let item = items.shift();
                    running++
                    console.log('item', item)
                    await invoke("import_addon", { item, isWorkshop: false })
                    running--
                }
            }, 1000)
        },
        onSelectAll(state) {
            for(const item of this.items) {
                this.$set(this.selected, item.publishedfileid, state)
            }
        },
        openMenu(item) {
            if(!this.$refs.menu) return
            this.menuItem = item
            this.$refs.menu.open(item);
        }
    }
}
</script>

<style scoped>
.context-item {
    padding: 0.5em;
    font-size: 1.3em;
}
.context-item a {
    color: #167df0
}
.context-item a:hover {
    color: black
}
</style>