<template>
<div class="mx-5">
    <nav class="navbar" role="navigation" aria-label="main navigation">
        <div class="navbar-menu">
            <b-field label="Sort">
                <b-select placeholder="Select a name">
                    <option v-for=" option in data " :value="option.id" :key="option.id">
                        {{ option.user.first_name }}
                    </option>
                </b-select>
            </b-field>
        </div>
        <div class="navbar-end">
            <b-input :loading="loadState === LoadState.Loading" v-model="query" icon="search" expanded required native-type="text"
    placeholder="Find an item" rounded />
        </div>
    </nav>
    <div class="box" v-if="loadState != LoadState.Downloading">
        <div class="columns is-variable is-8">
            <div class="column">
                <form @submit.prevent="searchWorkshop">
                <div class="field">
                    <label class="label">Search an item</label>
                    <div class="field has-addons ">
                        <b-input 
                            :loading="loadState === LoadState.Loading" 
                            v-model="query" 
                            icon="search"
                            expanded
                            required 
                            native-type="text" 
                            placeholder="Find an item" 
                        />
                        <p class="control">
                            <input type="submit" class="button is-info" :disabled="searching || !query || query.length == 0" value="Search" />
                        </p>
                    </div>
                </div>
                </form>
            </div>
            <div class="column">
                <form @submit.prevent="fetchItem">
                <div class="field">
                    <label class="label">Enter a link</label>
                    <div class="field has-addons">
                        <p class="control has-icons-left">
                           <b-input 
                                :loading="loadState === LoadState.Loading" 
                                v-model="manualInput" 
                                icon="link"
                                expanded
                                required 
                                native-type="text" 
                                placeholder="Enter an url" 
                            />
                        </p>
                        <p class="control">
                            <input type="submit" :class="['button', 'is-info', { 'is-loading': loadState === LoadState.Loading }]" :disabled="loadState !== LoadState.Idle" value="Fetch" />
                        </p>
                    </div>
                </div>
                </form>
            </div>
        </div>
    </div>
    <div class="box" v-else>
        <h4 class="title is-4">Installing...</h4>
        <!-- <progress class="progress" :value="installProgress.val" :max="installProgress.max" /> -->
    </div>
    <!-- <div class="box" v-if="item">
        <div class="columns">
            <div class="column is-8">
                <a class="button" v-if="searchResults" @click="item = null">
                    <font-awesome-icon icon="long-arrow-alt-left" />&nbsp;
                    Return to search
                </a>
                <h4 class="title is-4">{{item.title}}</h4>
                <span v-html="descriptionHTML" />
            </div>
            <div class="column is-4">
                <img :src="item.preview_url" :alt="item.title" />
                <br>
                <div class="tags">
                    <span class="tag is-dark">{{formatNumber(item.favorited)}} favorites</span>
                    <span class="tag is-dark">{{formatNumber(item.views)}} views</span>
                    <span class="tag is-link">{{formatBytes(item.file_size)}}</span>
                </div>
                <div class="buttons">
                    <b-button v-if="installState < 2" type="is-info" @click="install" :loading="installState == 1">Install</b-button>
                    <p class="has-text-success" v-if="installState == 2">Installed&nbsp;</p>
                    <a class="button is-secondary" target="_blank" :href="'https://steamcommunity.com/sharedfiles/filedetails/?id=' + item.publishedfileid">
                        Open Page
                    </a>
                </div>
                <div class="tags">
                    <span class="tag" v-for="tag in item.tags" :key="tag.tag">{{tag.tag}}</span>
                </div>
            </div>
        </div>
    </div> -->
    <div class="box" v-if="searchResults">
        <!-- TODO: Add Details -->
        <b-table :data="searchResults" detailed>

            <b-table-column field="title" label="Name" v-slot="props">
                {{ props.row.title }}
            </b-table-column>

            <b-table-column field="file_size" label="File Size" v-slot="props">
                {{ formatBytes(props.row.file_size) }}
            </b-table-column>

            <b-table-column v-slot="props">
                <b-button size="is-small" type="is-info" @click="selectItem(props.row)">Select</b-button>
            </b-table-column>

            <template #detail="props">
                {{props.row.file_description}}
            </template>
        </b-table>
    </div>
    
</div>
</template>

<script setup lang="ts">
const WORKSHOP_REGEX = new RegExp(/https:\/\/steamcommunity.com\/(workshop|sharedfiles)\/filedetails\/\?id=(\d+)/);
// import bbobHTML from '@bbob/html'
// import presetHTML5 from '@bbob/preset-html5'

import { formatBytes, formatDate } from '@/js/utils'
import { invoke } from '@tauri-apps/api/tauri'
import { listen } from '@tauri-apps/api/event'
import { computed, ref } from 'vue';
import { sendToast } from '../../js/utils';

enum LoadState {
    Idle,
    Loading,
    Downloading,
    Error,
    Done
}

let query = ref()
let loadState = ref<LoadState>(LoadState.Idle)
let selectedItem = ref()
let searchResults = ref()
let manualInput = ref()

let sortBy = ref()

const selectedDescription = computed( () => {
    if ( !selectedItem.value ) return
    return ""
    // return bbobHTML(selectedItem.value.description || selectedItem.value.file.description, presetHTML5())
} )

async function searchWorkshop() {
    loadState.value = LoadState.Loading
    try {
        searchResults.value = await invoke( "search_items", { query: query.value } )
        loadState.value = LoadState.Done
    } catch ( err ) {
        loadState.value = LoadState.Error  
        sendToast( {
            type: "is-danger",
            message: `<b>Failed to search steam workshop: </b>${err.message}`
        })
    } 
}

</script>