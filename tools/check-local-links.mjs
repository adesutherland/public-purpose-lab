import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import { dirname, extname, resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const excludedDirectories = new Set([
  ".git",
  ".idea",
  "dist",
  "node_modules",
  "target",
]);
const linkPattern = /!?\[[^\]]*\]\(([^)]+)\)/g;
const markdownFiles = [];

function visit(directory) {
  for (const name of readdirSync(directory)) {
    if (excludedDirectories.has(name)) {
      continue;
    }

    const path = resolve(directory, name);
    const stats = statSync(path);
    if (stats.isDirectory()) {
      visit(path);
    } else if (extname(path) === ".md") {
      markdownFiles.push(path);
    }
  }
}

visit(root);

const failures = [];

for (const markdownFile of markdownFiles) {
  const source = readFileSync(markdownFile, "utf8");
  for (const match of source.matchAll(linkPattern)) {
    const rawTarget = match[1].trim();
    const targetWithOptionalTitle = rawTarget.startsWith("<")
      ? rawTarget.slice(1, rawTarget.indexOf(">"))
      : rawTarget.split(/\s+["']/u, 1)[0];

    if (
      targetWithOptionalTitle === "" ||
      targetWithOptionalTitle.startsWith("#") ||
      /^[a-z][a-z0-9+.-]*:/iu.test(targetWithOptionalTitle)
    ) {
      continue;
    }

    const targetPath = decodeURIComponent(
      targetWithOptionalTitle.split(/[?#]/u, 1)[0],
    );
    if (!existsSync(resolve(dirname(markdownFile), targetPath))) {
      failures.push(
        `${markdownFile.slice(root.length + 1)} -> ${targetWithOptionalTitle}`,
      );
    }
  }
}

if (failures.length > 0) {
  throw new Error(`Missing local Markdown targets:\n${failures.join("\n")}`);
}

console.log(`Local Markdown links OK: ${markdownFiles.length} files checked.`);
