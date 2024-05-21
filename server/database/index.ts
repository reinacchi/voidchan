import { createPool, PoolConnection } from "mariadb";

export type ClearanceLevels =
  | "Project Lead"
  | "Developer"
  | "Moderator"
  | "Nominator"
  | "Contributor"
  | "Verified"
  | "Plus"
  | "Alumni"
  | "Supporter"
  | "Member";

export interface IUser {
  authKey: string;
  clearanceLevels: string;
  createdAt: Date;
  displayName: string;
  id: number;
  kudos: number;
  email: string;
  password: string;
  username: string;
}

const pool = createPool({
  host: "localhost",
  user: "root",
  port: 3306,
  database: "voidchan_db",
  connectionLimit: 5,
});

export default async function getConnection() {
  let conn;
  try {
    conn = await pool.getConnection();
  } finally {
    if (conn) conn.release();
  }

  return conn as PoolConnection;
}
