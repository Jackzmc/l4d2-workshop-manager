<template>
<context-menu ref="itemMenu" v-if="item">
    <li :class="['context-item', {'disabled': item.enabled}]">
        <component 
            :is="!item.enabled ? 'a' : 'p'" 
            @click="enableItem"
        >
            Enable
        </component>
    </li>
   <li :class="['context-item', {'disabled': !item.enabled}]">
        <component 
            class="px-5"
            @click="disableItem"
            :is="item.enabled ? 'a' : 'p'" 
        >
            Disable
        </component>
    </li>
    <li class="context-item"><a @click="$emit('uninstall')">Uninstall</a></li>
    <li class="context-item"><a @click="$emit('select')">Select</a></li>
</context-menu>
</template>

<script>
import contextMenu from 'vue-context-menu'

export default {
    props: ['item'],
    components: {
        contextMenu
    },
    methods: {
        open() {
            this.$refs.itemMenu.open()
        },
        close() {
            this.$refs.itemMenu.close()
        },
        enableItem() {
            if(!this.item.enabled)
                this.$emit('enable')
        },
        disableItem() {
            if(this.item.enabled)
                this.$emit('disable')
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
    padding: 1.5em;
    color: #167df0
}
.context-item a:hover {
    padding: 1.5em;
    color: black
}
</style>