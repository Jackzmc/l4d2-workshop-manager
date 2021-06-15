<template>
<div class="mx-5 mt-4">
    <div class="box">
        <div class="columns is-variable is-8">
            <div class="column">
                <form @submit.prevent="search">
                <div class="field">
                    <label class="label">Search an item</label>
                    <div class="field has-addons ">
                        <p class="control has-icons-left is-expanded">
                            <input v-model="query" required class="input" type="text" placeholder="Find an item">
                            <span class="icon is-small is-left">
                                <font-awesome-icon icon="search" aria-hidden="true" />
                            </span>
                        </p>
                        <p class="control">
                            <input type="submit" class="button is-info" :disabled="!query || query.length == 0" value="Search" />
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
                            <input required v-model="url" class="input" type="text" placeholder="Enter an url">
                            <span class="icon is-small is-left">
                                <font-awesome-icon icon="link" aria-hidden="true" />
                            </span>
                        </p>
                        <p class="control">
                            <input type="submit" class="button is-info" :disabled="!url || url.length == 0" value="Load">
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
                    <b-button type="is-info">Install Addon</b-button>
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

export default {
    data() {
        return {
            active: false,
            query: null,
            url: null,
            searchResults: null,
            item: null
        }
    },
    computed: {
        descriptionHTML() {
            if(!this.item) return null
            return bbobHTML(this.item.file_description, presetHTML5())
        }
    },
    methods: {
        formatBytes, 
        formatDate,
        toggle() {
            this.active = !this.active
        },
        search() {
            fetch(`https://jackz.me/l4d2/scripts/search_public.php?page=1&numperpage=20&search_text=${this.query}&appid=550&return_details=1`)
            .then(r => r.json())
            .then(json => {
                this.searchResults = json.response.publishedfiledetails
            })
        },
        fetchItem() {
            console.log(this.url)
        },
        select(item) {
            this.item = item
        },
        formatNumber(inp) {
            return inp.toLocaleString()
        }
    },
    watch: {
        url(url) {
            const match = url.match(WORKSHOP_REGEX)
            if(match) {
                this.url = match[2]
            }else if(!isNaN(parseInt(url))) {
                this.fetchItem()
            }else{
                this.url = this.url.replace(/\D/g,'')
            }
        }
    }
}
</script>