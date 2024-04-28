import { Files } from "../../database/models/files.model";
import { IProfile, Profile } from "~~/server/database/models/profile.model";
import { getServerSession } from "#auth";

export default defineEventHandler(async (event) => {
  const session = await getServerSession(event) as any;
  const file = getRouterParam(event, "file") as string;

  if (!session) {
    return {
      code: 403,
      message: "Not Allowed",
    }
  }

  const profile = await Profile.findOne({ name: session?.user?.name }) as IProfile;
  const uploader = await Profile.find({ files: file });

  if (profile.clearanceLevel.includes("Moderator")) {
    await Profile.findOneAndUpdate({ name: uploader[0].name }, { $pull: { files: file } });
    await Files.findOneAndDelete({ id: file });
  }
});
