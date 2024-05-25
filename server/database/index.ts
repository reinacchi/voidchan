import { createPool, PoolConnection } from "mariadb";

export type clearance_levels =
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
  auth_key: string;
  clearance_levels: string;
  created_at: Date;
  display_name: string;
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
  connectionLimit: 10,
  timeout: 120000,
  connectTimeout: 120000,
  acquireTimeout: 120000,
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
