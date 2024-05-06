<template>
<div v-if="!files">
  <NotFound />
</div>
<div class="hidden md:block">
  <div class="gallery-view noselect" v-for="file in files" :key="file.id">
    <nuxt-link v-if="file.nsfw" :href="'/v/' + file.id + '.' + file.ext"><Image style="cursor: pointer;" class="nsfw" :src="file.url" draggable="false" alt="" /></nuxt-link>
    <nuxt-link v-else :href="'/v/' + file.id + '.' + file.ext"><Image style="cursor: pointer;" :src="file.url" draggable="false" alt="" /></nuxt-link>
  </div>
</div>
<div class="md:hidden">
  <div class="gallery-view-mb noselect md:hidden" v-for="file in files" :key="file.id">
    <nuxt-link v-if="file.nsfw" :href="'/v/' + file.id + '.' + file.ext"><Image style="cursor: pointer;" class="nsfw" :src="file.url" draggable="false" alt="" /></nuxt-link>
    <nuxt-link v-else :href="'/v/' + file.id + '.' + file.ext"><Image style="cursor: pointer;" :src="file.url" draggable="false" alt="" /></nuxt-link>
  </div>
</div>
</template>

<script setup lang="ts">
const user = useRoute().params.user;
const { data: files } = await useAsyncData("files", () => $fetch(`/api/users/${user}/files`));
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

.gallery-view-mb {
  max-width: 40%;
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

.gallery-view-mb:hover {
  box-shadow: 0 0 10px 5px rgb(60, 27, 109);
  border: 20px;
  color: rgb(60, 27, 109);
  transform: translateY(-0.30em);
}

.gallery-view-mb img {
  max-width: 100%;
  height: auto;
  object-fit: contain;
  vertical-align: auto;
}
</style>
