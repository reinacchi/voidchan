<template>
<div class="gallery-view noselect" v-for="file in files?.reverse()" :key="file.id">
  <nuxt-link v-if="file.nsfw" :href="'/v/' + file.id + '.' + file.ext"><img style="cursor: pointer;" class="nsfw" :src="file.url" draggable="false" alt=""></nuxt-link>
  <nuxt-link v-else :href="'/v/' + file.id + '.' + file.ext"><img style="cursor: pointer;" :src="file.url" draggable="false" alt=""></nuxt-link>
</div>
</template>

<script setup lang="ts">
const user = useRoute().params.user;
const { data: files } = useFetch(`/api/users/${user}/files`);
</script>

<style scoped>
.gallery-view {
  max-width: 20%;
  border: none !important;
  box-shadow: 0 0 4px 2px rgb(60, 27, 109);
  margin: 20px;
  display: flex;
  display: inline-block;
  transition: all cubic-bezier(0.175, 0.885, 0.32, 1.275) 0.3s;
}

.gallery-view:hover {
  box-shadow: 0 0 10px 5px rgb(60, 27, 109);
  border: 20px;
  color: rgb(60, 27, 109);
  transform: translateY(-0.30em);
}

.gallery-view img {
  max-width: 100%;
  height: auto;
  object-fit: contain;
  vertical-align: auto;
}
</style>
