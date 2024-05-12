import { IProfile, Profile } from "~~/server/database/models/profile.model";

const config = useRuntimeConfig();

export default defineEventHandler(async (event) => {
  const user = getRouterParam(event, "user") as string;
  const profile = await Profile.findOne({ name: user }) as IProfile;

  if (event.node.req.headers.authorization === config.PrivateAuth) {
    return {
      clearanceLevel: profile.clearanceLevel,
      auth: profile.authKey,
      createdAt: profile.createdAt,
      displayName: profile.displayName,
      email: profile.email,
      name: profile.name,
      posts: profile.posts,
      kudos: profile.kudos,
    }
  } else {
    return {
      clearanceLevel: profile.clearanceLevel,
      createdAt: profile.createdAt,
      displayName: profile.displayName,
      name: profile.name,
      posts: profile.posts,
      kudos: profile.kudos,
    }
  }
});
