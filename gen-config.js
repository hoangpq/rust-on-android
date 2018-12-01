const fs = require("fs");
const os = require("os");

const homeDir = os.homedir();
const configTemplate = fs.readFileSync("./cargo-config.toml").toString();
const config = configTemplate.replace(
  /\$NDK_TOOLCHAIN/g,
  `${homeDir}/ndk-standalone`
);

fs.writeFileSync(`${homeDir}/.cargo/config`, config);
