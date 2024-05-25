import getConnection from "~~/server/database";
import mime from "mime";

export default defineEventHandler(async (event) => {
  const userParam = getRouterParam(event, "user") as string;
  const conn = await getConnection();
  const posts = await conn.query("SELECT * FROM posts WHERE uploader = ?", [userParam]);

  return posts.map((post: any) => {
    return {
      id: post.id,
      created_at: post.created_at,
      ext: mime.getExtension(post.mimetype),
      uploader: post.uploader,
      tags: post.tags,
      rating: post.rating,
      size: post.size,
      favourites: post.favourites,
      buffer: post.buffer,
      status: post.status,
    }
  });
});
