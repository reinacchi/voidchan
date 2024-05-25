import { getServerSession } from "#auth";
import getConnection from "~~/server/database";
import { generateString } from "~~/utils/generateString";

export default defineEventHandler(async (event) => {
  const session = await getServerSession(event);
  const conn = await getConnection();

  if (!session) {
    return {
      code: 403,
      message: "Not Allowed",
    };
  }

  const body = await readBody(event);

  await conn.execute(
    "INSERT INTO posts (buffer, comments, created_at, favourites, mimetype, rating, size, status, tags, uploader) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    [
      body.fileBuffer,
      [],
      new Date,
      0,
      body.fileType,
      body.fileRating,
      body.fileSize,
      "pending",
      body.fileTags ?? [],
      session.user?.name,
    ]
  );
});
