<template>
<div class="mx-5 mt-4">
    <div class="box">
        <div class="columns is-variable is-8">
            <div class="column">
                <form @submit.prevent="search">
                <div class="field">
                    <label class="label">Search an item</label>
                    <div class="field has-addons ">
                        <b-input 
                            :loading="searching" 
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
                                :loading="fetching" 
                                v-model="id" 
                                icon="link"
                                expanded
                                required 
                                native-type="text" 
                                placeholder="Enter an url" 
                            />
                        </p>
                        <p class="control">
                            <input type="submit" class="button is-info" :disabled="fetching || !id || id.length == 0" value="Load" />
                        </p>
                    </div>
                </div>
                </form>
            </div>
        </div>
    </div>
    <div class="box" v-if="item">
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
                    <b-button v-if="installState < 2" type="is-info" @click="install" :loading="installState == 1">Install Addon</b-button>
                    <p class="has-text-success" v-if="installState">Installed</p>
                    <a class="button is-secondary" target="_blank" :href="'https://steamcommunity.com/sharedfiles/filedetails/?id=' + item.publishedfileid">
                        Open Page
                    </a>
                </div>
                <div class="tags">
                    <span class="tag" v-for="tag in item.tags" :key="tag.tag">{{tag.tag}}</span>
                </div>
            </div>
        </div>
    </div>
    <div class="box" v-else-if="searchResults">
        <!-- TODO: Add Details -->
        <b-table :data="searchResults" detailed>

            <b-table-column field="title" label="Name" v-slot="props">
                {{ props.row.title }}
            </b-table-column>

            <b-table-column field="file_size" label="File Size" v-slot="props">
                {{ formatBytes(props.row.file_size) }}
            </b-table-column>

            <b-table-column v-slot="props">
                <b-button size="is-small" type="is-info" @click="select(props.row)">Select</b-button>
            </b-table-column>

            <template #detail="props">
                {{props.row.file_description}}
            </template>
        </b-table>
    </div>
    
</div>
</template>

<script>
const WORKSHOP_REGEX = new RegExp(/https:\/\/steamcommunity.com\/(workshop|sharedfiles)\/filedetails\/\?id=(\d+)/);
import bbobHTML from '@bbob/html'
import presetHTML5 from '@bbob/preset-html5'

import { formatBytes, formatDate } from '@/js/utils'
import { invoke } from '@tauri-apps/api/tauri'


export default {
    data() {
        return {
            query: null,
            id: null,
            searchResults: null,
            item: null,
            installState: 0, //0->Inactive, 1->Installing, 2->Installed
            fetching: false,
            searching: false
        }
    },
    computed: {
        descriptionHTML() {
            if(!this.item) return null
            return bbobHTML(this.item.file_description || this.item.description, presetHTML5())
        }
    },
    methods: {
        formatBytes, 
        formatDate,
        search() {
            this.item = null
            this.searching = true
            fetch(`https://jackz.me/l4d2/scripts/search_public.php?page=1&numperpage=20&search_text=${this.query}&appid=550&return_details=1`)
            .then(r => r.json())
            .then(json => {
                if(json.response.total > 0) {
                    this.searchResults = json.response.publishedfiledetails
                }else{
                    this.$buefy.snackbar.open({
                        duration: 5000,
                        message: 'Could not find any item matching your query',
                        type: 'is-warning',
                        position: 'is-bottom-left',
                        queue: false,
                    })
                }
            })
            .catch(err => {
                this.$buefy.snackbar.open({
                    duration: 5000,
                    message: '<b>Search failed: </b>' + err.message,
                    type: 'is-danger',
                    position: 'is-bottom-left',
                    actionText: 'Retry',
                    queue: false,
                    onAction: () => {
                        this.search()
                    }
                })
            })
            .finally(() => this.searching = false)
        },
        fetchItem() {
            this.fetching = true
            const params = new URLSearchParams();
            params.append("itemcount", 1)
            params.append("publishedfileids[0]", this.id)

            fetch(`https://proxy.jackz.me/api.steampowered.com/ISteamRemoteStorage/GetPublishedFileDetails/v1`, {
                method: 'POST',
                body: params
            })
            .then(r => r.json())
            .then(json => {
                if(json.response.resultcount > 0) {
                    this.select(json.response.publishedfiledetails[0])
                    this.id = null
                }else{
                    this.$buefy.snackbar.open({
                        duration: 5000,
                        message: 'Could not find that item',
                        type: 'is-warning',
                        position: 'is-bottom-left',
                        queue: false,
                    })
                }
            })
            .catch(err => {
                this.$buefy.snackbar.open({
                    duration: 5000,
                    message: '<b>Fetched failed: </b>' + err.message,
                    type: 'is-danger',
                    position: 'is-bottom-left',
                    actionText: 'Retry',
                    queue: false,
                    onAction: () => {
                        this.fetchItem()
                    }
                })
            })
            .finally(() => this.fetching = false)
        },
        async select(item) {
            this.item = item
            const installInfo = await invoke("get_install_info", { id: item.publishedfileid.toString() })
            console.log('got info', installInfo)
            this.installState = installInfo ? 2 : 0
        },
        formatNumber(inp) {
            return inp.toLocaleString()
        },
        async install() {
            this.installState = 1
            try {
                await invoke('download_addon', { item: this.item })
                this.item = null
            } catch(err) {
                this.$buefy.dialog.alert({
                    title: 'Install Failed',
                    message: 'An error occurred while installing this addon:<br>' + err.message,
                    type: 'is-danger',
                    ariaRole: 'alertdialog',
                    ariaModal: true
                })
                console.error('Installing failure: ', err)
            }
            this.installState = 0
        }
    },
    watch: {
        id(url) {
            if(!url) return
            const match = url.match(WORKSHOP_REGEX)
            if(match) {
                this.id = match[2]
            }else if(!isNaN(parseInt(url))) {
                this.fetchItem()
            }else{
                this.id = this.url.replace(/\D/g,'')
            }
        }
    }
}
</script>