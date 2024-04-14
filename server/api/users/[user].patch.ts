import { Profile } from "../../database/models/profile.model";
import { getServerSession } from "#auth";

const config = useRuntimeConfig();

export default defineEventHandler(async (event) => {
  const session = await getServerSession(event) as any;
  const name = getRouterParam(event, "user") as string;

  if (!session) {
    return {
      code: 403,
      message: "Not Allowed",
    }
  }

  if (name !== session.user.name) {
    return {
      code: 403,
      message: "Not Allowed",
    }
  }

  if (event.node.req.headers.authorization !== config.PrivateAuth) {
    return {
      code: 403,
      message: "Not Allowed",
    }
  }

  const body = await readBody(event);

  await Profile.findOneAndUpdate({ name }, { $set: body });
});
