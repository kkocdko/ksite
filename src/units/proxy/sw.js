/// <reference lib="webworker" />

const appendFn = () => {
  console.log("appended");
  document.querySelectorAll("img");
};
const append = `<script>${appendFn.toString()}</script>`;

/** @param {FetchEvent} e */
const inFetch = async (e) => {
  const req = e.request;
  const originUrl = req.url;
  req.url = "/proxy/inline?" + originUrl;
  let rep = await fetch(req);
  let mime = rep.headers.get("content-type");
  if (mime && (mime === "text/html" || mime.includes("text/html;"))) {
    let headers = new Headers(rep.headers); // response.headers is read-only
    headers.delete("content-security-policy");
    headers.delete("content-security-policy-report-only");
    headers.delete("report-to");
    let body = await rep.text();
    body = `<base href="${originUrl}">` + body;
    body += append;
    rep = new Response(body, {
      status: rep.status,
      statusText: rep.statusText,
      headers,
    });
  }
  return rep;
};
onfetch = (e) => e.respondWith(inFetch(e)); // remember to use `respondWith`
