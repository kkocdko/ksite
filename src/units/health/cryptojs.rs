//! Incomplete reimplement of
//! [crypto-js](https://cdnjs.cloudflare.com/ajax/libs/crypto-js/4.1.1/crypto-js.js)
//! with pure Rust.
//!
//! # Why
//!
//! According to JavaScript's weakness and dynamic features, the crypto-js allows many unnformal
//! operations like parsing negative utf-8 code. So I decide to port it for my `health-check-in`
//! function.
//!
//! # Limitation
//!
//! Only supports `AES` + `PKCS7` + `ECB`.

/*
https://stackoverflow.com/questions/70212075/how-to-make-unsigned-right-shift-in-rust
https://stackoverflow.com/questions/51571066/what-are-the-exact-semantics-of-rusts-shift-operators
https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Unsigned_right_shift
*/

const BASE64_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=";

fn div_ceil_posi(lhs: i32, rhs: i32) -> i32 {
    debug_assert!(lhs >= 0);
    debug_assert!(rhs >= 0);
    let d = lhs / rhs;
    let r = lhs % rhs;
    if r > 0 && rhs > 0 {
        d + 1
    } else {
        d
    }
}

struct WordArray {
    words: Vec<i32>,
    sig_bytes: i32,
}

impl WordArray {
    fn concat(&mut self, t: WordArray) {
        self.clamp();
        let e = &mut self.words;
        let i = self.sig_bytes;
        let s = &t.words;
        let h = t.sig_bytes;
        if i % 4 != 0 {
            let mut t = 0;
            while t < h {
                let h = ((s[(t as u32 >> 2) as usize] as u32 >> (24 - (t % 4) * 8)) & 255) as i32;
                let idx = ((i + t) as u32 >> 2) as usize;
                if e.len() == idx {
                    e.push(0);
                }
                e[idx] |= h << (24 - ((i + t) % 4) * 8);
                t += 1;
            }
        } else {
            let mut t = 0;
            while t < h {
                let idx = ((i + t) as u32 >> 2) as usize;
                if e.len() == idx {
                    e.push(0);
                }
                e[idx] = s[(t as u32 >> 2) as usize];
                t += 4;
            }
        }
        self.sig_bytes += h;
    }
    fn clamp(&mut self) {
        let e = &mut self.words;
        let s = self.sig_bytes;
        let idx = (s as u32 >> 2) as usize;
        if e.len() == idx {
            e.push(0);
        }
        e[idx] &= (4294967295u32 << ((32 - (s % 4) * 8) % 32)) as i32;
        e.truncate(div_ceil_posi(s, 4) as _);
    }
    fn to_base64(&self) -> String {
        let sig_bytes = self.sig_bytes;
        let words = &self.words;
        let mut r = Vec::new();
        let mut i = 0;
        while i < sig_bytes {
            let mut s = 0;
            if let Some(&w) = words.get((i as u32 >> 2) as usize) {
                s |= ((w as u32 >> (24 - (i % 4) * 8)) & 255) << 16;
            }
            if let Some(&w) = words.get(((i as u32 + 1) >> 2) as usize) {
                s |= ((w as u32 >> (24 - ((i + 1) % 4) * 8)) & 255) << 8;
            }
            if let Some(&w) = words.get(((i as u32 + 2) >> 2) as usize) {
                s |= (w as u32 >> (24 - ((i + 2) % 4) * 8)) & 255;
            }
            let mut j = 0;
            while j < 4 && 4 * i + 3 * j < 4 * sig_bytes {
                r.push(BASE64_CHARS[((s >> (6 * (3 - j))) & 63) as usize]);
                j += 1;
            }
            i += 3;
        }
        while r.len() % 4 != 0 {
            r.push(b'=');
        }
        String::from_utf8(r).unwrap()
    }
}

fn utf8_parse(t: String) -> WordArray {
    let t = t.into_bytes();
    let e = t.len();
    let the_max = (e - 1) >> 2;
    let mut s = vec![0i32; the_max + 1];
    for i in 0..e {
        s[i >> 2] |= (255 & t[i] as i32) << (24 - (i % 4) * 8);
    }
    WordArray {
        words: s,
        sig_bytes: e as _,
    }
}

fn pkcs7_pad(t: &mut WordArray, e: i32) {
    let s = e * 4;
    let i = s - t.sig_bytes % s;
    let r = (i << 24) | (i << 16) | (i << 8) | i;
    let mut n = Vec::new();
    let mut j = 0;
    while j < i {
        n.push(r);
        j += 4;
    }
    let padding = WordArray {
        words: n,
        sig_bytes: i,
    };
    t.concat(padding);
}

fn aes_encrypt(words: WordArray, key: WordArray, _cfg: Option<()>) -> WordArray {
    const fn gen_consts() -> [[i32; 256]; 5] {
        let mut h = [0; 256];
        let mut n = [0; 256];
        let mut c = [0; 256];
        let mut l = [0; 256];
        let mut o = [0; 256];
        let mut t = [0usize; 256];
        let mut e = 0;
        while e < 256 {
            t[e] = if e < 128 { e << 1 } else { (e << 1) ^ 283 };
            e += 1;
        }
        let mut e = 0;
        let mut s = 0;
        let mut i = 0;
        while i < 256 {
            let mut j = s ^ (s << 1) ^ (s << 2) ^ (s << 3) ^ (s << 4);
            j = (j >> 8) ^ (255 & j) ^ 99;
            h[e] = j as _;
            let d = t[e];
            let z = t[d];
            let u = t[z];
            let y = (257 * t[j]) ^ (16843008 * j);
            n[e] = ((y << 24) | (y >> 8)) as _;
            c[e] = ((y << 16) | (y >> 16)) as _;
            l[e] = ((y << 8) | (y >> 24)) as _;
            o[e] = y as _;
            if e != 0 {
                e = d ^ t[t[t[u ^ d]]];
                s ^= t[t[s]];
            } else {
                s = 1;
                e = 1;
            }
            i += 1;
        }
        [h, n, c, l, o]
    }
    const RR_: [[i32; 256]; 5] = gen_consts();
    let [h, n, c, l, o] = RR_;
    let d = [0, 1, 2, 4, 8, 16, 32, 64, 128, 27, 54];

    let mut l_twa = words;
    let l_nr9;
    let l_be = 4;
    let mut l_k4 = Vec::new();
    {
        // _doReset()
        let words = key.words;
        let sbm4 = (key.sig_bytes / 4) as usize;
        l_nr9 = sbm4 + 6;
        let n = 4 * (l_nr9 + 1) as usize;
        let mut t;
        let mut i = 0;
        while i < n {
            if i < sbm4 {
                l_k4.push(words[i]);
            } else {
                t = l_k4[i - 1];
                if i % sbm4 == 0 {
                    t = (t << 8) | ((t as u32 >> 24) as i32);
                    let t2 = t as u32;
                    t = (h[(t2 >> 24) as usize] << 24)
                        | (h[(t2 >> 16 & 255) as usize] << 16)
                        | (h[(t2 >> 8 & 255) as usize] << 8)
                        | h[255 & t2 as usize];
                    t ^= d[i / sbm4] << 24;
                } else if sbm4 > 6 && i % sbm4 == 4 {
                    let t2 = t as u32;
                    t = (h[(t2 >> 24) as usize] << 24)
                        | (h[(t2 >> 16 & 255) as usize] << 16)
                        | (h[(t2 >> 8 & 255) as usize] << 8)
                        | h[255 & t2 as usize];
                }
                l_k4.push(l_k4[i - sbm4] ^ t);
            }
            i += 1;
        }
    }
    {
        // f5()
        pkcs7_pad(&mut l_twa, l_be);
    }
    {
        // p8()
        let mut s = Vec::new();
        let lout = div_ceil_posi(l_twa.sig_bytes, 4 * l_be);
        let lxlbe = lout * l_be;
        let a = l_twa.sig_bytes.min(4 * lxlbe);
        if lxlbe != 0 {
            let mut i = 0;
            while i < lxlbe {
                let e = i as usize;
                let mut k = l_twa.words[e] ^ l_k4[0];
                let mut a = l_twa.words[e + 1] ^ l_k4[1];
                let mut p = l_twa.words[e + 2] ^ l_k4[2];
                let mut f = l_twa.words[e + 3] ^ l_k4[3];
                let mut g = 4;
                let mut j = 1;
                while j < l_nr9 {
                    macro_rules! vs_n {
                        ($k0:expr, $k1:expr, $k2:expr, $k3:expr) => {{
                            let vs_n_v = n[($k0 as u32 >> 24) as usize]
                                ^ c[($k1 as u32 >> 16 & 255) as usize]
                                ^ l[($k2 as u32 >> 8 & 255) as usize]
                                ^ o[255 & $k3 as usize]
                                ^ l_k4[g];
                            g += 1;
                            vs_n_v
                        }};
                    }
                    let vs0 = vs_n!(k, a, p, f);
                    let vs1 = vs_n!(a, p, f, k);
                    let vs2 = vs_n!(p, f, k, a);
                    let vs3 = vs_n!(f, k, a, p);
                    k = vs0;
                    a = vs1;
                    p = vs2;
                    f = vs3;
                    j += 1;
                }
                macro_rules! te_n {
                    ($k0:expr, $k1:expr, $k2:expr, $k3:expr) => {{
                        #[allow(unused_assignments)]
                        {
                            let te_n_v = ((h[($k0 as u32 >> 24) as usize] << 24)
                                | (h[($k1 as u32 >> 16 & 255) as usize] << 16)
                                | (h[($k2 as u32 >> 8 & 255) as usize] << 8)
                                | h[255 & $k3 as usize])
                                ^ l_k4[g];
                            g += 1;
                            te_n_v
                        }
                    }};
                }
                let te0 = te_n!(k, a, p, f);
                let te1 = te_n!(a, p, f, k);
                let te2 = te_n!(p, f, k, a);
                let te3 = te_n!(f, k, a, p);
                l_twa.words[e] = te0;
                l_twa.words[e + 1] = te1;
                l_twa.words[e + 2] = te2;
                l_twa.words[e + 3] = te3;
                i += l_be;
            }
            s = l_twa.words.drain(0..lxlbe as usize).collect();
            l_twa.sig_bytes -= a;
        }
        l_twa = WordArray {
            sig_bytes: if a == 0 { 4 * s.len() as i32 } else { a },
            words: s,
        };
    }
    l_twa.clamp();
    l_twa
}

fn btoa(s: &str) -> String {
    // https://github.com/zloirock/core-js/blob/master/packages/core-js/modules/web.btoa.js
    let s = s.as_bytes();
    let mut r = Vec::new();
    let mut map = BASE64_CHARS;
    let mut block = 0usize;
    let mut position = 0;
    let len = s.len();
    while {
        if position < len * 4 {
            true
        } else {
            map = b"=";
            position % 4 != 0
        }
    } {
        position += 3;
        let char_code = *s.get(position / 4).unwrap_or(&0) as usize;
        block = block << 8 | char_code;
        r.push(map[63 & block >> (8 - (position % 4 * 2))])
    }
    unsafe { String::from_utf8_unchecked(r) }
}

/// Encrypt the string for JUST's form submit api.
pub fn encrypt(s: String) -> String {
    let words = utf8_parse(s);
    let key = WordArray {
        words: vec![1947217763, 1550666530, -1301273701, -1041739952],
        sig_bytes: 16,
    };
    btoa(&aes_encrypt(words, key, None).to_base64())
}
