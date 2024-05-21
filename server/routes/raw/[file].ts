import mime from "mime";
import getConnection from "~~/server/database";

export default defineEventHandler(async (event) => {
  const fileID = getRouterParam(event, "file") as string;
  const conn = await getConnection();
  const file = await conn.query("SELECT * FROM files WHERE id = ?", [fileID.split(".")[0]])

  if (!file[0]) {
    return {
      code: 404,
      message: "Unknown File"
    }
  }

  const mimeType = mime.getExtension(file[0].mimetype);
  const fileName = `${file[0].id}.${mimeType}`;

  if (fileID !== fileName) {
    return {
      code: 404,
      message: "File Not Found"
    }
  }

  const data = Buffer.from(file[0].buffer.split(",")[1], "base64");

  event.node.res.setHeader("Content-Type", file[0].mimetype);
  return send(event, data);
});
