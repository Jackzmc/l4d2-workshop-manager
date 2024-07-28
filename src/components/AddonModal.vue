<template>
<div class="modal is-active">
    <div class="modal-background"></div>
    <div class="modal-card">
        <header class="modal-card-head">
            <p class="modal-card-title">{{ addonName }}</p>
            <button class="delete" aria-label="close" @click="emit('close')"></button>
        </header>
        <section class="modal-card-body">
            <AddonTags :addon="props.addon" />
            <h4 class="title is-4">File Info</h4>
            <table class="table is-striped is-fullwidth">
                <tr>
                    <th>Enabled</th>
                    <td>{{ isDisabled ? 'No' : 'Yes' }}</td>
                </tr>
                <tr>
                    <th>File Name</th>
                    <td>{{ props.addon.file_name }}</td>
                </tr>
                <tr>
                    <th>File Size</th>
                    <td>{{ formatBytes(props.addon.file_size) }}</td>
                </tr>
                <tr>
                    <th>Last Updated</th>
                    <td>{{ formatDate(props.addon.last_update_time) }}</td>
                </tr>
                <tr>
                    <th>Created</th>
                    <td>{{ formatDate(props.addon.create_time) }}</td>
                </tr>
            </table>
            <h4 class="title is-4">Addon Info</h4>
            <table class="table is-striped is-fullwidth" v-if="props.addon.addon_data">
                <tr>
                    <th>Author</th>
                    <td>{{ props.addon.addon_data.info.author }}</td>
                </tr>
                <tr>
                    <th>Version</th>
                    <td>{{ props.addon.addon_data.info.version }}</td>
                </tr>
                <tr>
                    <th>Chapters</th>
                    <td><ul>
                        <li v-for="(chapter, i) in chapters" :key="i">
                            {{ chapter.map }}
                        </li>
                    </ul></td>
                </tr>
                <tr>
                    <th>Description</th>
                    <td>{{ props.addon.addon_data.info.description }}</td>
                </tr>
            </table>
            <h4 class="title is-4">Workshop Info</h4>
            <table class="table is-striped is-fullwidth" v-if=" props.addon.workshop_info ">
                <tr>
                    <th>Workshop Link</th>
                    <td><a :href="workshopLink">{{ workshopLink }}</a></td>
                </tr>
                <tr>
                    <th>Favorites</th>
                    <td>{{ props.addon.workshop_info.favorited.toLocaleString() }}</td>
                </tr>
                <tr>
                    <th>Subscribers</th>
                    <td>{{ props.addon.workshop_info.subscriptions.toLocaleString() }}</td>
                </tr>
                
                <tr>
                    <th>Views</th>
                    <td>{{ props.addon.workshop_info.views.toLocaleString() }}</td>
                </tr>
                <tr>
                    <th>Up To Date</th>
                    <td>{{ uptoDateState ? 'Yes' : 'No' }}</td>
                </tr>
            </table>
        </section>
        <footer class="modal-card-foot">
            <div class="buttons">
                <b-button type="is-info" :loading="fetchingWorkshopInfo" v-if="props.addon.workshop_info && uptoDateState" @click="getLatestWorkshop">Check for Updates</b-button>
                <button class="button is-info" v-else-if=" props.addon.workshop_info && !uptoDateState ">Update</button>
                <button class="button is-danger" @click="deleteAddon">Delete</button>
                <button class="button is-warning" @click="toggleAddon"> {{ isDisabled ? 'Enable' : 'Disable' }}</button>

                <button class="button" @click="emit( 'close' )">Close</button>
            </div>
        </footer>
    </div>
</div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { formatBytes, formatDate, sendToast } from '../js/utils';
import AddonTags from './AddonTags.vue';
import { invoke } from '@tauri-apps/api';

const props = defineProps( ['addon'] )
const emit = defineEmits( ["close", "refresh"] )

let fetchingWorkshopInfo = ref(false)

const addonName = computed(() => {
    return props.addon.workshop_info?.title ?? props.addon.addon_info?.title ?? props.addon.filename 
} )

const chapters = computed( () => {
    const list = props.addon.addon_data?.mission_info?.modes?.coop
    console.log(list)
    if(!list) return
    const chapters = []
    for ( const chapter of Object.values(list) ) {
        chapters.push(chapter)
    }
    return chapters
} )

const isDisabled = computed( () => {
    return props.addon.file_name.endsWith(".disabled")
})

const workshopLink = computed( () => {
    const id = props.addon.workshop_info?.publishedfileid
    if(!id) return
    return `https://steamcommunity.com/sharedfiles/filedetails/?id=${id}`
} )

const uptoDateState = computed( () => {
    if ( !props.addon.workshop_info ) return "?"
    return Number(props.addon.workshop_info.time_updated) <= Number(props.addon.last_update_time)
} )

async function getLatestWorkshop() {
    fetchingWorkshopInfo.value = true
    try {
        const result = await invoke( "get_latest_workshop_info", {
            publishedfileid: Number( props.addon.workshop_info.publishedfileid )
        } )
        if ( result ) {
            emit( "refresh", result )
        }
        sendToast( {
            type: "is-success",
            message: "Updated workshop information"
        })
    } catch ( err ) {
        console.error( "failed", err )
        sendToast( {
            type: "is-danger",
            message: "<b>Fetching workshop information failed: </b>" + err.message
        } )
    } finally {
        fetchingWorkshopInfo.value = false
    }
}

async function toggleAddon() {
    await invoke( "toggle_addon", { file_name: props.addon.file_name } )
   
}
async function deleteAddon() {
    await invoke( "delete_addon", { file_name: props.addon.file_name } )
    sendToast( {
        type: "is-danger",
        message: `Deleted addon ${props.addon.file_name}`
    } )
    emit("close")
    // awa
}

</script>