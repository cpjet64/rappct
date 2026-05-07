const fs = require("node:fs");
const path = require("node:path");

const root = path.resolve(__dirname, "..");

function readText(relativePath) {
  return fs.readFileSync(path.join(root, relativePath), "utf8");
}

function matchVersion(relativePath, pattern, description) {
  const match = readText(relativePath).match(pattern);
  if (!match) {
    throw new Error(`Could not find ${description} in ${relativePath}.`);
  }
  return match[1];
}

const cargoVersion = matchVersion("Cargo.toml", /^version\s*=\s*"([^"]+)"/m, "package version");
const lockVersion = matchVersion(
  "Cargo.lock",
  /\[\[package\]\]\s+name = "rappct"\s+version = "([^"]+)"/,
  "rappct lockfile package version",
);

const versionPattern = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/;
if (!versionPattern.test(cargoVersion)) {
  console.error(`Cargo.toml version must be major.minor.patch, found ${cargoVersion}.`);
  process.exit(1);
}

const mismatches = [["Cargo.lock rappct version", lockVersion]].filter(([, actual]) => actual !== cargoVersion);
if (mismatches.length > 0) {
  console.error(`Version surfaces do not match Cargo.toml version ${cargoVersion}:`);
  for (const [name, actual] of mismatches) {
    console.error(`- ${name}: ${actual ?? "<missing>"}`);
  }
  process.exit(1);
}

const releaseTag = process.env.CI_COMMIT_TAG;
if (releaseTag) {
  const expectedTag = `v${cargoVersion}`;
  if (releaseTag !== expectedTag) {
    console.error(`Release tag ${releaseTag} does not match crate version ${cargoVersion}; expected ${expectedTag}.`);
    process.exit(1);
  }
}

console.log(`Version surfaces aligned at ${cargoVersion}.`);
if (releaseTag) {
  console.log(`Release tag ${releaseTag} matches crate version ${cargoVersion}.`);
}
