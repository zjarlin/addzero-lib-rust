import { spawn } from "node:child_process";
import { writeFileSync } from "node:fs";

const pidFile = new URL("../.aio-front.pid", import.meta.url);
const appCwd = new URL("..", import.meta.url);

const child = spawn("pnpm", ["exec", "vite", "--host", "127.0.0.1"], {
    cwd: appCwd,
    detached: false,
    stdio: "inherit",
});

writeFileSync(pidFile, `${child.pid}\n`);

child.on("exit", () => {
    try {
        writeFileSync(pidFile, "");
    } catch {
        // ignore
    }
});
