<template>
<div class="noselect md:block">
  <h1 class="text-4xl font-bold mt-12 flex ml-12">Site Map</h1>
  <p class="text-sm mt-4 flex ml-12">Are you lost? Perhaps you're in the right place...</p>
  <div class="hidden md:flex space-x-20 fixed ml-12 mt-12">
    <section class="flex-auto space-y-6">
    <ul>
      <li><h2 class="text-xl font-bold text-left">Posts</h2></li>
      <li class="text-base text-violet-500 text-left"><nuxt-link to="/posts">Posts</nuxt-link></li>
      <li class="text-base text-violet-500 text-left"><nuxt-link to="/posts/upload">Upload New</nuxt-link></li>
    </ul>
    <ul>
      <li><h2 class="text-xl font-bold text-left">Profile</h2></li>
      <div v-if="status === 'authenticated'">
        <li class="text-base text-violet-500 text-left"><nuxt-link :to="'/users/' + session?.user.name">Profile</nuxt-link></li>
        <li class="text-base text-violet-500 text-left"><nuxt-link to="/users/settings">Settings</nuxt-link></li>
        <li class="text-base text-violet-500 text-left"><nuxt-link :to="'/users/' + session?.user.name + '/uploads'">Uploads</nuxt-link></li>
        <li class="text-base text-violet-500 text-left"><nuxt-link :to="'/users/' + session?.user.name + '/shares'">Shares</nuxt-link></li>
        <li class="text-left"><button class="text-base text-violet-500 hover:underline cursor-pointer" @click="signOut({ callbackUrl: '/' })">Logout</button></li>
      </div>
      <div v-else>
        <li class="text-left"><button class="text-base text-violet-500 hover:underline cursor-pointer" @click="signIn('')">Login</button></li>
      </div>
    </ul>
  </section>
  <section class="flex-auto space-y-6">
    <ul>
      <li><h2 class="text-xl font-bold text-left">Users</h2></li>
      <li class="text-base text-violet-500 text-left"><nuxt-link to="/users">Users</nuxt-link></li>
    </ul>
  </section>
</div>
<div class="md:hidden space-y-6 ml-12 mt-12">
    <section class="flex-auto space-y-6">
    <ul>
      <li><h2 class="text-xl font-bold text-left">Posts</h2></li>
      <li class="text-base text-violet-500 text-left"><nuxt-link to="/posts">Posts</nuxt-link></li>
      <li class="text-base text-violet-500 text-left"><nuxt-link to="/posts/upload">Upload New</nuxt-link></li>
    </ul>
    <ul>
      <li><h2 class="text-xl font-bold text-left">Profile</h2></li>
      <div v-if="status === 'authenticated'">
        <li class="text-base text-violet-500 text-left"><nuxt-link :to="'/users/' + session?.user.name">Profile</nuxt-link></li>
        <li class="text-base text-violet-500 text-left"><nuxt-link to="/users/settings">Settings</nuxt-link></li>
        <li class="text-base text-violet-500 text-left"><nuxt-link :to="'/users/' + session?.user.name + '/uploads'">Uploads</nuxt-link></li>
        <li class="text-base text-violet-500 text-left"><nuxt-link :to="'/users/' + session?.user.name + '/shares'">Shares</nuxt-link></li>
        <li class="text-left"><button class="text-base text-violet-500 hover:underline cursor-pointer" @click="signOut({ callbackUrl: '/' })">Logout</button></li>
      </div>
      <div v-else>
        <li class="text-left"><button class="text-base text-violet-500 hover:underline cursor-pointer" @click="signIn('')">Login</button></li>
      </div>
    </ul>
    </section>
    <section class="flex-auto space-y-6">
    <ul>
      <li><h2 class="text-xl font-bold text-left">Users</h2></li>
      <li class="text-base text-violet-500 text-left"><nuxt-link to="/users">Users</nuxt-link></li>
    </ul>
    </section>

  </div>
</div>
</template>

<script setup lang="ts">
const { data: session, signIn, signOut, status } = useAuth() as any;

useHead({
  title: "Site Map | VoidChan"
})
</script>
