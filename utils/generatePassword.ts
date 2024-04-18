import bcrypt from "bcryptjs";

/**
 * Creates a bcrypt hashed password
 * @param password The password to hash
 * @returns {string}
 */
export default function(password: string) {
  const pass = bcrypt.hashSync(password, 12);

  return pass;
}
