import { Schema, model } from "mongoose";

type ClearanceLevel = "Project Lead" | "Developer" | "Moderator" | "Nominator" | "Contributor" | "Verified" | "Plus" | "Alumni" | "Supporter" | "Member";

export interface IProfile {
  authKey: string;
  clearanceLevel: ClearanceLevel[];
  date: Date;
  displayName: string;
  email: string;
  files: string[];
  password: string;
  posts: number[];
  name: string;
}

const profileSchema = new Schema<IProfile>({
  clearanceLevel: {
    type: [Schema.Types.String],
    default: ["Member"],
    required: true,
  },
  authKey: {
    type: Schema.Types.String,
    required: true,
  },
  date: {
    type: Schema.Types.Date,
    default: new Date(),
    required: true,
  },
  displayName: {
    type: Schema.Types.String,
    required: false,
  },
  email: {
    type: Schema.Types.String,
    required: false,
  },
  files: {
    type: [Schema.Types.String],
    default: [],
    required: true,
  },
  password: {
    type: Schema.Types.String,
    required: true,
  },
  posts: {
    type: [Schema.Types.Number],
    default: [],
    required: true,
  },
  name: {
    type: Schema.Types.String,
    required: true,
  },
});

export const Profile = model<IProfile>("profile", profileSchema);
