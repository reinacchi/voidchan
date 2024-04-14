// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  appConfig: {
    ClientID: process.env.CLIENT_ID,
    ClientSecret: process.env.CLIENT_SECRET,
    ClientToken: process.env.CLIENT_TOKEN,
    PrivateAuth: process.env.PRIVATE_AUTH,
    MongoDB: process.env.MONGODB_URI,
  },
  modules: [
    "@nuxtjs/tailwindcss",
    "@huntersofbook/naive-ui-nuxt",
    "@nuxtjs/color-mode",
    "@sidebase/nuxt-auth",
  ],
  typescript: {
    shim: false,
  },
  auth: {
    origin: process.env.BASE_URL,
    enableGlobalAppMiddleware: true,
  },
  css: [
    "@/assets/css/style.css",
  ],
  nitro: {
    plugins: ["~/server/database/index.ts", "~/server/discord/index.ts"],
  },
  runtimeConfig: {
    BaseURL: process.env.BASE_URL,
    ClientID: process.env.CLIENT_ID,
    ClientSecret: process.env.CLIENT_SECRET,
    ClientToken: process.env.CLIENT_TOKEN,
    MongoDB: process.env.MONGODB_URI,
    Port: process.env.PORT,
    Prefix: process.env.PREFIX,
    PrivateAuth: process.env.PRIVATE_AUTH,
  }
});
