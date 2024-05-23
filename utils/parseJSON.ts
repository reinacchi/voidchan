/**
 * Validate a JSON before parsing
 * @param str The string to parse
 * @returns {string}
 */
export function parseJSON(str: string): string {
  try {
    JSON.parse(str);
  } catch (err) {
    return str;
  }

  return JSON.parse(str);
}
