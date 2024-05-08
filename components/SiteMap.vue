<template>
<div class="noselect md:block">
  <h1 class="text-4xl font-bold mt-12 flex ml-12">Site Map</h1>
  <p class="text-sm mt-4 flex ml-12">Are you lost? Perhaps you're in the right place...</p>
  <div>
    <h2 class="text-xl font-bold mt-12 flex ml-12">Posts</h2>
    <p class="text-base flex ml-12 text-violet-500"><nuxt-link to="/posts">Posts</nuxt-link></p>
    <p class="text-base flex ml-12 text-violet-500"><nuxt-link to="/posts/upload">Upload New</nuxt-link></p>
  </div>
  <div>
    <h2 class="text-xl font-bold mt-12 flex ml-12">Profile</h2>
    <div v-if="status === 'authenticated'">
      <P class="text-base flex ml-12 text-violet-500"><nuxt-link :to="'/users/' + session?.user.name">Profile</nuxt-link></P>
      <p class="text-base flex ml-12 text-violet-500"><nuxt-link to="/users/settings">Settings</nuxt-link></p>
      <p class="text-base flex ml-12 text-violet-500"><nuxt-link :to="'/users/' + session?.user.name + '/uploads'">Uploads</nuxt-link></p>
      <p class="text-base flex ml-12 text-violet-500"><nuxt-link :to="'/users/' + session?.user.name + '/shares'">Shares</nuxt-link></p>
      <button @click="signOut({ callbackUrl: '/' })" class="text-base hover:underline cursor-pointer flex ml-12 text-violet-500">Logout</button>
    </div>
    <div v-else>
      <button @click="signIn('')" class="text-base hover:underline cursor-pointer flex ml-12 text-violet-500">Login</button>
    </div>
  </div>
</div>
</template>

<script setup lang="ts">
const { data: session, signIn, signOut, status } = useAuth() as any;

useHead({
  title: "Site Map | VoidChan"
})
</script>
