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

const config = useRuntimeConfig();
const pool = createPool({
  host: config.DBHost,
  user: config.DBUser,
  password: config.DBPassword,
  port: Number(config.DBPort),
  database: config.DBName,
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
