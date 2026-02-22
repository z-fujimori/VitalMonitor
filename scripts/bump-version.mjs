import fs from "fs";

const v = process.argv[2];
if (!v || !/^\d+\.\d+\.\d+$/.test(v)) {
  console.error("Usage: node scripts/bump-version.mjs 0.2.0");
  process.exit(1);
}

function writeJson(path, patch) {
  const j = JSON.parse(fs.readFileSync(path, "utf8"));
  patch(j);
  fs.writeFileSync(path, JSON.stringify(j, null, 2) + "\n");
}

writeJson("package.json", (j) => (j.version = v));
writeJson("src-tauri/tauri.conf.json", (j) => (j.version = v));

const cargoPath = "src-tauri/Cargo.toml";
let cargo = fs.readFileSync(cargoPath, "utf8");
cargo = cargo.replace(/^version\s*=\s*\"[^\"]+\"/m, `version = "${v}"`);
fs.writeFileSync(cargoPath, cargo);

console.log("Bumped version to", v);
