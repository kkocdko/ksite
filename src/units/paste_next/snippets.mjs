import crypto from "crypto";

let iv; // initialization vector

async function encrypt(data, key) {
  iv = crypto.getRandomValues(new Uint8Array(16)); // the iv must never be reused with a given key
  return crypto.subtle.encrypt({ name: "AES-CBC", iv }, key, data);
}

async function decrypt(data, key) {
  return crypto.subtle.decrypt({ name: "AES-CBC", iv }, key, data);
}

crypto.subtle
  .generateKey({ name: "AES-CBC", length: 128 }, true, ["encrypt", "decrypt"])
  .then(async (key) => {
    let origin = new Uint8Array([...Array(6).keys()]);
    let encrypted = new Uint8Array(await encrypt(origin, key));
    // let encrypted2 = new Uint8Array(await encryptMessage(origin, key));
    let decryoted = new Uint8Array(await decrypt(encrypted, key));
    console.log({ origin, encrypted, decryoted, iv });
  });
