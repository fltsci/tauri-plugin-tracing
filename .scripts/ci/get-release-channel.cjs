// @ts-check

const path = require("node:path");
const fs = require("node:fs");

const fn = () => {
  const loc = path.resolve(__dirname, "../../.changes/pre.json");
  if (!fs.existsSync(loc)) {
    return "latest";
  }
  const data = fs.readFileSync(loc, { encoding: "utf-8" });
  const obj = JSON.parse(data);
  const out = obj?.tag ?? "latest";
  return out.toString();
};

console.log(fn());

module.exports = () => fn();
