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

  if (!profile.admin) {
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
    admin: false,
    authKey,
    date: new Date(),
    email: "",
    files: [],
    name: body.name,
    password: body.password,
  });
});
