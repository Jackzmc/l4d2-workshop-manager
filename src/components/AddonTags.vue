<template>
<div class="tags" v-if="addon.addon_data?.info">
    <span class="tag is-primary" v-if=" addon.addon_data.info.content_campaign || workshopTags.includes( 'campaigns' ) || workshopTags.includes('co-op') ">Campaign</span>
    <span class="tag is-primary" v-if="workshopTags.includes('versus')">Versus</span>
    <span class="tag is-primary" v-if="workshopTags.includes('survival')">Survival</span>
    <span class="tag is-success" v-if=" addon.addon_data.info.content_script ">Script</span>
    <span class="tag is-black" v-if=" addon.addon_data.info.content_weapon || workshopTags.includes('weapons') ">Weapon</span>
    <span class="tag is-black" v-if=" addon.addon_data.info.content_skin || workshopTags.includes('textures')">Skin</span>
    <span class="tag is-info" v-if=" addon.addon_data.info.content_music ">Music</span>
    <span class="tag is-info" v-if=" addon.addon_data.info.content_sound || workshopTags.includes( 'sounds' ) ">Sound</span>
    <span class="tag is-danger" v-if=" workshopTags.includes( 'ui' ) ">UI</span>
</div>
</template>

<script setup lang="ts">
import { computed } from 'vue';

const props = defineProps( ['addon'] )

const workshopTags = computed( () => {
    if ( !props.addon.workshop_info ) return []
    return props.addon.workshop_info.tags.map(t => t.tag.toLowerCase())
})
</script>