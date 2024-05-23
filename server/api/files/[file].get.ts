import getConnection from "~~/server/database";
import mime from "mime";
import { getServerSession } from "#auth";
import { parseJSON } from "~~/utils/parseJSON";

export default defineEventHandler(async (event) => {
  const fileID = getRouterParam(event, "file") as string;
  const session = await getServerSession(event) as any;
  const conn = await getConnection();
  const file = await conn.query("SELECT * FROM files WHERE id = ?", [fileID]);
  const uploader = await conn.query("SELECT * FROM users WHERE username = ?", [file[0].uploader]);

  if (!file[0]) {
    return {
      code: 404,
      message: "Unknown File"
    }
  }

  if (!session) {
    return {
      id: file[0].id,
      date: file[0].date,
      ext: mime.getExtension(file[0].mimetype),
      nsfw: file[0].nsfw ? true : false,
      uploader: {
        name: uploader[0].username,
      },
      url: file[0].buffer,
    };
  } else {
    const user = await conn.query("SELECT * FROM users WHERE username = ?", [session.user.name]);

    return {
      id: file[0].id,
      date: file[0].date,
      ext: mime.getExtension(file[0].mimetype),
      nsfw: file[0].nsfw ? true : false,
      uploader: {
        clearanceLevels: parseJSON(user[0].clearanceLevels),
        name: uploader[0].username,
      },
      url: file[0].buffer,
    };
  }

});
