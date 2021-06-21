<template>
<div>
    <div class="box has-background-dark has-text-white">
        <div class="columns">
            <div class="column">
                Showing 0-{{items.length}} of 0 entries
            </div>
            <div class="column is-pulled-right mr-6">
                <b-field grouped>
                    <b-field horizontal>
                        <label class="label has-text-white">Sort by </label>
                        <b-select v-model="options.sort" size="is-small">
                            <option value="popular">Most Popular</option>
                            <option value="subscribed">Most Subscribed</option>
                            <option value="recent">Recent</option>
                        </b-select>
                    </b-field>
                    <b-field horizontal>
                        <label class="label has-text-white">Over time period </label>
                        <b-select v-model="options.time" size="is-small">
                            <option value="today">Today</option>
                            <option value="1week">Last Week</option>
                            <option value="1month">Last Month</option>
                            <option value="3month">3 Months</option>
                            <option value="6month">6 Months</option>
                            <option value="1year">1 Year</option>
                            <option value="alltime">All Time</option>
                        </b-select>
                    </b-field>
                </b-field>
            </div>
            <div class="column is-2">

            </div>
        </div>
    </div>
    <div class="container">
        <div class="columns is-multiline">
            <div class="column is-3" v-for="item in items" :key="item.publishedfileid">
                    <figure class="image is-1by1" >
                        <img :src="item.preview_url" :alt="item.title">
                    </figure>
                <p class="item-title subtitle">{{item.title}}<br> by {{item.author}}</p>
            </div>
        </div>
    </div>
</div>
</template>

<script>
export default {
    props: ['query'],
    data() {
        return {
            options: {
                sort: 'popular',
                time: '1week'
            },
            items: []
        }
    },
    methods: {
        search() {
            this.$parent.$emit('searching', true)
            fetch(`https://jackz.me/l4d2/scripts/search_public.php?page=1&numperpage=30&search_text=${this.query}&appid=550&return_details=1&return_vote_data=1`)
            .then(r => r.json())
            .then(json => {
                if(json.response.total > 0) {
                    this.items = json.response.publishedfiledetails
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
            .finally(() => this.$parent.$emit('searching', false))
        }
    },
    created() {
        
    }
}
</script>

<style scoped>
.item-title {
    text-overflow: ellipsis;
    overflow: hidden;
    white-space: nowrap;
}
</style>