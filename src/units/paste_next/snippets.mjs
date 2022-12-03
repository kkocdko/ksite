// import crypto from "crypto";

/* 
https://developer.mozilla.org/en-US/docs/Web/API/SourceBuffer
https://developer.mozilla.org/zh-CN/docs/Web/API/FormData
https://developer.mozilla.org/zh-CN/docs/Web/API/fetch
https://zhuanlan.zhihu.com/p/446145066
https://blog.csdn.net/weixin_39590639/article/details/99671730
https://www.bing.com/search?q=js+%E6%B5%81%E5%BC%8F%E4%B8%8B%E8%BD%BD&mkt=zh-CN
https://developer.mozilla.org/en-US/docs/Web/API/File_API/Using_files_from_web_applications
https://developer.mozilla.org/en-US/docs/Web/API/Blob/stream
https://www.bing.com/search?q=js+blob+stream&qs=n&form=QBRE&sp=-1&pq=js+fetch+body+stream&sc=10-20&sk=&cvid=29A5432EBAB34341AE2291B6F19F0712&ghsh=0&ghacc=0&ghpl=
https://github.com/mdn/dom-examples/blob/main/streams/png-transform-stream/png-transform-stream.js

*/

// let iv; // initialization vector
// async function encrypt(data, key) {
//   iv = crypto.getRandomValues(new Uint8Array(16)); // the iv must never be reused with a given key
//   return crypto.subtle.encrypt({ name: "AES-CBC", iv }, key, data);
// }
// async function decrypt(data, key) {
//   return crypto.subtle.decrypt({ name: "AES-CBC", iv }, key, data);
// }
// const key = await crypto.subtle.generateKey(AES128, true, ENCDEC);
// const keyData = new Uint8Array(await crypto.subtle.exportKey("raw", key));
// const keyData = new Uint8Array(
//   await crypto.subtle.digest("SHA-256", new Uint8Array([...Array(6).keys()]))
// ).slice(0, 16);
// const key = await crypto.subtle.importKey(
//   "raw",
//   keyData,
//   AESALGO,
//   true,
//   ENCDEC
// );

const toBase64 = async (arrbuf) => {
  // https://stackoverflow.com/a/66046176/11338291
  const reader = new FileReader();
  const promise = new Promise((r) => (reader.onload = r));
  reader.readAsDataURL(new Blob([arrbuf]));
  return promise.then(() => reader.result.split(",", 2)[1]);
};

// console.log(
//   new Uint8Array(
//     await crypto.subtle.digest("SHA-256", new Uint8Array([...Array(6).keys()]))
//   ).slice(0, 16)
// );
// return;
// let origin = new Uint8Array([...Array(6).keys()]);
// let encrypted = new Uint8Array(await encrypt(origin, key));
// // let encrypted2 = new Uint8Array(await encryptMessage(origin, key));
// let decryoted = new Uint8Array(await decrypt(encrypted, key));
// // console.log({ origin, encrypted, decryoted, iv });

const upwHash = await crypto.subtle.digest(
  "SHA-256",
  new Uint8Array([...Array(70).keys()])
);

const AES_INIT_VECTOR = new Uint8Array(
  // [...crypto.getRandomValues(new Uint8Array(16))].join()
  `80,25,201,247,1,153,204,120,30,195,70,73,218,3,89,81`.split(",")
);
const AES_ALGO = { name: "AES-CBC", length: 128, iv: AES_INIT_VECTOR };
const AES_KEY_USAGE = ["encrypt", "decrypt"];

const fpwRaw = crypto.getRandomValues(new Uint8Array(16)); // key = fpwRaw
// const fpwRaw = new Uint8Array([...Array(32).keys()]);
// const fpwEncKey = await crypto.subtle.importKey(
//   "raw",
//   upwHash,
//   AES_ALGO,
//   true,
//   AES_KEY_USAGE
// );
// const fpwEnc = await crypto.subtle.encrypt(AES_ALGO, fpwEncKey, fpwRaw);
// const fpwDec = await crypto.subtle.decrypt(AES_ALGO, fpwEncKey, fpwEnc);
// console.log(fpwRaw, new Uint8Array(fpwEnc), new Uint8Array(fpwDec));

const fpw = await crypto.subtle.importKey(
  "raw",
  fpwRaw,
  AES_ALGO,
  true,
  AES_KEY_USAGE
);

/**
 * @param {ReadableStream<Uint8Array>} readable
 * @param {number} chunkSize
 * @param {{ (chunk: Uint8Array): Promise<Uint8Array> }} processor
 * @returns {ReadableStream<Uint8Array>}
 */
const chunkStream = (readable, chunkSize, processor) => {
  return new ReadableStream({
    async start(controller) {
      const reader = readable.getReader();
      let bufferLength = chunkSize;
      let buffer = new Uint8Array(bufferLength);
      let bufferIndex = 0;
      while (true) {
        // MDN says:
        // If a chunk is available to read, { value: theChunk, done: false }.
        // If the stream becomes closed, { value: undefined, done: true }.
        const { done, value: source } = await reader.read();
        if (done) {
          controller.enqueue(await processor(buffer.slice(0, bufferIndex)));
          controller.close();
          return;
        }
        let sourceIndex = 0;
        let sourceLength = source.length;
        while (sourceIndex != sourceLength) {
          if (bufferIndex == bufferLength) {
            // console.log(buffer);
            // console.log(buffer.find((v) => v != 0));
            controller.enqueue(await processor(buffer));
            buffer = new Uint8Array(bufferLength);
            bufferIndex = 0;
          }
          let insertLength = Math.min(
            sourceLength - sourceIndex,
            bufferLength - bufferIndex
          );
          buffer.set(
            source.slice(sourceIndex, sourceIndex + insertLength),
            bufferIndex
          );
          sourceIndex += insertLength;
          bufferIndex += insertLength;
        }
      }
    },
  });
};

/**
 * @param {ReadableStream<Uint8Array>} readable
 * @param {CryptoKey} key
 * @returns {ReadableStream<Uint8Array>}
 */
const encryptStream = (readable, key) => {
  const CHUNK_RAW_LEN = 512 * 1024;
  return chunkStream(readable, CHUNK_RAW_LEN, (chunk) =>
    crypto.subtle.encrypt(AES_ALGO, key, chunk).then((v) => new Uint8Array(v))
  );
};

/**
 * @param {ReadableStream<Uint8Array>} readable
 * @param {CryptoKey} key
 * @returns {ReadableStream<Uint8Array>}
 */
const decryptStream = (readable, key) => {
  const CHUNK_AES_LEN = 512 * 1024 + 16;
  return chunkStream(readable, CHUNK_AES_LEN, (chunk) =>
    crypto.subtle.decrypt(AES_ALGO, key, chunk).then((v) => new Uint8Array(v))
  );
};

/**
 * @param {string} text
 * @returns {ReadableStream<Uint8Array>}
 */
const textStream = (text) => {
  return new ReadableStream({
    async start(controller) {
      const textEncoder = new TextEncoder();
      const buffer = textEncoder.encode(text);
      controller.enqueue(buffer);
      controller.close();
    },
  });
};

{
  new Promise((r) =>
    r(textStream("hello world this is kkocdko ".repeat(30000)))
  )
    .then((stream) => encryptStream(stream, fpw))
    .then((stream) =>
      fetch(location.origin + "/up", {
        method: "POST",
        duplex: "half",
        body: stream,
      })
    );
  // .then((stream) => decryptStream(stream, fpw))
  // .then((stream) => new Response(stream))
  // .then(async (response) => {
  //   console.log((await response.text()).slice(0, 256));
  // });

  // fetch("http://127.0.0.1:9005/test")
  //   .then((response) => response.body)
  //   .then((stream) => encryptStream(stream, fpw))
  //   .then((stream) => decryptStream(stream, fpw))
  //   .then((stream) => new Response(stream))
  //   .then(async (response) => {
  //     console.log(response);
  //   });
}

//   return new Response(stream, { headers: { "Content-Type": "text/html" } });
// });
// because one fpw_raw only pair to one file, so we use the same iv
// fpw_enc = aes(fpw_raw, upw)
// key + lv + upw -> raw
//
// console.log(await toBase64(new Uint8Array([...Array(48).keys()])));
// https://crypto.stackexchange.com/a/3970
// user forest says: For CTR, you can use a null (all zero) IV / nonce if you
// wish, as long as you don't encrypt multiple messages with the same key. Just
// make sure the key:nonce tuple never repeats.

// backlog becaused of https://bugzilla.mozilla.org/show_bug.cgi?id=1469359
