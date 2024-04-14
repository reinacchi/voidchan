import { Files, IFiles } from "~~/server/database/models/files.model";
import { IProfile, Profile } from "~~/server/database/models/profile.model";
import mime from "mime";
import { getServerSession } from "#auth";

export default defineEventHandler(async (event) => {
  const config = useRuntimeConfig();
  const fileID = getRouterParam(event, "file") as string;
  const file = await Files.findOne({ id: fileID }) as IFiles;
  const session = await getServerSession(event) as any;
  const uploader = await Profile.findOne({ name: file.uploader }) as IProfile;

  if (!file) {
    return {
      code: 404,
      message: "Unknown File"
    }
  }

  if (!session) {
    return {
      id: file.id,
      date: file.date,
      ext: mime.getExtension(file.mimetype),
      nsfw: file.nsfw,
      uploader: {
        name: uploader.name,
      },
      url: `${config.BaseURL}/raw/${file.id}.${mime.getExtension(file.mimetype)}`,
    };
  } else {
    const profile = await Profile.findOne({ name: session.user.name }) as IProfile;

    return {
      id: file.id,
      date: file.date,
      ext: mime.getExtension(file.mimetype),
      nsfw: file.nsfw,
      uploader: {
        admin: profile.admin,
        name: uploader.name,
      },
      url: `${config.BaseURL}/raw/${file.id}.${mime.getExtension(file.mimetype)}`,
    };
  }

});
