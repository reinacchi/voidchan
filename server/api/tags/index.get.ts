import getConnection from "~~/server/database";
import { countTagOccurrences } from "~~/utils/countTagOccurences";

export default defineEventHandler(async (event) => {
  const query = getQuery(event);
  const conn = await getConnection();

  const results = await conn.query("SELECT * FROM tags");

  const promises = results.map(async (result: any) => {
    const allTags = await conn.query("SELECT tags FROM posts WHERE tags IS NOT NULL");
    const count = countTagOccurrences(allTags, result.name);

    return { name: result.name, count };
  });

  const data = await Promise.all(promises);

  return data;
});
