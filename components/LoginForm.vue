<template>
  <form
    class="space-y-4 md:space-y-6 noselect"
    @submit.prevent="handleSignIn({ username: username, password: password })"
  >
    <div>
      <label for="username" class="block mb-2 text-sm font-medium text-left"
        >Username</label
      >
      <input
        v-model="username"
        autocomplete="off"
        type="username"
        name="username"
        id="username"
        class="bg-gray-50 border border-gray-300 text-gray-600 sm:text-sm rounded-lg focus:outline-none block w-full p-2.5"
        placeholder="Username"
      />
    </div>
    <div>
      <label for="password" class="block mb-2 text-sm font-medium text-left"
        >Password</label
      >
      <input
        v-model="password"
        type="password"
        name="password"
        id="password"
        placeholder="Password"
        class="bg-gray-50 border border-gray-300 text-gray-600 sm:text-sm focus:outline-none rounded-lg block w-full p-2.5"
      />
    </div>
    <div>
      <label class="block mb-2 text-sm font-medium text-left" for="captcha"
        >Captcha</label
      >
      <div class="flex items-center justify-start">
        <div class="w-40 flex items-center rounded">
          <span v-if="captcha" v-html="captcha.svg" />
        </div>
        <input
          id="captcha"
          maxlength="6"
          placeholder="Captcha"
          type="text"
          class="bg-gray-50 border border-gray-300 text-gray-600 sm:text-sm focus:outline-none rounded-lg flex items-center w-2/5 p-2.5"
          v-model="submission.captcha"
        />
      </div>
    </div>
    <div class="flex items-center justify-between">
      <nuxt-link
        to="/reset-password"
        class="text-sm font-medium text-primary-600 hover:underline"
        >Forgot password?</nuxt-link
      >
    </div>
    <button
      type="submit"
      class="btn glass text-white w-full relative flex items-center justify-center"
    >
      <span v-if="!loading">Login</span>
      <div v-if="loading" class="loader"></div>
    </button>
    <p class="text-sm font-light text-primary-600">
      New to VoidChan?
      <nuxt-link
        to="/register"
        class="font-bold text-violet-500 hover:underline"
        >Register</nuxt-link
      >
    </p>
  </form>
</template>

<script setup lang="ts">
import { toast } from "vue-sonner";
import type { Captcha, Submission } from "@/types/vue-shim";

const { signIn } = useAuth();
const username = ref("");
const password = ref("");
const loading = ref(false);

function getQueryParam(paramName: string) {
  const currentUrl = window.location.href;
  const urlObj = new URL(currentUrl);
  const params = new URLSearchParams(urlObj.search);
  return params.get(paramName);
}

function extractPathFromURL(fullURL: string) {
  try {
    const urlObj = new URL(fullURL);
    return urlObj.pathname;
  } catch (error) {
    return "/";
  }
}

const captcha = ref<Captcha | undefined>(undefined);
const submission = ref<Submission>({
  captcha: "",
  uuid: "",
});

const getCaptcha = async () => {
  captcha.value = (await useFetch("/api/captcha/generate")).data
    .value as Captcha;
  submission.value.uuid = captcha.value.uuid;
};

const handleSignIn = async ({
  username,
  password,
}: {
  username: string;
  password: string;
}) => {
  loading.value = true;

  const currentURL = getQueryParam("callbackUrl") as string;
  const path = extractPathFromURL(currentURL);

  if (!username) {
    loading.value = false;
    submission.value.captcha = "";
    getCaptcha();
    return toast.error("Invalid username!");
  }
  const { data } = await useFetch("/api/captcha/submit", {
    method: "POST",
    body: submission.value,
  });

  if (!password) {
    loading.value = false;
    submission.value.captcha = "";
    getCaptcha();
    return toast.error("Invalid password!");
  }

  if (data.value?.success === 0) {
    loading.value = false;
    toast.error(data.value.message);
    submission.value.captcha = "";
    return getCaptcha();
  }

  /* @ts-ignore */
  const { error, url } = await signIn("credentials", {
    username,
    password,
    callbackUrl: path,
    redirect: false,
  });


  if (!username) {
    loading.value = false;
    submission.value.captcha = "";
    getCaptcha();
    return toast.error("Invalid username!");
  }
  if (!password) {
    loading.value = false;
    submission.value.captcha = "";
    getCaptcha();
    return toast.error("Invalid password!");
  }

  if (data.value?.success === 0) {
    loading.value = false;
    toast.error(data.value.message);
    submission.value.captcha = "";
    return getCaptcha();
  } else {
    if (error) {
      loading.value = false;
      submission.value.captcha = "";
      getCaptcha();
      return toast.error("Username or password is wrong!");
    } else {
      return navigateTo(url, { external: true });
    }
  }
};

onMounted(getCaptcha);
</script>

<style scoped>
.btn {
  display: inline-flex;
  flex-shrink: 0;
  cursor: pointer;
  -webkit-user-select: none;
  -moz-user-select: none;
  user-select: none;
  flex-wrap: wrap;
  align-items: center;
  justify-content: center;
  border-color: transparent;
  border-color: hsl(var(--b2) / var(--tw-border-opacity));
  text-align: center;
  transition-property: color, background-color, border-color,
    text-decoration-color, fill, stroke, opacity, box-shadow, transform, filter,
    -webkit-backdrop-filter;
  transition-property: color, background-color, border-color,
    text-decoration-color, fill, stroke, opacity, box-shadow, transform, filter,
    backdrop-filter;
  transition-property: color, background-color, border-color,
    text-decoration-color, fill, stroke, opacity, box-shadow, transform, filter,
    backdrop-filter, -webkit-backdrop-filter;
  transition-timing-function: cubic-bezier(0.4, 0, 0.2, 1);
  transition-timing-function: cubic-bezier(0, 0, 0.2, 1);
  transition-duration: 0.2s;
  border-radius: var(--rounded-btn, 0.5rem);
  height: 3rem;
  padding-left: 1rem;
  padding-right: 1rem;
  min-height: 3rem;
  font-size: 0.875rem;
  line-height: 1em;
  gap: 0.5rem;
  font-weight: 600;
  text-decoration-line: none;
  border-width: var(--border-btn, 1px);
  animation: button-pop var(--animation-btn, 0.25s) ease-out;
  text-transform: var(--btn-text-case, uppercase);
  --tw-border-opacity: 1;
  --tw-bg-opacity: 1;
  background-color: hsl(var(--b2) / var(--tw-bg-opacity));
  --tw-text-opacity: 1;
  color: hsl(var(--bc) / var(--tw-text-opacity));
  outline-color: hsl(var(--bc) / 1);
}

.btn.glass {
  --tw-shadow: 0 0 #0000;
  --tw-shadow-colored: 0 0 #0000;
  box-shadow: var(--tw-ring-offset-shadow, 0 0 #0000),
    var(--tw-ring-shadow, 0 0 #0000), var(--tw-shadow);
  outline-color: currentColor;
}

.glass {
  border: none;
  -webkit-backdrop-filter: blur(var(--glass-blur, 40px));
  backdrop-filter: blur(var(--glass-blur, 40px));
  background-color: transparent;
  background-image: linear-gradient(
      135deg,
      rgb(255 255 255 / var(--glass-opacity, 30%)) 0%,
      rgb(0 0 0 / 0%) 100%
    ),
    linear-gradient(
      var(--glass-reflex-degree, 100deg),
      rgb(255 255 255 / var(--glass-reflex-opacity, 10%)) 25%,
      rgb(0 0 0 / 0%) 25%
    );
  box-shadow: 0 0 0 1px rgb(255 255 255 / var(--glass-border-opacity, 10%))
      inset,
    0 0 0 2px #0000000d;
  text-shadow: 0 1px rgb(0 0 0 / var(--glass-text-shadow-opacity, 5%));
}

.btn:hover,
.glass:hover {
  background-color: rgba(20, 20, 20, 0.9); /* Even darker grayish color */
  border: none;
  color: hsl(
    var(--bc) / var(--tw-text-opacity-hover)
  ); /* Keep the text color same or adjust if needed */
  text-shadow: 0 1px rgba(0, 0, 0, 0.1); /* Adjust text shadow if necessary */
}

.btn:active {
  transform: translateY(5px) scale(0.98);
}

.loader {
  border: 4px solid rgba(255, 255, 255, 0.3);
  border-top: 4px solid white;
  border-radius: 50%;
  width: 24px;
  height: 24px;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  0% {
    transform: rotate(0deg);
  }
  100% {
    transform: rotate(360deg);
  }
}
</style>
