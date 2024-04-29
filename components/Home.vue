<template>
  <div class="centered">
    <h1 class="text-2xl p-5 noselect">
      <span class="font-extrabold">VoidChan</span>
    </h1>
    <div>
      <div v-if="status === 'authenticated'">
        <p class="noselect">
          Hello, <b>{{ user.displayName }}</b>.
        </p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
/* @ts-ignore */
const { data, status} = useAuth();
const { data: user } = status.value === "authenticated" ? await useFetch(`/api/users/${data.value?.user?.name}`): {} as any;
</script>
