import { getServerSession } from "#auth";
import { Posts, IPosts } from "~~/server/database/models/posts.model";
import { generateString } from "~~/utils/generateString";

export default defineEventHandler(async (event) => {
  const session = await getServerSession(event);

  if (!session) {
    return {
      code: 403,
      message: "Not Allowed",
    };
  }

  const post = await Posts.find({});
  const body = await readBody(event);
  const filename = generateString(36);

  await Posts.create({
    buffer: body.fileBuffer,
    comments: [],
    date: new Date(),
    favourites: 0,
    filename,
    id: post.length ? post.length++ : 0,
    mimetype: body.fileType,
    rating: body.fileRating,
    size: body.fileSize,
    source: {
      characters: body.fileCharacters ?? [],
      copyright: body.fileCopyright ?? "",
      source: body.fileSource ?? "",
    },
    status: "pending",
    tags: body.fileTags ?? [],
    uploader: session.user?.name,
  } as IPosts);
});
