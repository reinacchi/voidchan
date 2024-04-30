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
  link: [
    {
      rel: "stylesheet",
      href: "https://site-assets.fontawesome.com/releases/v6.1.1/css/all.css",
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
