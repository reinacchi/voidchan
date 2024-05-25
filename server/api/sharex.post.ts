import { generateString } from "~~/utils/generateString";
import getConnection from "../database";
import useFiles from "~~/utils/useFiles";
import mime from "mime";

const config = useRuntimeConfig();

export default defineEventHandler(async (event) => {
  const auth = getHeader(event, "Authorisation");
  const conn = await getConnection();
  const user = await conn.query(`SELECT * FROM users WHERE auth_key = ?`, [auth]);
  const { files } = await useFiles(event);

  if (!user[0]) {
    event.node.res.statusCode = 401;
    event.node.res.setHeader("Content-Type", "text/plain");
    return (event.node.res.statusMessage =
      "You are not authorised to use this endpoint");
  }

  const name = generateString(5);
  const mimeType = mime.getExtension(files[0].mimetype);

  await conn.execute(
    "INSERT INTO files (id, created_at, buffer, mimetype, nsfw, uploader) VALUES (?, ?, ?, ?, ?, ?)",
    [
      name,
      new Date(),
      `data:${files[0].mimetype};base64,${files[0].buffer.toString("base64")}`,
      files[0].mimetype,
      false,
      user[0].username,
    ]
  );

  event.node.res.setHeader("Content-Type", "application/json");

  return {
    code: 200,
    url: `${config.BaseURL}/v/${name}.${mimeType}`,
  };
});
