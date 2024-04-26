import { Schema, model } from "mongoose";

export interface IFiles {
  id: string;
  date: Date;
  favourites: number;
  buffer: string;
  mimetype: string;
  nsfw: boolean;
  uploader: string;
}

const filesSchema = new Schema<IFiles>({
  id: {
    type: Schema.Types.String,
    required: true,
  },
  date: {
    type: Schema.Types.Date,
    default: new Date(),
    required: true,
  },
  favourites: {
    type: Schema.Types.Number,
    default: 0,
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
  nsfw: {
    type: Schema.Types.Boolean,
    default: false,
    required: true,
  },
  uploader: {
    type: Schema.Types.String,
    required: true,
  },
});

export const Files = model<IFiles>("files", filesSchema);
