/// <reference lib="webworker" />

/** @param {FetchEvent} e */
onfetch = (e) => {
  e.respondWith(
    (async () => {
      console.log(e);
      // e.request.headers.set("_origin_url_", e.request.url);
      e.request.url = "/proxy/inline?" + e.request.url;
      return fetch(e.request);
    })()
  );
};

// remember to use `respondWith`
// fetch("/proxy/http", {
//   mode: "cors",
//   method: "post",
//   body: JSON.stringify(e.request),
// });
