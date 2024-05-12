import { ClearanceLevel } from "~~/server/database/models/profile.model";

/**
 *
 * @param userID The username
 * @param userLevels The clearance levelsof the user
 * @param levels The clearance levels to check
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
