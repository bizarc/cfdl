import fs from "node:fs";
import path from "node:path";

/**
 * How many exercises the synced bundle carries.
 *
 * Read rather than stated, for the same reason the chapter list is derived:
 * a hand-kept count is a second place to update, and it drifts.
 */
export function getExerciseCount(): number {
  const p = path.join(process.cwd(), "content", "exercises.json");
  try {
    return Object.keys(JSON.parse(fs.readFileSync(p, "utf8"))).length;
  } catch {
    return 0;
  }
}
