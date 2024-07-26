<template>
<div class="container">
    <div class="box" v-if="hasChanged">
        <p>You have unsaved changes</p>
        <div class="buttons">
            <b-button type="is-success" @click="save">Save</b-button>
            <b-button type="is-danger" @click="cancel">Cancel</b-button>
        </div>
    </div>
    <div class="mx-5 mt-4 box">
        <b-field label="Addons Folder">
            <b-input v-model="changed.gamedir"></b-input>
        </b-field>
        <b-button type="is-info" @click="openBrowse">Browse</b-button>
        <br><br>

        <b-field label="Steam API Key" message="Is required to check for updates and fetch information from the workshop">
            <b-input v-model="changed.steam_apikey"></b-input>
        </b-field>

        <div v-if=" !changed.steam_apikey">
            <p>An api key is required to check for addon updates, fetch information and download from the workshop.</p>
            <p>You can get your api key from <a href="https://steamcommunity.com/dev/apikey">https://steamcommunity.com/dev/apikey</a>, enter any domain, and paste it above</p>
        </div>

    </div>
</div>
</template>

<script setup lang="ts">

import { invoke } from '@tauri-apps/api/tauri'
import { computed, onMounted, ref } from 'vue';
const props = defineProps( ["settings"] )
const emit = defineEmits( ["saved"] )
import { SnackbarProgrammatic as Snackbar } from 'buefy'

let changed = ref<Record<string, any>>( {
    telemetry: null,
    gamedir: "",
    steam_apikey: "",
} )

const hasChanged = computed( () => {
    for ( const item in props.settings ) {
        if ( props.settings[item] !== changed.value[item] ) return true
    }
    return false
} )

async function save() {
    try {
        await invoke( 'save_settings', { changed: changed.value } )
        emit( "saved", changed.value )
        alert("Saved settings")
        // Snackbar.open( {
        //     type: "is-success",
        //     message: "Settings were saved successfully"
        // } )
    } catch ( err ) {
        alert("Could not save")
        // Snackbar.open( {
        //     type: "is-danger",
        //     message: "Could not save changes. " + (err as any).message
        // } )
        console.log( err )
    }
}
function cancel() {
    changed.value = Object.assign( {}, props.settings )
}

function openBrowse() {
    invoke("open_browse")
}

onMounted( () => {
    cancel()
})
</script>

<style scoped>
li {
    margin-bottom: 1em;
}
</style>