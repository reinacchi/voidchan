<template>
  <div class="noselect">
    <h1 class="text-4xl m-5 p-5 noselect">{{ user.username }}'s Shared Files</h1>
    <FileGridGallery />
    <br><br><br>
    <nuxt-link to="/"><button class="btn mx-2 noselect">Back</button></nuxt-link>
  </div>
</template>

<script setup lang="ts">
const param = useRoute().params.user;
const { data: user } = useFetch(`/api/users/${param}`) as any;

useHead({
  title: "Shares | VoidChan"
})
</script>
