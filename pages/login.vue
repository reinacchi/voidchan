<template>
  <Login />
</template>

<script setup lang="ts">
/* @ts-ignore */
definePageMeta({
  auth: {
    unauthenticatedOnly: true,
    navigateAuthenticatedTo: "/"
  }
});
</script>
