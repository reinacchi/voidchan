import CredentialsProvider from "next-auth/providers/credentials";
/* @ts-ignore */
import { NuxtAuthHandler } from "#auth";
import { IProfile, Profile } from "~~/server/database/models/profile.model";
import bcrypt from "bcryptjs";

export default NuxtAuthHandler({
  secret: "testinglololol",
  providers: [
    /* @ts-ignore */
    CredentialsProvider.default({
      name: "Credentials",
      credentials: {
        username: {
          label: "Username",
          type: "text",
          placeholder: "Your Username",
        },
        password: {
          label: "Password",
          type: "password",
          placeholder: "Your Password",
        },
      },
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
