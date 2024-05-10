import { Files } from "../../database/models/files.model";
import { IProfile, Profile } from "~~/server/database/models/profile.model";
import { getServerSession } from "#auth";

export default defineEventHandler(async (event) => {
  const session = await getServerSession(event) as any;
  const fileID = getRouterParam(event, "file") as string;

  if (!session) {
    return {
      code: 403,
      message: "Not Allowed",
    }
  }

  const profile = await Profile.findOne({ name: session?.user?.name }) as IProfile;
  const file = await Files.findOne({ id: fileID });

  if (profile.name === file?.uploader) {
    await Profile.findOneAndUpdate({ name: file.uploader }, { $pull: { files: fileID } });
    await Files.findOneAndDelete({ id: fileID });
  }
});
