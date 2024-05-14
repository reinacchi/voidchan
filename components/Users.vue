<template>
<div class="noselect md:block">
  <h1 class="text-4xl font-bold mt-12 ml-12 text-left">Users</h1>
  <h2 class="text-base mt-2 ml-12 text-left">Pages {{ currentPage }} / {{ totalPages }}</h2>
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
          <tr v-for="user in currentPageUsers" :key="user.name">
            <td class="text-left border-t border-violet-900 px-4 py-2"><nuxt-link class="text-violet-500" :to="'/users/' + user.name">{{ user.name }}</nuxt-link></td>
            <td class="text-left border-t border-violet-900 px-4 py-2">{{ user.clearanceLevel[0] }}</td>
            <td class="text-left border-t border-violet-900 px-4 py-2">{{ moment(user.createdAt).format("DD-MM-YYYY HH:MM") }}</td>
          </tr>
        </tbody>
      </table>
    </div>
    <div class="p-5 space-x-3 flex">
      <button class="btn" @click="previousPage" :disabled="currentPage === 1">Previous</button>
      <button class="btn" @click="nextPage" :disabled="currentPage === totalPages">Next</button>
    </div>
  </div>
</div>
</template>

<script setup lang="ts">
import moment from "moment";

const { data: users } = await useAsyncData("users", () => $fetch(`/api/users`));
const usersPerPage = 25;
const totalUsers = users.value?.length;
const currentPage = ref(1);

const totalPages = computed(() => Math.ceil((totalUsers as number) / usersPerPage));
const currentPageUsers = computed(() => {
  const startIndex = (currentPage.value - 1) * usersPerPage;
  const endIndex = startIndex + usersPerPage;

  return users.value?.slice(startIndex, endIndex);
});

function nextPage() {
  if (currentPage.value < totalPages.value) {
    currentPage.value++;
  }
}

function previousPage() {
  if (currentPage.value > 1) {
    currentPage.value--;
  }
}

useHead({
  title: "Users | VoidChan"
});
</script>
