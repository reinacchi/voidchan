import getConnection from "~~/server/database";
import { parseJSON } from "~~/utils/parseJSON";

const config = useRuntimeConfig();

export default defineEventHandler(async (event) => {
  const userParam = getRouterParam(event, "user") as string;
  const conn = await getConnection();
  const user = await conn.query(`SELECT * FROM users WHERE username = ?`, [userParam]);
  const userFiles = await conn.query("SELECT * FROM files WHERE uploader = ?", [userParam]);
  const userPosts = await conn.query("SELECT * FROM posts WHERE uploader = ?", [userParam]);

  if (event.node.req.headers.authorization === config.PrivateAuth) {
    return {
      clearance_levels: parseJSON(user[0].clearance_levels),
      auth: user[0].auth_key,
      created_at: user[0].created_at,
      display_name: user[0].display_name,
      email: user[0].email,
      files: userFiles.length as number,
      posts: userPosts.length as number,
      username: user[0].username,
      kudos: user[0].kudos,
    }
  } else {
    return {
      clearance_levels: parseJSON(user[0].clearance_levels),
      created_at: user[0].created_at,
      display_name: user[0].display_name,
      files: userFiles.length as number,
      posts: userPosts.length as number,
      username: user[0].username,
      kudos: user[0].kudos,
    }
  }
});
