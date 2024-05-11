<template>
<div class="noselect md:block">
  <h1 class="text-4xl font-bold mt-12 ml-12 text-left ">Users</h1>
  <div class="mx-10 w-5/6 m-5">
    <div class="overflow-x-auto">
      <table class="min-w-full border-violet-900 border shadow-full bg-violet-900 bg-opacity-20">
        <thead>
          <tr>
            <th class="text-left py-2 px-4">Username</th>
            <th class="text-left py-2 px-4">Level</th>
            <th class="text-left py-2 px-4">Joined</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="user in users" :key="user.name">
            <td class="text-left border-t border-violet-900 px-4 py-2"><nuxt-link class="text-violet-500" :to="'/users/' + user.name">{{ user.name }}</nuxt-link></td>
            <td class="text-left border-t border-violet-900 px-4 py-2">{{ user.clearanceLevel[0] }}</td>
            <td class="text-left border-t border-violet-900 px-4 py-2">{{ moment(user.createdAt).format("DD-MM-YYYY HH:MM") }}</td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</div>
</template>

<script setup lang="ts">
import moment from "moment";

const { data: users } = await useAsyncData("users", () => $fetch(`/api/users`));

useHead({
  title: "Users | VoidChan"
});
</script>
