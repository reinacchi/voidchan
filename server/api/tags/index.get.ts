import getConnection from "~~/server/database";
import { countTagOccurrences } from "~~/utils/countTagOccurences";

export default defineEventHandler(async (event) => {
  const query = getQuery(event);
  const conn = await getConnection();

  if (!query.name || (query.name as string).length === 0) {
    return [];
  }

  const results = await conn.execute("SELECT name FROM tags WHERE REPLACE(name, ' ', '') LIKE ? LIMIT 10", [`%${query.name}%`]);

  const promises = results.map(async (result: any) => {
    const allTags = await conn.query(`SELECT tags FROM posts WHERE tags IS NOT NULL`);
    const count = countTagOccurrences(allTags, result.name);

    return { name: result.name, count };
  });

  const data = await Promise.all(promises);

  return data;
});
