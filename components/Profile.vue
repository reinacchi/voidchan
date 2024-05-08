<template>
  <div class="hidden noselect md:block">
    <h1 class="text-4xl font-bold mt-12 flex items-center ml-12">
      {{ user.displayName }}
      <div
        v-if="user.clearanceLevel.includes('Project Lead')"
        class="relative group inline-block ml-2"
      >
        <i
          class="text-3xl fal fa-leaf-maple cursor-pointer text-accent"
        ></i>
        <div
          class="absolute -top-8 left-1/2 transform -translate-x-1/2 bg-[#27183b] text-white px-2 py-1 invisible text-xs rounded-md whitespace-nowrap opacity-0 group-hover:opacity-100 group-hover:visible transition-opacity duration-300"
        >
          <span
            class="arrow-up absolute bottom-full left-1/2 transform -translate-x-1/2"
          ></span>
          <span class="text-accent">Project Lead</span>
        </div>
      </div>
      <div
        v-if="user.clearanceLevel.includes('Developer')"
        class="relative group inline-block ml-2"
      >
        <i
          class="text-3xl fal fa-code cursor-pointer text-teal-500"
        ></i>
        <div
          class="absolute -top-8 left-1/2 transform -translate-x-1/2 bg-[#27183b] text-white px-2 py-1 invisible text-xs rounded-md whitespace-nowrap opacity-0 group-hover:opacity-100 group-hover:visible transition-opacity duration-300"
        >
          <span
            class="arrow-up absolute bottom-full left-1/2 transform -translate-x-1/2"
          ></span>
          Developer
        </div>
      </div>
      <div
        v-if="user.clearanceLevel.includes('Moderator')"
        class="relative group inline-block ml-2"
      >
        <i
          class="text-3xl fal fa-shield cursor-pointer text-red-500"
        ></i>
        <div
          class="absolute -top-8 left-1/2 transform -translate-x-1/2 bg-[#27183b] text-white px-2 py-1 invisible text-xs rounded-md whitespace-nowrap opacity-0 group-hover:opacity-100 group-hover:visible transition-opacity duration-300"
        >
          <span
            class="arrow-up absolute bottom-full left-1/2 transform -translate-x-1/2"
          ></span>
          Moderator
        </div>
      </div>
      <div
        v-if="user.clearanceLevel.includes('Nominator')"
        class="relative group inline-block ml-2"
      >
        <i
          class="text-3xl fal fa-gavel cursor-pointer text-orange-500"
        ></i>
        <div
          class="absolute -top-8 left-1/2 transform -translate-x-1/2 bg-[#27183b] text-white px-2 py-1 invisible text-xs rounded-md whitespace-nowrap opacity-0 group-hover:opacity-100 group-hover:visible transition-opacity duration-300"
        >
          <span
            class="arrow-up absolute bottom-full left-1/2 transform -translate-x-1/2"
          ></span>
          Nominator
        </div>
      </div>
      <div
        v-if="user.clearanceLevel.includes('Contributor')"
        class="relative group inline-block ml-2"
      >
        <i
          class="text-3xl fal fa-brackets-curly cursor-pointer text-green-500"
        ></i>
        <div
          class="absolute -top-8 left-1/2 transform -translate-x-1/2 bg-[#27183b] text-white px-2 py-1 invisible text-xs rounded-md whitespace-nowrap opacity-0 group-hover:opacity-100 group-hover:visible transition-opacity duration-300"
        >
          <span
            class="arrow-up absolute bottom-full left-1/2 transform -translate-x-1/2"
          ></span>
          Contributor
        </div>
      </div>
      <div
        v-if="user.clearanceLevel.includes('Verified')"
        class="relative group inline-block ml-2"
      >
        <i
          class="text-3xl fal fa-badge-check cursor-pointer text-blue-500"
        ></i>
        <div
          class="absolute -top-8 left-1/2 transform -translate-x-1/2 bg-[#27183b] text-white px-2 py-1 invisible text-xs rounded-md whitespace-nowrap opacity-0 group-hover:opacity-100 group-hover:visible transition-opacity duration-300"
        >
          <span
            class="arrow-up absolute bottom-full left-1/2 transform -translate-x-1/2"
          ></span>
          Verified Account
        </div>
      </div>
      <div
        v-if="user.clearanceLevel.includes('Plus')"
        class="relative group inline-block ml-2"
      >
        <i
          class="text-3xl fal fa-crown cursor-pointer text-yellow-500"
        ></i>
        <div
          class="absolute -top-8 left-1/2 transform -translate-x-1/2 bg-[#27183b] text-white px-2 py-1 invisible text-xs rounded-md whitespace-nowrap opacity-0 group-hover:opacity-100 group-hover:visible transition-opacity duration-300"
        >
          <span
            class="arrow-up absolute bottom-full left-1/2 transform -translate-x-1/2"
          ></span>
          Plus Account
        </div>
      </div>
      <div
        v-if="user.clearanceLevel.includes('Supporter')"
        class="relative group inline-block ml-2"
      >
        <i
          class="text-3xl fal fa-heart cursor-pointer text-pink-500"
        ></i>
        <div
          class="absolute -top-8 left-1/2 transform -translate-x-1/2 bg-[#27183b] text-white px-2 py-1 invisible text-xs rounded-md whitespace-nowrap opacity-0 group-hover:opacity-100 group-hover:visible transition-opacity duration-300"
        >
          <span
            class="arrow-up absolute bottom-full left-1/2 transform -translate-x-1/2"
          ></span>
          Supporter
        </div>
      </div>
      <div
        v-if="user.clearanceLevel.includes('Alumni')"
        class="relative group inline-block ml-2"
      >
        <i
          class="text-3xl fal fa-dice cursor-pointer text-gray-500"
        ></i>
        <div
          class="absolute -top-8 left-1/2 transform -translate-x-1/2 bg-[#27183b] text-white px-2 py-1 invisible text-xs rounded-md whitespace-nowrap opacity-0 group-hover:opacity-100 group-hover:visible transition-opacity duration-300"
        >
          <span
            class="arrow-up absolute bottom-full left-1/2 transform -translate-x-1/2"
          ></span>
          Alumni
        </div>
      </div>
    </h1>

    <br /><br />
    <div class="flex ml-28 items-center text-gray-300">
      <h2 class="text-lg font-semibold ml-12">Username:</h2>
      <h2 class="text-lg ml-14">
        {{ user.name }}
        <span v-if="session?.user.name === user.name"
          >(<nuxt-link to="/users/settings" class="text-violet-500"
            >edit</nuxt-link
          >)</span
        >
      </h2>
    </div>
    <br v-if="session?.user.name === user.name" />
    <div
      v-if="session?.user.name === user.name"
      class="relative ml-28 flex items-center text-gray-300"
    >
      <h2 class="text-lg font-semibold ml-[88px]">Email:</h2>
      <h2 class="text-lg ml-14">{{ session?.user.email }}</h2>
    </div>
    <br />
    <div class="flex ml-28 items-center text-gray-300">
      <h2 class="text-lg font-semibold ml-[58px]">Joined At:</h2>
      <h2 class="text-lg ml-14">
        {{ moment(user.createdAt).format("DD-MM-YYYY") }}
      </h2>
    </div>
    <br />
    <div class="flex ml-[114px] items-center text-gray-300">
      <h2 class="text-lg font-semibold">Clearance Level:</h2>
      <h2 class="text-lg ml-14">{{ user.clearanceLevel[0] }}</h2>
    </div>
    <br />
    <div class="flex ml-28 items-center text-gray-300">
      <h2 class="text-lg font-semibold ml-[86px]">Posts:</h2>
      <h2 class="text-lg ml-14">{{ user.posts.length }}</h2>
    </div>
    <br /><br /><br />
    <div class="ml-6 text-gray-300">
      <h2 class="text-2xl font-semibold mr-[60rem]">Uploads</h2>
    </div>
    <div class="flex items-center ml-[8rem]">
      <div class="gallery-view noselect" v-for="post in posts" :key="post.id">
        <nuxt-link :href="'/posts/' + post.id"
          ><Image
            style="cursor: pointer"
            :src="post.buffer"
            draggable="false"
            alt=""
        /></nuxt-link>
      </div>
    </div>
  </div>
  <div class="noselect md:hidden">
    <h1 class="text-4xl font-bold mt-12 block ml-12">
      {{ user.displayName }}
      <div
        v-if="user.clearanceLevel.includes('Project Lead')"
        class="relative group inline-block ml-2"
      >
        <i
          class="text-3xl fal fa-leaf-maple cursor-pointer text-accent"
        ></i>
        <div
          class="absolute -top-8 left-1/2 transform -translate-x-1/2 bg-[#27183b] text-white px-2 py-1 invisible text-xs rounded-md whitespace-nowrap opacity-0 group-hover:opacity-100 group-hover:visible transition-opacity duration-300"
        >
          <span
            class="arrow-up absolute bottom-full left-1/2 transform -translate-x-1/2"
          ></span>
          <span class="text-accent">Project Lead</span>
        </div>
      </div>
      <div
        v-if="user.clearanceLevel.includes('Developer')"
        class="relative group inline-block ml-2"
      >
        <i
          class="text-3xl fal fa-code cursor-pointer text-teal-500"
        ></i>
        <div
          class="absolute -top-8 left-1/2 transform -translate-x-1/2 bg-[#27183b] text-white px-2 py-1 invisible text-xs rounded-md whitespace-nowrap opacity-0 group-hover:opacity-100 group-hover:visible transition-opacity duration-300"
        >
          <span
            class="arrow-up absolute bottom-full left-1/2 transform -translate-x-1/2"
          ></span>
          Developer
        </div>
      </div>
      <div
        v-if="user.clearanceLevel.includes('Moderator')"
        class="relative group inline-block ml-2"
      >
        <i
          class="text-3xl fal fa-shield cursor-pointer text-red-500"
        ></i>
        <div
          class="absolute -top-8 left-1/2 transform -translate-x-1/2 bg-[#27183b] text-white px-2 py-1 invisible text-xs rounded-md whitespace-nowrap opacity-0 group-hover:opacity-100 group-hover:visible transition-opacity duration-300"
        >
          <span
            class="arrow-up absolute bottom-full left-1/2 transform -translate-x-1/2"
          ></span>
          Moderator
        </div>
      </div>
      <div
        v-if="user.clearanceLevel.includes('Nominator')"
        class="relative group inline-block ml-2"
      >
        <i
          class="text-3xl fal fa-gavel cursor-pointer text-orange-500"
        ></i>
        <div
          class="absolute -top-8 left-1/2 transform -translate-x-1/2 bg-[#27183b] text-white px-2 py-1 invisible text-xs rounded-md whitespace-nowrap opacity-0 group-hover:opacity-100 group-hover:visible transition-opacity duration-300"
        >
          <span
            class="arrow-up absolute bottom-full left-1/2 transform -translate-x-1/2"
          ></span>
          Nominator
        </div>
      </div>
      <div
        v-if="user.clearanceLevel.includes('Contributor')"
        class="relative group inline-block ml-2"
      >
        <i
          class="text-3xl fal fa-brackets-curly cursor-pointer text-green-500"
        ></i>
        <div
          class="absolute -top-8 left-1/2 transform -translate-x-1/2 bg-[#27183b] text-white px-2 py-1 invisible text-xs rounded-md whitespace-nowrap opacity-0 group-hover:opacity-100 group-hover:visible transition-opacity duration-300"
        >
          <span
            class="arrow-up absolute bottom-full left-1/2 transform -translate-x-1/2"
          ></span>
          Contributor
        </div>
      </div>
      <div
        v-if="user.clearanceLevel.includes('Verified')"
        class="relative group inline-block ml-2"
      >
        <i
          class="text-3xl fal fa-badge-check cursor-pointer text-blue-500"
        ></i>
        <div
          class="absolute -top-8 left-1/2 transform -translate-x-1/2 bg-[#27183b] text-white px-2 py-1 invisible text-xs rounded-md whitespace-nowrap opacity-0 group-hover:opacity-100 group-hover:visible transition-opacity duration-300"
        >
          <span
            class="arrow-up absolute bottom-full left-1/2 transform -translate-x-1/2"
          ></span>
          Verified Account
        </div>
      </div>
      <div
        v-if="user.clearanceLevel.includes('Plus')"
        class="relative group inline-block ml-2"
      >
        <i
          class="text-3xl fal fa-crown cursor-pointer text-yellow-500"
        ></i>
        <div
          class="absolute -top-8 left-1/2 transform -translate-x-1/2 bg-[#27183b] text-white px-2 py-1 invisible text-xs rounded-md whitespace-nowrap opacity-0 group-hover:opacity-100 group-hover:visible transition-opacity duration-300"
        >
          <span
            class="arrow-up absolute bottom-full left-1/2 transform -translate-x-1/2"
          ></span>
          Plus Account
        </div>
      </div>
      <div
        v-if="user.clearanceLevel.includes('Supporter')"
        class="relative group inline-block ml-2"
      >
        <i
          class="text-3xl fal fa-heart cursor-pointer text-pink-500"
        ></i>
        <div
          class="absolute -top-8 left-1/2 transform -translate-x-1/2 bg-[#27183b] text-white px-2 py-1 invisible text-xs rounded-md whitespace-nowrap opacity-0 group-hover:opacity-100 group-hover:visible transition-opacity duration-300"
        >
          <span
            class="arrow-up absolute bottom-full left-1/2 transform -translate-x-1/2"
          ></span>
          Supporter
        </div>
      </div>
      <div
        v-if="user.clearanceLevel.includes('Alumni')"
        class="relative group inline-block ml-2"
      >
        <i
          class="text-3xl fal fa-dice cursor-pointer text-gray-500"
        ></i>
        <div
          class="absolute -top-8 left-1/2 transform -translate-x-1/2 bg-[#27183b] text-white px-2 py-1 invisible text-xs rounded-md whitespace-nowrap opacity-0 group-hover:opacity-100 group-hover:visible transition-opacity duration-300"
        >
          <span
            class="arrow-up absolute bottom-full left-1/2 transform -translate-x-1/2"
          ></span>
          Alumni
        </div>
      </div>
    </h1>
    <br /><br />
    <div class="flex ml-14 items-center text-gray-300">
      <h2 class="text-lg font-semibold ml-12">Username:</h2>
      <h2 class="text-lg ml-14">
        {{ user.name }}
        <span v-if="session?.user.name === user.name"
          >(<nuxt-link to="/users/settings" class="text-violet-500"
            >edit</nuxt-link
          >)</span
        >
      </h2>
    </div>
    <br v-if="session?.user.name === user.name" />
    <div
      v-if="session?.user.name === user.name"
      class="flex ml-14 items-center text-gray-300"
    >
      <h2 class="text-lg font-semibold ml-[88px]">Email:</h2>
      <h2 class="text-lg ml-14">{{ session?.user.email }}</h2>
    </div>
    <br />
    <div class="flex ml-14 items-center text-gray-300">
      <h2 class="text-lg font-semibold ml-[56px]">Joined At:</h2>
      <h2 class="text-lg ml-14">
        {{ moment(user.createdAt).format("DD-MM-YYYY") }}
      </h2>
    </div>
    <br />
    <div class="flex ml-[48px] items-center text-gray-300">
      <h2 class="text-lg font-semibold">Clearance Levels:</h2>
      <h2 class="text-lg ml-14">{{ user.clearanceLevel[0] }}</h2>
    </div>
    <br />
    <div class="flex ml-14 items-center text-gray-300">
      <h2 class="text-lg font-semibold ml-[84px]">Posts:</h2>
      <h2 class="text-lg ml-14">{{ user.posts.length }}</h2>
    </div>
    <br /><br /><br />
    <div class="ml-6 text-gray-300">
      <h2 class="text-2xl font-semibold mr-[25rem]">Uploads</h2>
    </div>
    <div class="flex items-center ml-[3rem]">
      <div
        class="gallery-view-small noselect"
        v-for="post in posts"
        :key="post.id"
      >
        <nuxt-link :href="'/posts/' + post.id"
          ><Image
            style="cursor: pointer"
            :src="post.buffer"
            draggable="false"
            alt=""
        /></nuxt-link>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import moment from "moment";

const param = useRoute().params.user;
const { data: session } = useAuth() as any;
const { data: user } = (await useFetch(`/api/users/${param}`, {
  method: "GET",
})) as any;
const { data: posts } = await useAsyncData("posts", () =>
  $fetch(`/api/users/${param}/posts`)
);

useHead({
  title: `${user.value.name} | VoidChan`,
});
</script>

<style scoped>
.gallery-view {
  max-width: 15%;
  border: none !important;
  box-shadow: 0 0 4px 2px rgb(60, 27, 109);
  margin: 20px;
  display: flex;
  display: inline-block;
  transition: all cubic-bezier(0.175, 0.885, 0.32, 1.275) 0.3s;
}

.gallery-view:hover,
.gallery-view-small:hover {
  box-shadow: 0 0 10px 5px rgb(60, 27, 109);
  border: 20px;
  color: rgb(60, 27, 109);
  transform: translateY(-0.3em);
}

.gallery-view img {
  max-width: 100%;
  height: auto;
  object-fit: contain;
  vertical-align: auto;
}

.gallery-view-small {
  max-width: 30%;
  border: none !important;
  box-shadow: 0 0 4px 2px rgb(60, 27, 109);
  margin: 20px;
  display: flex;
  display: inline-block;
  transition: all cubic-bezier(0.175, 0.885, 0.32, 1.275) 0.3s;
}

.gallery-view-small img {
  max-width: 100%;
  height: auto;
  object-fit: contain;
  vertical-align: auto;
}
</style>
