<template>
  <div class="noselect">
    <h1 class="text-4xl m-5 p-5 noselect">{{ data?.user?.name }}'s Posts</h1>
    <GridGallery />
    <br><br><br>
    <nuxt-link to="/"><button class="btn mx-2 noselect">Back</button></nuxt-link>
  </div>
</template>

<script setup lang="ts">
const { data } = useSession();

useHead({
  title: "Uploads | VoidChan"
})
</script>
