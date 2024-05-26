<template>
  <div class="noselect">
    <h1 class="text-4xl m-5 p-5 font-extrabold">Upload an Image</h1>
    <div class="hidden md:flex items-center justify-center w-full">
      <label
        class="flex flex-col items-center justify-center w-2/6 h-44 border-2 border-violet-900 border-opacity-40 shadow bg-violet-900 bg-opacity-20 rounded-lg cursor-pointer"
      >
        <div class="flex flex-col items-center justify-center pt-5 pb-6">
          <svg
            class="w-8 h-8 mb-4 text-gray-500 dark:text-gray-400"
            aria-hidden="true"
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 20 16"
          >
            <path
              stroke="currentColor"
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M13 13h3a3 3 0 0 0 0-6h-.025A5.56 5.56 0 0 0 16 6.5 5.5 5.5 0 0 0 5.207 5.021C5.137 5.017 5.071 5 5 5a4 4 0 0 0 0 8h2.167M10 15V6m0 0L8 8m2-2 2 2"
            />
          </svg>
          <p class="mb-2 text-sm text-gray-500 dark:text-gray-400">
            <span class="font-semibold">Click to upload</span> or drag and drop
          </p>
          <p class="text-xs text-gray-500 dark:text-gray-400">
            Max Size: 100 MB
          </p>
        </div>
        <input
          hidden
          accept=".png,.jpg,.jpeg,.gif,.mp4,.webp,.webm"
          @change="handleFileUpload"
          type="file"
        />
      </label>
    </div>
    <div class="md:hidden flex items-center justify-center w-full">
      <label
        class="flex flex-col items-center justify-center w-4/6 h-44 border-2 border-violet-900 border-opacity-40 shadow bg-violet-900 bg-opacity-20 rounded-lg cursor-pointer"
      >
        <div class="flex flex-col items-center justify-center pt-5 pb-6">
          <svg
            class="w-8 h-8 mb-4 text-gray-500 dark:text-gray-400"
            aria-hidden="true"
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 20 16"
          >
            <path
              stroke="currentColor"
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M13 13h3a3 3 0 0 0 0-6h-.025A5.56 5.56 0 0 0 16 6.5 5.5 5.5 0 0 0 5.207 5.021C5.137 5.017 5.071 5 5 5a4 4 0 0 0 0 8h2.167M10 15V6m0 0L8 8m2-2 2 2"
            />
          </svg>
          <p class="mb-2 text-sm text-gray-500 dark:text-gray-400">
            <span class="font-semibold">Click to upload</span> or drag and drop
          </p>
          <p class="text-xs text-gray-500 dark:text-gray-400">
            Max Size: 100 MB
          </p>
        </div>
        <input
          hidden
          accept=".png,.jpg,.jpeg,.gif,.mp4,.webp,.webm"
          @change="handleFileUpload"
          type="file"
        />
      </label>
    </div>
    <hr
      v-if="previewFile !== null"
      class="w-5/6 h-px mx-auto my-4 bg-violet-500 bg-opacity-20 border-0 rounded md:my-10"
    />
    <div class="hidden md:inline-flex" v-if="previewFile !== null">
      <div class="md:block pr-[10rem]">
        <Image :src="previewFile" class="image-view" draggable="false" />
        <p>{{ fileSize }} .{{ fileType }}</p>
      </div>
      <div class="h-[27rem] m-4 w-px bg-violet-500 bg-opacity-20"></div>
      <div class="mt-3 ml-2 pr-[5rem]">
        <div class="flex items-center">
          <label for="rating" class="mr-2 text-xl font-semibold">Rating:</label>
          <div class="relative inline-block group">
            <i class="fal fa-circle-question pr-2 cursor-pointer"></i>
            <div
              class="absolute mt-2 top-full left-1/2 transform -translate-x-1/2 bg-[#27183b] text-white px-2 py-1 text-xs rounded-md whitespace-nowrap opacity-0 invisible group-hover:opacity-100 group-hover:visible transition-opacity duration-300"
            >
              <span
                class="arrow-down ml-12 absolute top-0 left-1/2 transform -translate-x-1/2"
              ></span>
              <span>
                <b>Explicit</b>
                <br />
                Graphic sex or violence. Sex acts, exposed genitals (pussy,
                penis, anus), body fluids (cum, pussy juice).
                <br />
                <br />
                <b>Erotica</b>
                <br />
                Simple nudity or near-nudity. Bare nipples, ass, areolae,
                revealing clothes, cameltoes, etc. No sex or exposed genitals.
                <br />
                <br />
                <b>Suggestive</b>
                <br />
                Sexy or suggestive, even mildly so. Cleavage, breast or ass
                focus, swimsuits, underwear, skimpy clothes, etc. No nudity.
                <br />
                <br />
                <b>Safe</b>
                <br />
                100% safe. Nothing sexualised or inappropriate to view in front
                of others.
              </span>
            </div>
          </div>
          <div
            class="border bg-violet-900 bg-opacity-10 border-violet-900 rounded-md p-1 flex items-center"
          >
            <input
              type="radio"
              id="explicit"
              name="rating"
              value="explicit"
              class="hidden"
            />
            <label
              for="explicit"
              class="radio-label mr-2 cursor-pointer"
              @click="fileRating = 'explicit'"
              :style="{
                fontWeight: fileRating === 'explicit' ? 'bold' : 'normal',
              }"
              >Explicit</label
            >
            <div style="width: 5px"></div>
            <input
              type="radio"
              id="erotica"
              name="rating"
              value="erotica"
              class="hidden"
            />
            <label
              for="erotica"
              class="radio-label mr-2 cursor-pointer"
              @click="fileRating = 'erotica'"
              :style="{
                fontWeight: fileRating === 'erotica' ? 'bold' : 'normal',
              }"
              >Erotica</label
            >
            <div style="width: 5px"></div>
            <input
              type="radio"
              id="suggestive"
              name="rating"
              value="suggestive"
              class="hidden"
            />
            <label
              for="suggestive"
              class="radio-label mr-2 cursor-pointer"
              @click="fileRating = 'suggestive'"
              :style="{
                fontWeight: fileRating === 'suggestive' ? 'bold' : 'normal',
              }"
              >Suggestive</label
            >
            <div style="width: 5px"></div>
            <input
              type="radio"
              id="safe"
              name="rating"
              value="safe"
              class="hidden"
            />
            <label
              for="safe"
              class="radio-label cursor-pointer"
              @click="fileRating = 'safe'"
              :style="{ fontWeight: fileRating === 'safe' ? 'bold' : 'normal' }"
              >Safe</label
            >
          </div>
        </div>
        <br />
        <div class="inline-block">
          <label class="text-xl mr-[20rem] font-semibold">Tags</label>
          <br />
          <textarea
            v-model="fileTags"
            @keydown.space.prevent="handleSpace"
            @input="fetchTagsAutocomplete"
            class="shadow bg-violet-900 bg-opacity-10 border-violet-900 appearance-none border border-solid w-full h-full py-2 px-3 leading-tight focus:outline-none focus:shadow-outline"
          ></textarea>
          <div
            v-if="tagsSuggestions.length"
            class="absolute w-1/6 bg-[#231a31] border border-[#312644] shadow-lg"
          >
            <ul class="py-1">
              <li v-for="(suggestion, index) in tagsSuggestions" :key="index">
                <button
                  @click="selectTag(suggestion)"
                  class="block w-full text-xs px-4 py-2 text-violet-500"
                >
                  <span class="inline-flex justify-between w-full">
                    <span v-html="highlightMatch(suggestion.name)"></span>
                    <span>{{ suggestion.count }}</span>
                  </span>
                </button>
              </li>
            </ul>
          </div>
          <br /><br />
          <label class="text-xl mr-[18rem] font-semibold">Source</label>
          <br />
          <input
            class="shadow bg-violet-900 bg-opacity-10 border-violet-900 appearance-none border border-solid w-full h-6 py-2 px-3 leading-tight focus:outline-none focus:shadow-outline"
            v-model="fileSource"
            type="text"
          />
        </div>
        <br /><br />
        <button class="btn mx-2" @click="handleFileSubmit">Upload</button>
      </div>
    </div>
    <div class="md:hidden" v-if="previewFile !== null">
      <h1 class="text-3xl">Preview</h1>
      <Image :src="previewFile" class="image-view" draggable="false" />
      <p>{{ fileSize }} .{{ fileType }}</p>
      <hr
        v-if="previewFile !== null"
        class="w-5/6 h-px mx-auto my-4 bg-violet-500 bg-opacity-20 border-0 rounded md:my-10"
      />
      <br />
      <div class="flex items-center ml-12">
        <label for="rating" class="mr-2 text-xl font-semibold">Rating:</label>
        <div class="relative inline-block group">
          <i class="fal fa-circle-question pr-2 cursor-pointer"></i>
          <div
            class="absolute w-[15rem] mt-2 top-full left-1/2 transform -translate-x-1/2 bg-[#27183b] text-white px-2 py-1 text-xs rounded-md whitespace-pre-line opacity-0 invisible group-hover:opacity-100 group-hover:visible transition-opacity duration-300"
          >
            <span
              class="arrow-down ml-12 absolute top-0 left-1/2 transform -translate-x-1/2"
            ></span>
            <span>
              <b>Explicit</b>
              <br />
              Graphic sex or violence. Sex acts, exposed genitals (pussy, penis,
              anus), body fluids (cum, pussy juice).
              <br />
              <br />
              <b>Erotica</b>
              <br />
              Simple nudity or near-nudity. Bare nipples, ass, areolae,
              revealing clothes, cameltoes, etc. No sex or exposed genitals.
              <br />
              <br />
              <b>Suggestive</b>
              <br />
              Sexy or suggestive, even mildly so. Cleavage, breast or ass focus,
              swimsuits, underwear, skimpy clothes, etc. No nudity.
              <br />
              <br />
              <b>Safe</b>
              <br />
              100% safe. Nothing sexualised or inappropriate to view in front of
              others.
            </span>
          </div>
        </div>
        <div
          class="border bg-violet-900 bg-opacity-10 border-violet-900 rounded-md p-1 flex items-center"
        >
          <input
            type="radio"
            id="explicit"
            name="rating"
            value="explicit"
            class="hidden"
          />
          <label
            for="explicit"
            class="radio-label mr-2 cursor-pointer"
            @click="fileRating = 'explicit'"
            :style="{
              fontWeight: fileRating === 'explicit' ? 'bold' : 'normal',
            }"
            >Explicit</label
          >
          <div style="width: 5px"></div>
          <input
            type="radio"
            id="erotica"
            name="rating"
            value="erotica"
            class="hidden"
          />
          <label
            for="erotica"
            class="radio-label mr-2 cursor-pointer"
            @click="fileRating = 'erotica'"
            :style="{
              fontWeight: fileRating === 'erotica' ? 'bold' : 'normal',
            }"
            >Erotica</label
          >
          <div style="width: 5px"></div>
          <input
            type="radio"
            id="suggestive"
            name="rating"
            value="suggestive"
            class="hidden"
          />
          <label
            for="suggestive"
            class="radio-label mr-2 cursor-pointer"
            @click="fileRating = 'suggestive'"
            :style="{
              fontWeight: fileRating === 'suggestive' ? 'bold' : 'normal',
            }"
            >Suggestive</label
          >
          <div style="width: 5px"></div>
          <input
            type="radio"
            id="safe"
            name="rating"
            value="safe"
            class="hidden"
          />
          <label
            for="safe"
            class="radio-label cursor-pointer"
            @click="fileRating = 'safe'"
            :style="{ fontWeight: fileRating === 'safe' ? 'bold' : 'normal' }"
            >Safe</label
          >
        </div>
      </div>
      <br />
      <div class="flex ml-12 items-center">
        <label class="text-xl font-semibold">Tags</label>
      </div>
      <textarea
        v-model="fileTags"
        @input="fetchTagsAutocomplete"
        @keyword.space.prevent="handleSpace"
        class="shadow bg-violet-900 bg-opacity-10 border-violet-900 flex ml-12 appearance-none border border-solid w-4/5 h-full py-2 px-3 leading-tight focus:outline-none focus:shadow-outline"
      ></textarea>
      <div
        v-if="tagsSuggestions.length"
        class="absolute w-3/6 bg-[#231a31] border ml-12 border-[#312644] shadow-lg"
      >
        <ul class="py-1">
          <li v-for="(suggestion, index) in tagsSuggestions" :key="index">
            <button
              @click="selectTag(suggestion)"
              class="block w-full text-xs text-left px-4 py-2 text-violet-500"
            >
              <span class="inline-flex justify-between w-full">
                <span v-html="highlightMatch(suggestion.name)"></span>
                <span>{{ suggestion.count }}</span>
              </span>
            </button>
          </li>
        </ul>
      </div>
      <br /><br />
      <div class="flex ml-12 items-center">
        <label class="text-xl font-semibold">Source</label>
      </div>
      <input
        class="shadow bg-violet-900 bg-opacity-10 border-violet-900 flex ml-12 appearance-none border border-solid w-4/5 h-8 py-2 px-3 leading-tight focus:outline-none focus:shadow-outline"
        v-model="fileSource"
        type="text"
      />
      <br /><br />
      <button class="btn mx-2" @click="handleFileSubmit">Upload</button>
    </div>
    <br /><br />
  </div>
</template>

<script setup lang="ts">
import { bytesToSize } from "~~/utils/bytesToSize";

const file = ref<FileList | null>(null);
const previewFile = ref<string | null>(null);
const fileSize = ref<string | null>(null);
const fileType = ref<string | null>(null);
const fileRating = ref<string | null>(null);
const fileSource = ref<string>("");
const fileTags = ref<string>("");
const tagsSuggestions = ref<any>([]);
const { $toast } = useNuxtApp();

async function fetchTagsAutocomplete() {
  const query = fileTags.value.trim().split(" ").pop() as string;

  if (!query && /^\s*$/.test(query)) {
    tagsSuggestions.value = [];
    return;
  }

  const { data } = await useAsyncData("tags", () =>
    $fetch(`/api/tags?name=${query}`)
  );

  tagsSuggestions.value = data.value;
  console.log(query);
}

function selectTag(tag: any) {
  fileTags.value = fileTags.value.replace(/\S+$/, tag.name + " ");
  tagsSuggestions.value = [];
}

function handleSpace() {
  fileTags.value += " ";
  tagsSuggestions.value = [];
}

function highlightMatch(tagName: string) {
  const query = fileTags.value.trim().split(" ").pop() as string;

  const escapedLastWord = query.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const regex = new RegExp(`(${escapedLastWord})`, "gi");

  return tagName.replace(regex, '<span class="font-black">$1</span>');
}

function handleFileUpload(event: Event) {
  file.value = (event.target as HTMLInputElement).files;

  const reader = new FileReader();
  reader.onload = (e) => {
    previewFile.value = e.target?.result as string;
  };
  /* @ts-ignore */
  reader.readAsDataURL(file.value[0]);
  fileSize.value = bytesToSize((file as any).value[0].size);
  fileType.value = (file as any).value[0].type.split("/")[1];
}

async function handleFileSubmit() {
  const formData = new FormData();
  if (file.value) {
    Array.from(file.value).forEach((value) => {
      if (value.size > 104857600) {
        return $toast.warning("The image size is too big!");
      }

      formData.append(value.name.split(".")[0], value);
    });
  }

  if (!file.value) return $toast.error("No file uploaded!");
  if (!fileRating.value)
    return $toast.error("Please choose the rating of the file selected!");

  await $fetch("/api/posts", {
    method: "POST",
    body: {
      fileBuffer: previewFile.value,
      fileRating: fileRating.value,
      fileSize: file.value[0].size,
      fileType: file.value[0].type,
      fileSource: fileSource.value,
      fileTags: fileTags.value.trim().split(/\s+/),
    },
  }).then(() => useRouter().push("/posts"));
}
useHead({
  title: "New Upload | VoidChan",
});
</script>

<style scoped>
.image-view {
  max-width: 80%;
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
