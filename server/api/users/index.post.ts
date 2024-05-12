import { IProfile, Profile } from "../../database/models/profile.model";
import { getServerSession } from "#auth";
import { generateString } from "~~/utils/generateString";

export default defineEventHandler(async (event) => {
  const session = await getServerSession(event) as any;
  const body = await readBody(event);
  const profile = await Profile.findOne({ name: session.user.name }) as IProfile;
  const newProfile = await Profile.findOne({ name: body.name });

  if (!session) {
    return {
      code: 403,
      message: "Not Allowed",
    }
  }

  if (!profile.clearanceLevel.includes("Moderator")) {
    return {
      code: 403,
      message: "Not Allowed"
    }
  }

  if (newProfile) {
    return {
      code: 400,
      message: "This account has existed"
    }
  }

  const authKey = generateString(32);

  Profile.create({
    clearanceLevel: ["Member"],
    displayName: body.name,
    kudos: 0,
    authKey,
    createdAt: new Date(),
    email: "",
    files: [],
    posts: [],
    name: body.name,
    password: body.password,
  });
});
