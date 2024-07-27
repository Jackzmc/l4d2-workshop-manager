<template>
<div class="mt-3">
    <!-- TODO: dont do full refresh -->
    <AddonModal v-if=" showDetailAddon " :addon="showDetailAddon" @close="showDetailAddon = null" @refresh="emit( 'refresh' )" />
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
            <tr v-for=" item in itemsFiltered" :key="item.filename">
                <td><b-checkbox v-model="selected[item.file_name]" /></td>
                <td>
                    <a @click="showDetails(item)">{{ item.workshop_info?.title ?? item.addon_data?.info?.title ?? item.file_name }}</a>
                    <AddonTags :addon="item" />
                </td>
                <td>{{ formatBytes( item.file_size ) }}</td>
                <td>{{ formatDate( Number(item.last_update_time) ) }}</td>
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
import { ref, computed, onMounted } from 'vue'
import AddonModal from '../AddonModal.vue';
import AddonTags from '../AddonTags.vue';

const props = defineProps(["items"])
const emit = defineEmits(["refresh"])

let loading = ref(false)
let active = ref(false)
let selected = ref({})
let search = ref({active: false, query: ""})
let showDetailAddon = ref()

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
function showDetails(item) {
    showDetailAddon.value = item
}

let lastUpdateTime
const MIN_UPDATE_INTERVAL = 1000 * 60
onMounted(async() => {
    if(Date.now() - lastUpdateTime > MIN_UPDATE_INTERVAL) {
        invoke("get_my_addons")
            .then(items => {
                emit("refresh", items)
                lastUpdateTime = Date.now()
            })
    }
})

</script>

<style scoped>
td {
    vertical-align: bottom; /** Not sure why, but this seems to center it? */
}
</style>