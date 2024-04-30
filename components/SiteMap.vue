<template>
<div class="noselect md:block">
  <h1 class="text-4xl font-bold mt-12 flex ml-12">Site Map</h1>
  <p class="text-sm mt-4 flex ml-12">Are you lost? Perhaps you're in the right place...</p>
  <div>
    <h2 class="text-xl font-bold mt-12 flex ml-12">Posts</h2>
    <nuxt-link to="/posts" class="text-base flex ml-12 text-violet-500">Posts</nuxt-link>
    <nuxt-link to="/posts/upload" class="text-base flex ml-12 text-violet-500">Upload New</nuxt-link>
  </div>
  <div>
    <h2 class="text-xl font-bold mt-12 flex ml-12">Profile</h2>
    <nuxt-link :to="'/users/' + session?.user.name" class="text-base flex ml-12 text-violet-500">Profile</nuxt-link>
    <nuxt-link to="/users/settings" class="text-base flex ml-12 text-violet-500">Settings</nuxt-link>
    <nuxt-link :to="'/users/' + session?.user.name + '/uploads'" class="text-base flex ml-12 text-violet-500">Uploads</nuxt-link>
    <nuxt-link :to="'/users/' + session?.user.name + '/shares'" class="text-base flex ml-12 text-violet-500">Shares</nuxt-link>
  </div>
</div>
</template>

<script setup lang="ts">
const { data: session } = useAuth() as any;

useHead({
  title: "Site Map | VoidChan"
})
</script>
