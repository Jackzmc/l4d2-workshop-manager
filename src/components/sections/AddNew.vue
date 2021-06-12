<template>
<div class="card">
    <header class="card-header  has-background-success " @click="toggle">
    <p class="card-header-title has-text-white" style="cursor: pointer">
        Add New Item
    </p>
    <a class="card-header-icon has-text-white" aria-label="more options">
        <font-awesome-icon :icon="active ? 'angle-up' : 'angle-down'" size="lg" aria-hidden="true" />
    </a>
    </header>
    <div class="card-content" v-if="active">
    <div class="content">
        
    </div>
    </div>
</div>
</template>

<script>
import { formatBytes, formatDate } from '@/js/utils'
export default {
    data() {
        return {
            active: false,
        }
    },
    computed: {
        
    },
    methods: {
        formatBytes, 
        formatDate,
        toggle() {
            this.active = !this.active
        }
    }
}
</script>