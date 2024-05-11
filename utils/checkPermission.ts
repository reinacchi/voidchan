import { ClearanceLevel } from "~~/server/database/models/profile.model";

/**
 *
 * @param userID The username
 * @param The clearance levelsof the user
 * @param levels The clearance levels to check (in array)
 * @returns {boolean}
 */
export function checkPermission(userLevels: ClearanceLevel[], levels: ClearanceLevel[]): boolean {
  for (let value of levels) {
    if (userLevels.includes(value)) {
      return true;
    }
  }

  return false;
}
