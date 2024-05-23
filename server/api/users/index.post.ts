import getConnection from "~~/server/database";
import { getServerSession } from "#auth";
import { generateString } from "~~/utils/generateString";

export default defineEventHandler(async (event) => {
  const session = (await getServerSession(event)) as any;
  const body = await readBody(event);
  const conn = await getConnection();
  const user = await conn.query("SELECT * FROM users WHERE username = ?", [
    session.user.name,
  ]);
  const newUser = await conn.query("SELECT * FROM users WHERE username = ?", [
    body.username,
  ]);

  if (!session) {
    return {
      code: 403,
      message: "Not Allowed",
    };
  }

  if (!user[0].clearanceLevels.includes("Moderator")) {
    return {
      code: 403,
      message: "Not Allowed",
    };
  }

  if (newUser[0]) {
    return {
      code: 400,
      message: "This account has existed",
    };
  }

  const authKey = generateString(32);
  const password = generatePassword(body.password);

  await conn.execute(
    "INSERT INTO users (clearanceLevels, displayName, kudos, authKey, createdAt, email, username, password) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    [
      ["Member"],
      body.username,
      0,
      authKey,
      new Date(),
      body.email,
      body.username,
      password,
    ]
  );
});
