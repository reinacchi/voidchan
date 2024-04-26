<template>
  <div>
    <Navbar />
    <NuxtLoadingIndicator color="#7308ff" :duration="10000" />
    <NuxtPage />
  </div>
</template>

<script setup lang="ts">
  useHead({
    title: "VoidChan",
    script: [
      {
        src: "https://kit.fontawesome.com/e931be9ca1.js",
      },
    ],
    meta: [
      {
        name: "description",
        content:
          "VoidChan is an extensive collection of images. Upload your files and share them with your friends.",
      },
      {
        name: "keywords",
        content: "voidchan, voidchan cdn, voidchan sharex, voidchan sharex cd",
      },
    ],
  });
</script>
