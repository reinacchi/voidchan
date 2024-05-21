import getConnection from "~~/server/database";
import mime from "mime";

export default defineEventHandler(async (event) => {
  const userParam = getRouterParam(event, "user") as string;
  const conn = await getConnection();
  const user = await conn.query("SELECT * FROM users WHERE username = ?", [userParam]);
  const files = await conn.query("SELECT * FROM files WHERE uploader = ?", [userParam]);

  return files.map((file: any) => {
    return {
      id: file.id,
      date: file.date,
      ext: mime.getExtension(file.mimetype),
      nsfw: file.nsfw,
      uploader: {
        name: file.uploader,
      },
      url: file.buffer,
    }
  });
});
