onfetch = (e) =>
  e.respondWith(
    (async () => {
      console.log(e);
      return fetch(e.request);
    })()
  );

// remember to use `respondWith`
// fetch("/proxy/http", {
//   mode: "cors",
//   method: "post",
//   body: JSON.stringify(e.request),
// });
