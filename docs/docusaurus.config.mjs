import {themes as prismThemes} from "prism-react-renderer";
import {fileURLToPath} from "node:url";
import path from "node:path";
import {createDocusaurusConfig, loadConfig} from "xiaoeyu";

const currentDir = path.dirname(fileURLToPath(import.meta.url));
const {config: xiaoeyuConfig} = loadConfig("../xiaoeyu.config.json", currentDir);

const config = createDocusaurusConfig(xiaoeyuConfig, {
  customCss: "./custom.css",
  themeConfig: {
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula
    }
  }
});

export default config;
