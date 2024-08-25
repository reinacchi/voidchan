import type { Submission } from "~~/types/vue-shim";

export default defineEventHandler(async (event) => {
  const body = await readBody<Submission>(event);
  const entry = await captchaStorage.getItem(body.uuid);

  if (!entry || entry !== body.captcha) {
    return {
      success: 0,
      message: "Invalid captcha!"
    }
  } else {
    return {
      success: 1,
      message: "Captcha is valid!"
    }
  }
});
