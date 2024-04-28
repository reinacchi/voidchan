import { IPosts, Posts } from "~~/server/database/models/posts.model";
import { IProfile, Profile } from "~~/server/database/models/profile.model";
import mime from "mime";

export default defineEventHandler(async (event) => {
  const user = getRouterParam(event, "user") as string;
  const profile = await Profile.findOne({ name: user }) as IProfile;
  const posts = await Posts.find({ id: profile.posts }) as IPosts[];

  return posts.map((post) => {
    return {
      id: post.id,
      date: post.date,
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
