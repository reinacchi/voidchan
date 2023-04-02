import { IProfile, Profile } from "~~/server/database/models/profile.model";

export default defineEventHandler(async (event) => {
  const userID = getRouterParam(event, "user") as string;
});
