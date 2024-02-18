import CredentialsProvider from "next-auth/providers/credentials";
/* @ts-ignore */
import { NuxtAuthHandler } from "#auth";
import { Profile } from "~~/server/database/models/profile.model";

export default NuxtAuthHandler({
  secret: "testinglololol",
  providers: [
    /* @ts-ignore */
    CredentialsProvider.default({
      name: "Credentials",
      credentials: {
        username: { label: "Username", type: "text", placeholder: "Your Username" },
        password: { label: "Password", type: "password", placeholder: "Your Password" }
      },
      async authorize(credentials: any, req: any) {
          const user = await Profile.findOne({ name: credentials.username, password: credentials.password });

          if (user) {
            return user;
          } else {
            return null;
          }
      },
    })
  ]
});
