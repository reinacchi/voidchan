<template>
  <img
    v-if="!failed"
    ref="element"
    :src="src"
  />
</template>
<script setup lang="ts">
defineProps<{
  src: string;
}>();

const element = ref<HTMLImageElement | undefined>();
const loading = ref(true);
const failed = ref(false);

onMounted(() => {
  element.value?.addEventListener('load', handleImageLoad);
  element.value?.addEventListener('error', handleImageLoadError);
});

onBeforeUnmount(() => {
  element.value?.removeEventListener('load', handleImageLoad);
  element.value?.removeEventListener('error', handleImageLoadError);
});

function handleImageLoad() {
  loading.value = false;
}

function handleImageLoadError() {
  failed.value = true;
}
</script>
