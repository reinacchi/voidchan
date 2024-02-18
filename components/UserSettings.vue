<template>
  <div>
    <NuxtSnackBar />
    <h1 class="text-5xl m-5 noselect">Settings</h1>
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
          </form>
          </div>
          <button class="btn mx-2 noselect" @click.prevent="$router.go(-1)">Back</button>
          <button class="btn mx-2 noselect" @click.prevent="updateSettings">Update</button>
    </center>
  </div>
</template>

<script setup lang="ts">
const config = useRuntimeConfig();
const password = ref(undefined);
const newName = ref(undefined);
const newPassword = ref(undefined);
const { data } = useSession();
const { data: userProfile } = useFetch(`/api/users/${data.value?.user?.name}`, {
  method: "GET",
  headers: {
    authorization: "internalsecretkeylololol"
  }
});

function updateSettings() {
  $fetch(`/api/users/${data.value?.user?.name}`, {
    method: "PATCH",
    body: JSON.stringify({
      password: password.value
    })
  });


  password.value = undefined;
}

function createUser() {
  if (!newName.value && !newPassword.value) {
    return;
  }

  $fetch(`/api/users/create`, {
    method: "POST",
    body: JSON.stringify({
      name: newName.value,
      password: newPassword.value
    })
  });

  newName.value = undefined;
  newPassword.value = undefined;
}
</script>
