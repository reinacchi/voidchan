<template>
  <div v-if="file?.code === 404">
    <NotFound />
  </div>
  <div v-else>
    <div class="flex flex-col items-center justify-center">
      <br class="noselect" ondragstart="return false" draggable="false">
      <h1 style="margin-top: 5%;" class="text-3xl font-extrabold noselect">{{ fileParam }}</h1>
      <br class="noselect" ondragstart="return false" draggable="false">
      <div class="image-view">
        <Image v-if="files?.nsfw" class="nsfw noselect" ondragstart="return false" draggable="false" :src="files.url" />
        <Image v-else class="noselect" ondragstart="return false" draggable="false" :src="files?.url" />
      </div>
      <p class="noselect">Uploaded by <nuxt-link class="text-violet-500" :to="'/users/' + files?.uploader?.name"><b>{{ files?.uploader?.name }}</b></nuxt-link> at <b>{{ moment(files?.date).format("D/MM/YY, h:mm:ss A") }}</b></p>
      <br class="noselect" ondragstart="return false" draggable="false">
      <div>
        <nuxt-link to="/"><button class="btn mx-2 noselect"><i class="fas fa-home"></i> Home</button></nuxt-link>
        <button class="btn mx-2 noselect" @click="downloadImage()"><i class="fas fa-cloud-download-alt"></i> Download</button>
      </div>
      <br class="noselect" ondragstart="return false" draggable="false">
      <div v-if="status === 'authenticated'">
        <div v-if="data?.user?.name === files?.uploader?.name">
          <h2 class="text-2xl p-5 font-bold noselect">Manage Your File</h2>
          <button class="btn mx-2 noselect" style="border-color: #ad0c00;" @click="deleteImage()"><i class="fas fa-trash-alt"></i> Delete Image</button>
          <button v-if="files?.nsfw" class="btn mx-2 noselect" style="border-color: #d8e43a;" @click="markNSFW(false)"><i class="fas fa-ban"></i> Mark as non-NSFW</button>
          <button v-else class="btn mx-2 noselect" style="border-color: #d8e43a;" @click="markNSFW(true)"><i class="fas fa-ban"></i> Mark as NSFW</button>
          <br class="noselect" ondragstart="return false" draggable="false">
          <br class="noselect" ondragstart="return false" draggable="false">
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import moment from "moment";

const fileParam = useRoute().params.file as string;
const { data, status } = useAuth();
const { data: file } = useFetch(`/raw/${fileParam}`);
const { data: files } = await useAsyncData("files", () => $fetch(`/api/files/${fileParam.split(".")[0]}`)) as any;
const config = useRuntimeConfig();

async function refreshPage() {
  try {
    await refreshNuxtData();
  } finally {}
}

function deleteImage() {
  useFetch(`/api/files/${fileParam.split(".")[0]}`, {
    method: "DELETE",
  });

  window.location.href = "/";
}

function downloadImage() {
  window.location.href = `/api/download/${fileParam}`;
}

function markNSFW(val: boolean) {
  useFetch(`/api/files/${fileParam.split(".")[0]}`, {
    method: "PATCH",
    body: JSON.stringify({
      nsfw: val,
    }),
  });

  window.location.reload();
}

useHead({
  title: `${fileParam} | VoidChan`,
  meta: [
    {
      property: "og:title",
      content: `VoidChan`,
    },
    {
      property: "og:site_name",
      content: `${fileParam} • Uploaded by ${files.value?.uploader?.name}`,
    },
    {
      property: "og:url",
      content: `${config.BaseURL}/v/${fileParam}`,
    },
    {
      property: "og:image",
      content: `${config.BaseURL}/raw/${fileParam}`,
    },
    {
      property: "theme-color",
      content: "#42B893",
    },
    {
      property: "twitter:card",
      content: "summary_large_image",
    },
  ],
});
</script>

<style scoped>
.image-view {
  max-width: 100%;
  border: none !important;
  box-shadow: 0 0 4px 2px rgb(60, 27, 109);
  margin: 20px;
  display: flex;
  display: inline-block;
  transition: all cubic-bezier(0.175, 0.885, 0.32, 1.275) 0.3s;
}

.image-view img {
  max-width: 100%;
  height: auto;
  object-fit: contain;
  vertical-align: auto;
}
</style>
