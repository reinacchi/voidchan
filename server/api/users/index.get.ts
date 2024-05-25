import getConnection, { IUser } from "~~/server/database";
import { parseJSON } from "~~/utils/parseJSON";

const config = useRuntimeConfig();

export default defineEventHandler(async (event) => {
  const conn = await getConnection();
  const users = (await conn.query("SELECT * FROM users")) as IUser[];

  if (event.node.req.headers.authorization === config.PrivateAuth) {
    const promises = users.map(async (user) => {
      const userFiles = await conn.query(
        "SELECT * FROM files WHERE uploader = ?",
        [user.username]
      );

      const userPosts = await conn.query(
        "SELECT * FROM posts WHERE uploader = ?",
        [user.username]
      );

      return {
        id: user.id,
        clearance_levels: parseJSON(user.clearance_levels),
        created_at: user.created_at,
        auth: user.auth_key,
        email: user.email,
        display_name: user.display_name,
        username: user.username,
        kudos: user.kudos,
        files: userFiles.length as number,
        posts: userPosts.length as number,
      };
    });

    const data = await Promise.all(promises);

    return data;

  } else {
    const promises = users.map(async (user) => {
      const userFiles = await conn.query(
        "SELECT * FROM files WHERE uploader = ?",
        [user.username]
      );

      const userPosts = await conn.query(
        "SELECT * FROM posts WHERE uploader = ?",
        [user.username]
      );

      return {
        id: user.id,
        clearance_levels: parseJSON(user.clearance_levels),
        created_at: user.created_at,
        display_name: user.display_name,
        username: user.username,
        kudos: user.kudos,
        files: userFiles.length as number,
        posts: userPosts.length as number,
      };
    });

    const data = await Promise.all(promises);

    return data;

  }
});
