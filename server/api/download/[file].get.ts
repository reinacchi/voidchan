import getConnection from "~~/server/database";

export default defineEventHandler(async (event) => {
  const fileParam = getRouterParam(event, "file") as string;
  const conn = await getConnection();
  const file = await conn.query("SELECT * FROM files WHERE id = ?", [fileParam.split(".")[0]]);

  if (file[0]) {
    const image = Buffer.from(file[0].buffer.split(",")[1], "base64");

    event.node.res.setHeader("Content-Disposition", `attachment;filename="${fileParam}"`);
    return send(event, image);
  } else {
    return {
      code: 404,
      message: "File Not Found"
    }
  }
});
