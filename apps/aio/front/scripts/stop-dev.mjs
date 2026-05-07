import { execFileSync } from "node:child_process";
import { existsSync, readFileSync, unlinkSync } from "node:fs";
import { kill } from "node:process";

const pidFile = new URL("../.aio-front.pid", import.meta.url);
const appCwd = decodeURIComponent(new URL("..", import.meta.url).pathname).replace(
    /\/$/,
    "",
);

function readPid() {
    if (!existsSync(pidFile)) {
        return null;
    }
    const raw = readFileSync(pidFile, "utf8").trim();
    if (!/^\d+$/.test(raw)) {
        return null;
    }
    return Number(raw);
}

function processLineFor(pid) {
    try {
        return execFileSync("ps", ["-p", String(pid), "-o", "command="], {
            encoding: "utf8",
        }).trim();
    } catch {
        return "";
    }
}

function isExpectedProcess(pid) {
    const command = processLineFor(pid);
    return (
        command.includes(appCwd) ||
        (command.includes("vite") && command.includes("aio-front")) ||
        (command.includes("node") && command.includes("vite"))
    );
}

const pid = readPid();
if (!pid) {
    console.log("没有记录中的 AIO front dev 进程。");
    process.exit(0);
}

if (!isExpectedProcess(pid)) {
    console.error(`拒绝停止 PID ${pid}：它不像 AIO front dev 进程。`);
    process.exit(1);
}

try {
    kill(pid, "SIGTERM");
    console.log(`已停止 AIO front dev 进程：${pid}`);
} catch (error) {
    if (error && error.code === "ESRCH") {
        console.log(`AIO front dev 进程已不存在：${pid}`);
    } else {
        throw error;
    }
} finally {
    try {
        unlinkSync(pidFile);
    } catch {
        // ignore
    }
}
