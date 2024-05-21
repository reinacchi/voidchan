import getConnection from "~~/server/database";

const config = useRuntimeConfig();

export default defineEventHandler(async (event) => {
  const userParam = getRouterParam(event, "user") as string;
  const conn = await getConnection();
  const user = await conn.query(`SELECT * FROM users WHERE username = ?`, [userParam]);
  const userFiles = await conn.query("SELECT * FROM files WHERE uploader = ?", [userParam]);
  const userPosts = await conn.query("SELECT * FROM posts WHERE uploader = ?", [userParam]);

  if (event.node.req.headers.authorization === config.PrivateAuth) {
    return {
      clearanceLevel: JSON.parse(user[0].clearanceLevels),
      auth: user[0].authKey,
      createdAt: user[0].createdAt,
      displayName: user[0].displayName,
      email: user[0].email,
      files: userFiles.length as number,
      posts: userPosts.length as number,
      username: user[0].username,
      kudos: user[0].kudos,
    }
  } else {
    return {
      clearanceLevel: JSON.parse(user[0].clearanceLevels),
      createdAt: user[0].createdAt,
      displayName: user[0].displayName,
      files: userFiles.length as number,
      posts: userPosts.length as number,
      username: user[0].username,
      kudos: user[0].kudos,
    }
  }
});
