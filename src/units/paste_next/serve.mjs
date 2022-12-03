import { createServer } from "node:http";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const modulePath = fileURLToPath(import.meta.url);
const port = 4000;
const mime = { html: "text/html;charset=utf8", js: "text/javascript" };
const r2a = path.join.bind(null, path.dirname(modulePath));
createServer(({ url, method, ...req }, res) => {
  console.log([method, r2a(url)]);
  if (method === "POST") {
    const requestBody = [];
    req.on("data", (chunks) => {
      requestBody.push(chunks);
    });
    req.on("end", () => {
      const parsedData = Buffer.concat(requestBody);
      fs.writeFileSync("test", parsedData);
    });
    return;
  }
  const pair = [
    [200, r2a(url)],
    [200, r2a(url, "index.html")],
    [404, r2a("404.html")],
  ].find(([_, p]) => fs.existsSync(p) && fs.statSync(p).isFile());
  if (!pair) return res.writeHead(404).end("404 Not Found");
  const [status, local] = pair;
  res.setHeader("content-type", mime[local.split(".").pop()] || "");
  res.writeHead(status).end(fs.readFileSync(local));
}).listen(port);
console.info(`server: 127.0.0.1:${port}`);
