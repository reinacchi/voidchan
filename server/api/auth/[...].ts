import CredentialsProvider from "next-auth/providers/credentials";
/* @ts-ignore */
import { NuxtAuthHandler } from "#auth";
import { IProfile, Profile } from "~~/server/database/models/profile.model";
import bcrypt from "bcryptjs";

const config = useRuntimeConfig();

export default NuxtAuthHandler({
  secret: config.PrivateAuth,
  pages: {
    signIn: "/login",
  },
  providers: [
    /* @ts-ignore */
    CredentialsProvider.default({
      name: "Credentials",
      async authorize(credentials: any, req: any) {
        const userToCheck = (await Profile.findOne({
          name: credentials.username,
        })) as IProfile;

        if (!userToCheck) return null;

        const pass = bcrypt.compareSync(
          credentials.password,
          userToCheck.password
        );

        if (!pass) return null;

        if (userToCheck) {
          return userToCheck;
        } else {
          return null;
        }
      },
    }),
  ],
});
