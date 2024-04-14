<template>
  <div>
    <h1 class="text-5xl m-5 noselect">Settings</h1>
    <br class="noselect">
    <center>
      <div class="w-full max-w-xs">
        <form class="shadow-md rounded px-8 pt-6 pb-8 mb-4">
          <div class="mb-4">
            <label
              class="block text-lg font-bold mb-3 noselect"
              for="password"
            >
              New Password
            </label>
            <input
              class="shadow bg-violet-900 bg-opacity-10 border-violet-900 border-opacity-60 appearance-none border rounded w-full py-2 px-3 leading-tight focus:outline-none focus:shadow-outline noselect"
              v-model="password"
              type="password"
              placeholder="Password"
            />
          </div>
          <button class="btn mx-2 noselect" @click.prevent="updateSettings">Update</button>
          </form>
          </div>
          <div class="w-full max-w-sm shadow-md rounded px-8 pt-6 pb-8 mb-8 noselect">
            <label class="block text-lg font-bold mb-3 noselect">Authentiation Key</label>
            <button @click="copyAuth" class="btn mx-2"><span class="far fa-clipboard"></span> {{ copiedAuth ? "Key Copied!" : "Copy Key" }}</button>
            <button @click="resetKey" style="border-color: #ad0c00;" class="btn mx-2"><span class="fas fa-exclamation-triangle"></span> {{ resetAuth ? "Key Reset!" : "Reset Key" }}</button>
          </div>
          <button class="btn mx-2 noselect" @click.prevent="$router.go(-1)">Back</button>
    </center>
  </div>
</template>

<script setup lang="ts">
const config = useAppConfig();
const password = ref(undefined);
const copiedAuth = ref(false);
const resetAuth = ref(false);
const newKey = ref<string | undefined>(undefined);
const { data } = useSession();

function resetKey() {
  newKey.value = generateString(32);

  resetAuth.value = true;
  useLazyFetch(`/api/users/${data.value?.user?.name}`, {
    method: "PATCH",
    headers: {
      authorization: config.PrivateAuth,
    },
    body: JSON.stringify({
      authKey: newKey.value,
    }),
  });

  setTimeout(() => {
    resetAuth.value = false;
  }, 1500);
}

function updateSettings() {
  if (password.value === undefined) {
    return;
  }

  useLazyFetch(`/api/users/${data.value?.user?.name}`, {
    method: "PATCH",
    headers: {
      authorization: config.PrivateAuth,
    },
    body: JSON.stringify({
      password: password.value
    })
  });

  password.value = undefined;
}

function copyAuth() {
  const { data: key } = useLazyFetch(`/api/users/${data.value?.user?.name}`, {
    method: "GET",
    headers: {
      authorization: config.PrivateAuth,
    }
  });

  navigator.clipboard.writeText(newKey.value !== undefined ? newKey.value : (key.value as any)?.auth).then(() => {
    copiedAuth.value = true;

    setTimeout(() => {
      copiedAuth.value = false;
    }, 1500)
  })
}
</script>
