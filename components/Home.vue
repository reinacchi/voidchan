<template>
  <div class="centered">
    <h1 class="text-4xl p-5 noselect">
      Void<span class="font-extrabold">Chan</span>
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
