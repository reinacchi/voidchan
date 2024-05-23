import CredentialsProvider from "next-auth/providers/credentials";
/* @ts-ignore */
import { NuxtAuthHandler } from "#auth";
import getConnection from "~~/server/database";
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
        const conn = await getConnection();
        const userToCheck = await conn.query("SELECT * FROM users WHERE username = ?", [credentials.username]);

        if (!userToCheck[0]) return null;

        const pass = bcrypt.compareSync(
          credentials.password,
          userToCheck[0].password
        );

        if (!pass) return null;

        if (userToCheck[0]) {
          return {
            name: userToCheck[0].username,
            email: userToCheck[0].email,
          };
        } else {
          return null;
        }
      },
    }),
  ],
});
