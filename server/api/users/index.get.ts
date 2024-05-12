import { IProfile, Profile } from "~~/server/database/models/profile.model";

const config = useRuntimeConfig();

export default defineEventHandler(async (event) => {
  const users = (await Profile.find({})) as IProfile[];

  if (event.node.req.headers.authorization === config.PrivateAuth) {
    return users.map((user) => {
      return {
        clearanceLevel: user.clearanceLevel,
        auth: user.authKey,
        createdAt: user.createdAt,
        displayName: user.displayName,
        email: user.email,
        name: user.name,
        posts: user.posts,
      };
    });
  } else {
    return users.map((user) => {
      return {
        clearanceLevel: user.clearanceLevel,
        createdAt: user.createdAt,
        displayName: user.displayName,
        name: user.name,
        posts: user.posts
      };
    });
  }
});
