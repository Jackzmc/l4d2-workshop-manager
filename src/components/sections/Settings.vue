<template>
<div class="container">
    <div class="mt-2 box">
        <div class="field">
            <div class="field is-horizontal">
                <div class="field-label is-normal">
                    <label class="label">Addons Folder</label>
                </div>
                <div class="field-body">
                    <div class="field has-addons has-addons-centered">
                        <p class="control">
                            <input class="input" style="width:600px" type="text" :value="settings.gamedir">
                        </p>
                        <p class="control">
                            <a class="button is-info">
                            Update
                            </a>
                        </p>
                    </div>
                </div>
            </div>
        </div>
    </div>
</div>
</template>

<script>
export default {
    props: ['settings']
}
</script>

<style scoped>
li {
    margin-bottom: 1em;
}
</style>