import getConnection from "~~/server/database";
import { getServerSession } from "#auth";

export default defineEventHandler(async (event) => {
  const session = await getServerSession(event) as any;
  const fileID = getRouterParam(event, "file") as string;
  const conn = await getConnection();

  if (!session) {
    return {
      code: 403,
      message: "Not Allowed",
    }
  }

  const user = await conn.query("SELECT * FROM users WHERE username = ?", [session.user.name]);
  const file = await conn.query("SELECT * FROM files WHERE id = ?", [fileID]);

  if (user[0].username === file[0].uploader) {
    const body = await readBody(event);

    await conn.execute("UPDATE files SET nsfw = ? WHERE id = ?", [body.nsfw, fileID]);
  }
});
