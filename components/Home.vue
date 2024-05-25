<template>
  <div class="centered noselect">
    <h1 class="text-5xl p-5">
      <span class="font-sans text-accent font-black tracking-widest">VoidChan</span>
    </h1>
    <div>
      <div v-if="status === 'authenticated'">
        <p>
          Hello, <b>{{ user.display_name }}</b>.
        </p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
/* @ts-ignore */
const { data, status} = useAuth();
const { data: user } = status.value === "authenticated" ? await useFetch(`/api/users/${data.value?.user?.name}`): {} as any;

useHead({
  title: "VoidChan"
});
</script>
