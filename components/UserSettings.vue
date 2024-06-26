<template>
  <div class="noselect">
    <h1 class="text-5xl m-5">Settings</h1>
    <br class="">
    <center>
      <div class="w-full max-w-xs">
        <form class="shadow border border-violet-900 border-opacity-60 rounded-2xl px-8 pt-6 pb-8 mb-4">
          <div class="mb-4">
            <label
              class="block text-2xl font-bold mb-3"
              for="password"
            >
              New Password
            </label>
            <p class="text-sm mb-5">Change your old password with a new one if you have forgotten yours.</p>
            <input
              class="shadow bg-violet-900 bg-opacity-10 border-violet-900 border-opacity-60 appearance-none border rounded-lg w-full py-2 px-3 leading-tight focus:outline-none focus:shadow-outline"
              v-model="password"
              type="password"
              placeholder="Password"
            />
          </div>
          <button class="btn mx-2" @click.prevent="updateSettings">Update</button>
          </form>
          </div>
          <div class="w-full max-w-sm shadow border border- border-violet-900 border-opacity-60 rounded-2xl px-8 pt-6 pb-8 mb-4">
            <label class="block text-2xl font-bold mb-2">Authentication Key</label>
            <p class="mb-5 text-sm">Your authentication key is utilised for accessing VoidChan's ShareX uploader. <br><b>Keep your key confidential! Use the "Reset Key" to reset your authentication key.</b></p>
            <button @click="copyAuth" class="btn mx-2"><span class="far fa-clipboard"></span> Copy Key</button>
            <button @click="resetKey" style="border-color: #ad0c00;" class="btn mx-2"><span class="fas fa-exclamation-triangle"></span> Reset Key</button>
          </div>
          <button class="btn mx-2" @click.prevent="$router.go(-1)">Back</button>
    </center>
  </div>
</template>

<script setup lang="ts">
const config = useAppConfig();
const password = ref(undefined);
const newKey = ref<string | undefined>(undefined);
const { data } = useAuth();
const { $toast } = useNuxtApp();

function resetKey() {
  newKey.value = generateString(32);

  $fetch(`/api/users/${data.value?.user?.name}`, {
    method: "PATCH",
    headers: {
      authorization: config.PrivateAuth,
    },
    body: JSON.stringify({
      auth_key: newKey.value,
    }),
  });

  $toast.success("Authentication successfully reset!");
}

function updateSettings() {
  if (password.value === undefined) {
    return $toast.error("Please type your new password!");
  }

  $fetch(`/api/users/${data.value?.user?.name}`, {
    method: "PATCH",
    headers: {
      authorization: config.PrivateAuth,
    },
    body: JSON.stringify({
      password: generatePassword(password.value as string),
    })
  });

  password.value = undefined;
  $toast.success("Password changed successfully!");
}

async function copyAuth() {
  const { data: key } = await useFetch(`/api/users/${data.value?.user?.name}`, {
    method: "GET",
    headers: {
      authorization: config.PrivateAuth,
    }
  });

  if (newKey.value === undefined) newKey.value = (key.value as any).auth;

  navigator.clipboard.writeText(newKey.value as string);
  $toast.info("Authentication key copied!");
}

useHead({
  title: "Settings | VoidChan"
})
</script>
