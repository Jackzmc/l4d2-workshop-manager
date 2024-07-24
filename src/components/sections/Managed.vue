<template>
<div class="mt-3">
    <table class="table is-fullwidth">
        <thead>
            <tr>
                <th style="width: 40px">
                    <b-checkbox @input="onSelectAll" />
                </th>
                <th>
                    <span>Item Name</span>
                    <div class="is-inline is-pulled-right">
                        <b-input v-model.lazy="search.query" placeholder="Search..." icon="search" rounded
                            @blur="search.active = false" @focus="search.active = true"
                            :size="search.active === false ? 'is-small' : ''" />
                    </div>
                </th>
                <th>File Size</th>
                <th>Last Updated</th>
            </tr>
        </thead>
        <tbody>
            <tr v-for=" item in itemsFiltered" :key="item.publishedfileid">
                <td><b-checkbox v-model="selected[item.file_name]" /></td>
                <td @click="selected[item.file_name] = !selected[item.file_name]">
                    <a v-if="item.workshop_info" target="_blank"
                        :href="'https://steamcommunity.com/sharedfiles/filedetails/?id=' + item.workshop_info.publishedfileid"
                        class="has-text-info">
                        {{ item.workshop_info?.title ?? item.file_name }}
                    </a>
                    <span v-else>{{ item.file_name }}</span>
                </td>
                <td>{{ formatBytes( item.file_size ) }}</td>
                <td>{{ formatDate( item.last_update_time ) }}</td>
            </tr>
        </tbody>
        <tfoot>
            <tr>
                <td></td>
                <th>Total File Size: </th>
                <th>{{ formatBytes( totalBytes ) }}</th>
                <td></td>
            </tr>
        </tfoot>
    </table>
    <!-- <template v-if="hasItemSelected">
        <hr>
        <div class="container ml-5" v-if="hasItemSelected">
            <b>Action for selected items</b><br>
            <div class="buttons">
                <a class="button is-success">Enable</a>
                <a class="button is-danger">Disable</a>
            </div>
        </div>
    </template> -->
</div>
</template>

<script setup>
import { formatBytes, formatDate } from '@/js/utils'
import { invoke } from '@tauri-apps/api/tauri'
import Fuse from 'fuse.js'
import { ref, computed } from 'vue'

const props = defineProps(["items"])
const emit = defineEmits(["refreshItems"])

let loading = ref(false)
let active = ref(false)
let selected = ref({})
let search = ref({active: false, query: ""})

const totalBytes = computed(() => {
    let bytes = 0;
    for(const item in props.items) {
        bytes += props.items[item].file_size
    }
    return bytes
})
const canOpen = computed(() => {
    return props.items.length > 0
})

const hasItemSelected = computed(() => {
    for(const item in selected.value) {
        if(selected.value[item] === true) return true
    } 
    return false;
})

const itemsFiltered = computed(() => {
    if(search.value.query === "") return props.items
    const fuse = new Fuse(props.items, {
        keys: ['file_name','workshop_info.title', 'workshop_info.author'],
        distance: 15,
        threshold: 0.5,
        includeScore: true
    })
    return fuse.search(search.value.query).map(r => r.item)
})

function toggle() {
    if(props.items.length == 0) return active.value = false
    active.value = !active.value
}
function importAddons() {
    loading.value = true
    let items = props.items.filter(item => selected.value[item.publishedfileid] && item)
    const itemsCopy = [...items]
    let running = 0;
    let timer = setInterval(async() => {
        if(items.length == 0 && running == 0) {
            emit('refreshItems')
            loading.value = false
            // TODO: add
            // this.$buefy.dialog.alert({
            //     title: 'Addons Imported',
            //     message: `Successfully imported ${itemsCopy.length} items. <br>Please unsubscribe to them on the steam workshop to prevent duplicates:<br>`
            //         + itemsCopy.map(item => `<a href="https://steamcommunity.com/sharedfiles/filedetails/?id=${item.publishedfileid}" target="_blank">${item.title}</a>`).join("<br"),
            //     type: 'is-success',
            //     ariaRole: 'alertdialog',
            //     ariaModal: true
            // })
            return clearInterval(timer)
        }else if(running < 6) {
            let item = items.shift();
            running++
            console.log('item', item)
            await invoke("import_addon", { item, isWorkshop: true })
            running--
        }
    }, 1000)
}

function onSelectAll(state) {
    for(const item of props.items) {
        selected.value[item.publishedfileid] = state
    }
} 

</script>

<style scoped>
td {
    vertical-align: bottom; /** Not sure why, but this seems to center it? */
}
</style>