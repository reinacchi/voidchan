import { Schema, model } from "mongoose";

type PostRating = "explicit" | "erotica" | "suggestive" | "safe";
type PostStatus = "posted" | "pending" | "disapproved";

interface PostSource {
  characters: string[];
  copyright: string;
  source: string;
}

interface PostComments {
  comment: string;
  date: Date;
  name: string;
}

export interface IPosts {
  id: number;
  filename: string;
  date: Date;
  buffer: string;
  mimetype: string;
  uploader: string;
  status: PostStatus;
  tags: string[];
  comments: PostComments[];
  favourites: number;
  source: PostSource;
  size: number;
  rating: PostRating;
}

const postsSchema = new Schema<IPosts>({
  id: {
    type: Schema.Types.Number,
    required: true,
  },
  filename: {
    type: Schema.Types.String,
    required: true,
  },
  date: {
    type: Schema.Types.Date,
    default: new Date(),
    required: true,
  },
  buffer: {
    type: Schema.Types.String,
    required: true,
  },
  mimetype: {
    type: Schema.Types.String,
    required: true,
  },
  uploader: {
    type: Schema.Types.String,
    required: true,
  },
  status: {
    type: Schema.Types.String,
    required: true,
  },
  tags: {
    type: [Schema.Types.String],
    required: true,
  },
  comments: {
    type: Schema.Types.Mixed,
    required: true,
  },
  favourites: {
    type: Schema.Types.Number,
    default: 0,
    required: true,
  },
  source: {
    type: Schema.Types.Mixed,
    required: true,
  },
  size: {
    type: Schema.Types.Number,
    required: true,
  },
  rating: {
    type: Schema.Types.String,
    required: true,
  },
});

export const Posts = model<IPosts>("posts", postsSchema);
