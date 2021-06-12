<template>
<div class="container">
    <div class="box">
        <p>
            <b><u>Addons Folder</u></b> {{settings.gamedir}}
            <a class="button is-small is-info is-inline is-pulled-right">Update</a>
        </p>
    </div>
</div>
</template>

<script>
export default {
    props: ['settings']
}
</script>