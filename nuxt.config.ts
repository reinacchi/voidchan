// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  appConfig: {
    BaseURL: process.env.BASE_URL,
    DBHost: process.env.DB_HOST,
    DBPort: process.env.DB_PORT,
    DBName: process.env.DB_NAME,
    DBPassword: process.env.DB_PASSWORD,
    DBUser: process.env.DB_USER,
    ClientID: process.env.CLIENT_ID,
    ClientSecret: process.env.CLIENT_SECRET,
    ClientToken: process.env.CLIENT_TOKEN,
    PrivateAuth: process.env.PRIVATE_AUTH,
  },
  build: {
    transpile: ["vue-sonner"],
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
    baseURL: process.env.BASE_URL,
    globalAppMiddleware: true,
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
    DBHost: process.env.DB_HOST,
    DBPort: process.env.DB_PORT,
    DBName: process.env.DB_NAME,
    DBPassword: process.env.DB_PASSWORD,
    DBUser: process.env.DB_USER,
    Port: process.env.PORT,
    Prefix: process.env.PREFIX,
    PrivateAuth: process.env.PRIVATE_AUTH,
  }
});
