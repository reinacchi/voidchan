import getConnection, { IUser } from "~~/server/database";

const config = useRuntimeConfig();

export default defineEventHandler(async (event) => {
  const conn = await getConnection();
  const users = (await conn.query("SELECT * FROM users")) as IUser[];

  if (event.node.req.headers.authorization === config.PrivateAuth) {
    const promises = await users.map(async (user) => {
      return conn.query(
        "SELECT * FROM files WHERE uploader = ?",
        [user.username]
      ).then((userFiles) => {
        return conn.query(
          "SELECT * FROM posts WHERE uploader = ?",
          [user.username]
        ).then((userPosts) => {
          return {
            id: user.id,
            clearanceLevels: JSON.parse(user.clearanceLevels) as string[],
            createdAt: user.createdAt,
            auth: user.authKey,
            email: user.email,
            displayName: user.displayName,
            username: user.username,
            kudos: user.kudos,
            files: userFiles.length as number,
            posts: userPosts.length as number,
          };
        });
      });
    });
    const data = await Promise.all(promises).then((res) => res);

    return data;
  } else {
    const promises = await users.map(async (user) => {
      return conn.query(
        "SELECT * FROM files WHERE uploader = ?",
        [user.username]
      ).then((userFiles) => {
        return conn.query(
          "SELECT * FROM posts WHERE uploader = ?",
          [user.username]
        ).then((userPosts) => {
          return {
            id: user.id,
            clearanceLevels: JSON.parse(user.clearanceLevels) as string[],
            createdAt: user.createdAt,
            displayName: user.displayName,
            username: user.username,
            kudos: user.kudos,
            files: userFiles.length as number,
            posts: userPosts.length as number,
          };
        });
      });
    });
    const data = await Promise.all(promises).then((res) => res);

    return data;
  }
});
