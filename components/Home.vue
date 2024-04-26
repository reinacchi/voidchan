<template>
  <div class="centered">
    <h1 class="text-2xl p-5 noselect">
      <span class="font-extrabold">VoidChan</span>
    </h1>
    <div>
      <div v-if="status === 'authenticated'">
        <p class="noselect">
          Hello, <b>{{ data?.user?.name }}</b>.
        </p>
        <br class="noselect" /><br class="noselect" />
        <nuxt-link :to="'/users/' + data?.user?.name + '/uploads'"
          ><button
            class="btn mx-2 noselect"
            style="color: #7308ff; border-color: #7308ff"
          >
            My Uploads
          </button></nuxt-link
        >
        <nuxt-link to="/users/settings"
          ><button
            class="btn mx-2 noselect"
            style="color: #9ff331; border-color: #9ff331"
          >
            Settings
          </button></nuxt-link
        >
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
/* @ts-ignore */
const { data, status, signIn, signOut } = useSession();

</script>
