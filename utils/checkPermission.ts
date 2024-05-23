import { ClearanceLevels } from "~~/server/database";

/**
 *
 * @param userID The username
 * @param userLevels The clearance levelsof the user
 * @param levels The clearance levels to check
 * @returns {boolean}
 */
export function checkPermission(userLevels: ClearanceLevels[], levels: ClearanceLevels[]): boolean {
  for (let value of levels) {
    if (userLevels.includes(value)) {
      return true;
    }
  }

  return false;
}
