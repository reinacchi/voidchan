import { Files, IFiles } from "~~/server/database/models/files.model";
import { IProfile, Profile } from "~~/server/database/models/profile.model";
import mime from "mime";

export default defineEventHandler(async (event) => {
  const user = getRouterParam(event, "user") as string;
  const config = useRuntimeConfig();
  const profile = await Profile.findOne({ name: user }) as IProfile;
  const files = await Files.find({ id: profile.files }) as IFiles[];

  return files.map((file) => {
    return {
      id: file.id,
      date: file.date,
      ext: mime.getExtension(file.mimetype),
      nsfw: file.nsfw,
      uploader: {
        name: profile.name,
      },
      url: `data:${file.mimetype};base64,${file.buffer}`,
      // url: `${config.BaseURL}/raw/${file.id}.${mime.getExtension(file.mimetype)}`,
    }
  }).reverse();
});
