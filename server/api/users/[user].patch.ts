import getConnection from "~~/server/database";
import { getServerSession } from "#auth";

const config = useRuntimeConfig();

export default defineEventHandler(async (event) => {
  const session = await getServerSession(event) as any;
  const name = getRouterParam(event, "user") as string;
  const conn = await getConnection();

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
  const fields = Object.keys(body).map(key => `${key} = ?`).join(", ");
  const values = Object.values(body);
  values.push(name);

  await conn.execute(`UPDATE users SET ${fields} WHERE username = ?`, values);
});
