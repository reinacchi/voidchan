<template>
  <nav class="max-w-full bg-[#251a36]">
    <div class="noselect max-w-full mx-auto px-4 sm:px-6 lg:px-8">
      <div class="flex items-center justify-between h-16">
        <div class="flex items-center">
          <div class="flex-shrink-0">
            <nuxt-link to="/"><img class="h-14 w-14" src="../assets/img/voidchan.png" draggable="false"></nuxt-link>
          </div>
          <div class="ml-4 text-white">
            <nuxt-link class="hover:no-underline" to="/"><span class="text-xl text-gray-300 font-sans font-black">VoidChan</span></nuxt-link>
          </div>
          <div class="hidden md:block">
            <div class="ml-10 flex items-baseline space-x-2">
              <nuxt-link @click="toPosts" class="text-gray-300 font-sans hover:cursor-pointer hover:text-white hover:no-underline px-3 py-2 rounded-md text-sm font-medium"><i class="fas fa-book mr-1"></i> Posts</nuxt-link>
              <nuxt-link v-if="status === 'authenticated'" @click="toMyShares" class="text-gray-300 font-sans hover:cursor-pointer hover:text-white hover:no-underline px-3 py-2 rounded-md text-sm font-medium"><i class="fas fa-share mr-1"></i> My Shares</nuxt-link>
              <nuxt-link v-if="status === 'authenticated'" @click="toMyProfile" class="text-gray-300 font-sans hover:cursor-pointer hover:text-white hover:no-underline px-3 py-2 rounded-md text-sm font-medium"><i class="fas fa-user mr-1"></i> My Profile</nuxt-link>
              <nuxt-link v-if="status === 'authenticated'" @click="toUpload" class="text-gray-300 font-sans hover:cursor-pointer hover:text-white hover:no-underline px-3 py-2 rounded-md text-sm font-medium"><i class="fas fa-square-plus mr-1"></i> Upload New</nuxt-link>
              <nuxt-link @click="toMore" class="text-gray-300 font-sans hover:cursor-pointer hover:text-white hover:no-underline px-3 py-2 rounded-md text-sm font-medium">More »</nuxt-link>
            </div>
          </div>
        </div>
        <div class="flex items-center space-x-4">
          <div class="hidden md:flex">
            <button v-if="status === 'unauthenticated'" @click="signIn('')" class="text-gray-300 font-sans hover:text-white px-3 py-2 rounded-md text-base font-medium">Login</button>
            <button v-else @click="signOut({ callbackUrl: '/' })" class="text-gray-300 font-sans hover:text-white px-3 py-2 rounded-md text-base font-medium">Logout</button>
          </div>
          <div class="md:hidden">
            <button @click="toggleMenu" type="button" class="inline-flex items-center justify-center p-2 rounded-md text-gray-400 focus:outline-none focus:bg-gray-700 focus:text-white">
            <svg class="h-6 w-6" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path :class="{ 'hidden': isOpen, 'inline-flex': !isOpen }" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16m-7 6h7"></path>
              <path :class="{ 'hidden': !isOpen, 'inline-flex': isOpen }" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
            </svg>
          </button>
          </div>
        </div>
      </div>
    </div>
    <div :class="{ 'block': isOpen, 'hidden': !isOpen }" class="md:hidden noselect">
      <div class="px-2 pt-2 pb-3 space-y-1 sm:px-3">
        <nuxt-link @click="toPosts" class="text-gray-300 font-sans hover:cursor-pointer hover:text-white block hover:no-underline px-3 py-2 rounded-md text-base font-medium"><i class="fas fa-book mr-1"></i> Posts</nuxt-link>
        <nuxt-link v-if="status === 'authenticated'" @click="toMyShares" class="text-gray-300 font-sans hover:cursor-pointer hover:text-white block hover:no-underline px-3 py-2 rounded-md text-base font-medium"><i class="fas fa-share mr-1"></i> My Shares</nuxt-link>
        <nuxt-link v-if="status === 'authenticated'" @click="toMyProfile" class="text-gray-300 font-sans hover:cursor-pointer hover:text-white block hover:no-underline px-3 py-2 rounded-md text-base font-medium"><i class="fas fa-user mr-1"></i> My Profile</nuxt-link>
        <nuxt-link v-if="status === 'authenticated'" @click="toUpload" class="text-gray-300 font-sans hover:cursor-pointer hover:text-white block hover:no-underline px-3 py-2 rounded-md text-base font-medium"><i class="fas fa-square-plus mr-1"></i> Upload New</nuxt-link>
        <nuxt-link @click="toMore" class="text-gray-300 font-sans hover:cursor-pointer hover:text-white block hover:no-underline px-3 py-2 rounded-md text-base font-medium">More »</nuxt-link>
        <hr class="border-violet-500 border-opacity-20">
        <span v-if="status === 'unauthenticated'" @click="signIn('')" class="text-gray-300 font-sans hover:text-white block px-3 py-2 rounded-md text-base font-medium">Login</span>
        <span v-else @click="signOut({ callbackUrl: '/' })" class="text-gray-300 font-sans hover:text-white block px-3 cursor-pointer py-2 rounded-md text-base font-medium">Logout</span>
      </div>
    </div>
  </nav>
</template>

<script setup lang="ts">
const isOpen = ref(false);
const { data, status, signIn, signOut } = useAuth();

function toggleMenu() {
  isOpen.value = !isOpen.value;
}
function toPosts() {
  useRouter().push("/posts");
}
function toMyShares() {
  useRouter().push(`/users/${data?.value?.user?.name}/shares`);
}
function toMyProfile() {
  useRouter().push(`/users/${data?.value?.user?.name}`);
}
function toUpload() {
  useRouter().push("/posts/upload");
}
function toMore() {
  useRouter().push("/sitemap");
}
</script>
