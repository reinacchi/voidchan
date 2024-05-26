/**
 * Retrieve the count of a tag that exists in every post
 * @param allTags All tags in every post
 * @param tag The tag to match
 * @returns {number}
 */
export function countTagOccurrences(allTags: any, tag: string): number {
  let count = 0;

  allTags.forEach((post: any) => {
    const tagsArray = post.tags || [];
    count += tagsArray.filter((t: any) => t === tag).length;
  });

  return count;
};
