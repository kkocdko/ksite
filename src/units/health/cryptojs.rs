use crate::utils::encode_uri;

pub fn encrypt(s: String) -> String {
    let words = utf8_parse(s);
    let key = Key {
        words: vec![1947217763, 1550666530, -1301273701, -1041739952],
        sigBytes: 16,
    };
    todo!()
}

fn utf8_parse(t: String) -> WordArray {
    let t = t.into_bytes();
    let e = t.len();
    let mut the_max = 0;
    for i in 0..e {
        the_max = the_max.max(i >> 2); // opti?
    }
    let mut s = vec![0i32; the_max];
    for i in 0..e {
        s[i >> 2] |= (255 & t[i] as i32) << (24 - (i % 4) * 8);
    }
    WordArray {
        words: s,
        sigBytes: e as _,
    }
}

struct WordArray {
    words: Vec<i32>,
    sigBytes: i32,
}

impl WordArray {
    fn concat(&mut self, wordArray: WordArray) {
        // // Shortcuts
        let thisWords = &self.words;
        let thatWords = &wordArray.words;
        let thisSigBytes = self.sigBytes;
        let thatSigBytes = wordArray.sigBytes;

        // // Clamp excess bits
        // this.clamp();

        // // Concat
        // if (thisSigBytes % 4) {
        //     // Copy one byte at a time
        //     for (var i = 0; i < thatSigBytes; i++) {
        //         var thatByte = (thatWords[i >>> 2] >>> (24 - (i % 4) * 8)) & 0xff;
        //         thisWords[(thisSigBytes + i) >>> 2] |= thatByte << (24 - ((thisSigBytes + i) % 4) * 8);
        //     }
        // } else {
        //     // Copy one word at a time
        //     for (var j = 0; j < thatSigBytes; j += 4) {
        //         thisWords[(thisSigBytes + j) >>> 2] = thatWords[j >>> 2];
        //     }
        // }
        // this.sigBytes += thatSigBytes;

        // // Chainable
        // return this;
    }
    fn clamp(&mut self) {
        let e = &mut self.words;
        let s = self.sigBytes;
        let s_urs_2 = s as u32 >> 2;
        e[s_urs_2 as usize] &= 4294967295 << (32 - (s % 4) * 8);
        e.truncate((s / 4) as _);
    }
}

struct Key {
    words: Vec<i32>,
    sigBytes: i32,
}

fn pkcs7_pad(data: &mut WordArray, blockSize: i32) {
    let blockSizeBytes = blockSize * 4;
    let nPaddingBytes = blockSizeBytes - data.sigBytes % blockSizeBytes;
    let paddingWord =
        (nPaddingBytes << 24) | (nPaddingBytes << 16) | (nPaddingBytes << 8) | nPaddingBytes;
    let mut paddingWords = Vec::new();
    let mut i = 0;
    while i < nPaddingBytes {
        paddingWords.push(paddingWord);
        i += 4;
    }
    let padding = WordArray {
        words: paddingWords,
        sigBytes: nPaddingBytes,
    };
    // data.concat(padding);
}
