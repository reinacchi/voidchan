import { createCaptcha } from "kyapu";
import { v4 as uuidv4 } from "uuid";
import { captchaStorage } from "~~/server/utils/storage";
import { Captcha } from "~~/types/vue-shim";

export default defineEventHandler(async () => {
  const captcha = createCaptcha({
    background: "#121030",
    fontSize: Math.floor(50 * 0.9),
    noise: 6,
    size: 6,
  });
  const uuid = uuidv4();
  await captchaStorage.setItem(uuid, captcha.text);

  return { uuid, svg: captcha.data } as Captcha;
});
