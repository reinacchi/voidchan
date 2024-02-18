import { IProfile, Profile } from "~~/server/database/models/profile.model";

const config = useRuntimeConfig();

export default defineEventHandler(async (event) => {
  const user = getRouterParam(event, "user") as string;
  const profile = await Profile.findOne({ name: user }) as IProfile;

  if (event.node.req.headers.authorization === config.PrivateAuth) {
    return {
      admin: profile.admin,
      createdAt: profile.date,
      email: profile.email,
      name: profile.name,
    }
  } else {
    return {
      createdAt: profile.date,
      name: profile.name,
    }
  }
});
