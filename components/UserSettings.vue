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
              class="shadow bg-neutral-900 border-neutral-500 appearance-none border rounded w-full py-2 px-3 leading-tight focus:outline-none focus:shadow-outline noselect"
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
            <button @click="copyAuth" class="btn mx-2">{{ copiedAuth ? "Copied to Clipboard!" : "Copy Key" }}</button>
            <button style="border-color: #ad0c00;" class="btn mx-2">Reset Key</button>
          </div>
          <button class="btn mx-2 noselect" @click.prevent="$router.go(-1)">Back</button>
    </center>
  </div>
</template>

<script setup lang="ts">
const config = useAppConfig();
const password = ref(undefined);
const copiedAuth = ref(false);
const { data } = useSession();
const { data: userProfile } = useFetch(`/api/users/${data.value?.user?.name}`, {
  method: "GET",
  headers: {
    authorization: config.PrivateAuth,
  }
});

function updateSettings() {
  if (password.value === undefined) {
    return;
  }

  $fetch(`/api/users/${data.value?.user?.name}`, {
    method: "PATCH",
    body: JSON.stringify({
      password: password.value
    })
  });

  password.value = undefined;
}

function copyAuth() {
  navigator.clipboard.writeText(userProfile.value.auth).then(() => {
    copiedAuth.value = true;

    setTimeout(() => {
      copiedAuth.value = false;
    }, 1500)
  })
}
</script>
