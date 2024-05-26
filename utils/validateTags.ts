import getConnection from "~~/server/database"

/**
 * Validate tag names
 * @param tags Array of tags to validate
 * @returns {Promise<boolean>}
 */
export async function validateTags(tags: string[]): Promise<boolean> {
  const conn = await getConnection();
  const invalidChars = /[^a-z0-9_]/;

  try {
    // Check for invalid tags
    for (const tag of tags) {
      if (
        !tag || // Blank tag
        tag.includes('*') || // Contains asterisks
        tag.endsWith('_') || // Ends with an underscore
        /__/.test(tag) || // Consecutive underscores
        /[\x00-\x1F\x7F]/.test(tag) || // Non-printable characters
        tag.includes(',') || // Contains commas
        invalidChars.test(tag) || // Contains invalid characters
        tag !== tag.toLowerCase() || // Not in snake_case (all lowercase and underscores)
        /\s/.test(tag) // Contains spaces
      ) {
        return false;
      }
    }

    // Check if tags exist in the database
    const [rows] = await conn.execute(
      'SELECT name FROM tags WHERE name IN (?)',
      [tags]
    );

    const existingTags = (rows as any[]).map(row => row.name);

    // Ensure all provided tags exist in the database
    for (const tag of tags) {
      if (!existingTags.includes(tag)) {
        return false;
      }
    }

    return true;
  } catch (error) {
    console.error('Database query error:', error);
    return false;
  } finally {
    await conn.end();
  }
};
