{
  const encrypt = (str) => {
    const words = CryptoJS.enc.Utf8.parse(str);
    const keyWords = [1947217763, 1550666530, -1301273701, -1041739952];
    const key = { words: keyWords, sigBytes: 16 }; // CryptoJS.MD5("zntb666666666666")
    const cfg = { mode: CryptoJS.mode.ECB, padding: CryptoJS.pad.Pkcs7 };
    return btoa(CryptoJS.AES.encrypt(words, key, cfg).toString());
  };
  let CryptoJS = {};
  let h = (CryptoJS.l = {});
  let r = (h.Base = {
    e(t) {
      let s = Object.create(this);
      return (
        t && s.m1(t),
        (s.hasOwnProperty("_") && this._ !== s._) ||
          (s._ = function () {
            s.sa._.apply(this, arguments);
          }),
        (s._.prototype = s),
        (s.sa = this),
        s
      );
    },
    _c() {
      let t = this.e();
      return t._.apply(t, arguments), t;
    },
    _: function () {},
    m1(t) {
      for (let e in t) t.hasOwnProperty(e) && (this[e] = t[e]);
      t.hasOwnProperty("toString") && (this.toString = t.toString);
    },
  });
  let WordArray = (h.wa = r.e({
    _: function (t, e) {
      (t = this.words = t || []),
        (this.sigBytes = null != e ? e : 4 * t.length);
    },
    toString(t) {
      return (t || Hex).st(this);
    },
    concat(t) {
      let e = this.words,
        s = t.words,
        i = this.sigBytes,
        h = t.sigBytes;
      if ((this.cp(), i % 4))
        for (let t = 0; t < h; t++) {
          let h = (s[t >>> 2] >>> (24 - (t % 4) * 8)) & 255;
          e[(i + t) >>> 2] |= h << (24 - ((i + t) % 4) * 8);
        }
      else for (let t = 0; t < h; t += 4) e[(i + t) >>> 2] = s[t >>> 2];
      return (this.sigBytes += h), this;
    },
    cp() {
      let e = this.words,
        s = this.sigBytes;
      (e[s >>> 2] &= 4294967295 << (32 - (s % 4) * 8)),
        (e.length = Math.ceil(s / 4));
    },
  }));
  let c = (CryptoJS.enc = {});
  let o = (c.Utf8 = {
    parse: (t) => {
      let e = t.length,
        s = [];
      for (let i = 0; i < e; i++)
        s[i >>> 2] |= (255 & t.charCodeAt(i)) << (24 - (i % 4) * 8);
        console.log(s.length)
      return new WordArray._(s, e);
    },
  });
  h.ba = r.e({
    rt() {
      (this._t = new WordArray._()), (this.nb = 0);
    },
    ap(t) {
      "string" == typeof t && (t = o.parse(t)),
        this._t.concat(t),
        (this.nb += t.sigBytes);
    },
    p8(e) {
      let s,
        i = this._t,
        h = i.words,
        r = i.sigBytes,
        c = this.be,
        l = r / (4 * c);
      l = e ? Math.ceil(l) : Math.max((0 | l) - this.m8, 0);
      let o = l * c,
        a = Math.min(4 * o, r);
      if (o) {
        for (let t = 0; t < o; t += c) this.k2(h, t);
        (s = h.splice(0, o)), (i.sigBytes -= a);
      }
      return new WordArray._(s, a);
    },
    m8: 0,
  });
  CryptoJS.a = {};
  globalThis.CryptoJS = CryptoJS;
  if (typeof exports === "object") {
    module.exports = CryptoJS;
  }

  CryptoJS.enc.Base64 = {
    st(t) {
      let e,
        s,
        i = t.words,
        h = t.sigBytes,
        r = [],
        n = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=";
      for (t.cp(), t = 0; t < h; t += 3)
        for (
          s =
            (((i[t >>> 2] >>> (24 - (t % 4) * 8)) & 255) << 16) |
            (((i[(t + 1) >>> 2] >>> (24 - ((t + 1) % 4) * 8)) & 255) << 8) |
            ((i[(t + 2) >>> 2] >>> (24 - ((t + 2) % 4) * 8)) & 255),
            e = 0;
          e < 4 && t + 0.75 * e < h;
          e++
        )
          r.push(n.charAt((s >>> (6 * (3 - e))) & 63));
      let c = n.charAt(64);
      if (c) for (; r.length % 4; ) r.push(c);
      return r.join("");
    },
  };
  {
    let t = CryptoJS,
      e = t.l,
      s = e.Base,
      h = e.wa,
      r = e.ba,
      n = t.enc.Base64,
      c = r.e({
        cfg: s.e(),
        c7(t, e) {
          return this._c(this.e4, t, e);
        },
        _: function (t, e, s) {
          (this.cfg = this.cfg.e(s)), (this.x7 = t), (this.kk = e), this.rt();
        },
        rt() {
          r.rt.call(this), this.d7();
        },
        f5(t) {
          return t && this.ap(t), this.d9();
        },
        ks: 4,
        ivSize: 4,
        e4: 1,
        ch: ((t) => (
          (t = (t) => ("string" == typeof t ? p : f)),
          (e) => ({ encrypt: (s, i, h) => t(i).encrypt(e, s, i, h) })
        ))(),
      }),
      l =
        (c.e({ be: 1 }),
        (t.mode = {}),
        (e._b = s.e({
          c7(t, e) {
            return this.e0._c(t, e);
          },
          _: function (t, e) {
            (this._c = t), (this._iv = e);
          },
        })),
        ((t.pad = {}).Pkcs7 = {
          pad(t, e) {
            let s = 4 * e,
              i = s - (t.sigBytes % s),
              r = (i << 24) | (i << 16) | (i << 8) | i,
              n = [],
              c = 0;
            for (; c < i; c += 4) n.push(r);
            t.concat(h._c(n, i));
          },
        })),
      o =
        ((e.b3 = c.e({
          cfg: c.cfg.e({ padding: l }),
          rt() {
            c.rt.call(this);
            let t,
              e = this.cfg,
              s = e.iv,
              i = e.mode;
            this.x7 == this.e4 ? (t = i.c7) : (this.m8 = 1),
              this.md && this.md.c3 == t
                ? this.md._(this, s && s.words)
                : ((this.md = t.call(i, this, s && s.words)), (this.md.c3 = t));
          },
          k2(t, e) {
            this.md.pb(t, e);
          },
          d9() {
            let t,
              e = this.cfg.padding;
            return (
              this.x7 == this.e4
                ? (e.pad(this._t, this.be), (t = this.p8(!0)))
                : ((t = this.p8(!0)), e.unpad(t)),
              t
            );
          },
          be: 4,
        })),
        s.e({
          _: function (t) {
            this.m1(t);
          },
          toString(t) {
            return (t || this.ft2).st(this);
          },
        })),
      a = {
        st(t) {
          let e,
            s = t.ct6,
            i = t.salt;
          return (
            (e = i ? h._c([1398893684, 1701076831]).concat(i).concat(s) : s),
            e.toString(n)
          );
        },
      },
      f = s.e({
        cfg: s.e({ f: a }),
        encrypt(t, e, s, i) {
          i = this.cfg.e(i);
          let h = t.c7(s, i),
            r = h.f5(e),
            n = h.cfg;
          return o._c({
            ct6: r,
            key: s,
            iv: n.iv,
            ag8: t,
            mode: n.mode,
            padding: n.padding,
            be: t.be,
            ft2: i.f,
          });
        },
      });
  }
  let ECB = CryptoJS.l._b.e();
  (ECB.e0 = ECB.e({
    pb(t, e) {
      this._c.v0(t, e);
    },
  })),
    (CryptoJS.mode.ECB = ECB);
  {
    let t = CryptoJS,
      e = t.l.b3,
      s = t.a,
      h = [],
      r = [],
      n = [],
      c = [],
      l = [],
      o = [],
      a = [],
      p = [],
      f = [],
      g = [];
    {
      let t = [];
      for (let e = 0; e < 256; e++) t[e] = e < 128 ? e << 1 : (e << 1) ^ 283;
      let e = 0,
        s = 0;
      for (let i = 0; i < 256; i++) {
        let i = s ^ (s << 1) ^ (s << 2) ^ (s << 3) ^ (s << 4);
        (i = (i >>> 8) ^ (255 & i) ^ 99), (h[e] = i), (r[i] = e);
        let d = t[e],
          _ = t[d],
          u = t[_],
          y = (257 * t[i]) ^ (16843008 * i);
        (n[e] = (y << 24) | (y >>> 8)),
          (c[e] = (y << 16) | (y >>> 16)),
          (l[e] = (y << 8) | (y >>> 24)),
          (o[e] = y),
          (y = (16843009 * u) ^ (65537 * _) ^ (257 * d) ^ (16843008 * e)),
          (a[i] = (y << 24) | (y >>> 8)),
          (p[i] = (y << 16) | (y >>> 16)),
          (f[i] = (y << 8) | (y >>> 24)),
          (g[i] = y),
          e ? ((e = d ^ t[t[t[u ^ d]]]), (s ^= t[t[s]])) : (e = s = 1);
      }
    }
    let d = [0, 1, 2, 4, 8, 16, 32, 64, 128, 27, 54],
      _ = (s.AES = e.e({
        d7() {
          if (this.nr9 && this.r5 === this.kk) return;
          let t,
            e,
            s = (this.r5 = this.kk),
            i = s.words,
            r = s.sigBytes / 4,
            n = 4 * ((this.nr9 = r + 6) + 1),
            c = (this.k4 = []),
            l = 0,
            o = 0,
            _ = (this.i1 = []);
          for (; l < n; l++)
            l < r
              ? (c[l] = i[l])
              : ((t = c[l - 1]),
                l % r
                  ? r > 6 &&
                    l % r == 4 &&
                    (t =
                      (h[t >>> 24] << 24) |
                      (h[(t >>> 16) & 255] << 16) |
                      (h[(t >>> 8) & 255] << 8) |
                      h[255 & t])
                  : ((t = (t << 8) | (t >>> 24)),
                    (t =
                      (h[t >>> 24] << 24) |
                      (h[(t >>> 16) & 255] << 16) |
                      (h[(t >>> 8) & 255] << 8) |
                      h[255 & t]),
                    (t ^= d[(l / r) | 0] << 24)),
                (c[l] = c[l - r] ^ t));
          for (; o < n; o++)
            (e = n - o),
              o % 4 ? c[e] : c[e - 4],
              (_[o] =
                o < 4 || e <= 4
                  ? t
                  : a[h[t >>> 24]] ^
                    p[h[(t >>> 16) & 255]] ^
                    f[h[(t >>> 8) & 255]] ^
                    g[h[255 & t]]);
        },
        v0(t, e) {
          this.h2(t, e, this.k4, n, c, l, o, h);
        },
        h2(t, e, s, i, h, r, n, c) {
          let l = this.nr9,
            o = t[e] ^ s[0],
            a = t[e + 1] ^ s[1],
            p = t[e + 2] ^ s[2],
            f = t[e + 3] ^ s[3],
            g = 4;
          for (let t = 1; t < l; t++) {
            let t =
                i[o >>> 24] ^
                h[(a >>> 16) & 255] ^
                r[(p >>> 8) & 255] ^
                n[255 & f] ^
                s[g++],
              e =
                i[a >>> 24] ^
                h[(p >>> 16) & 255] ^
                r[(f >>> 8) & 255] ^
                n[255 & o] ^
                s[g++],
              c =
                i[p >>> 24] ^
                h[(f >>> 16) & 255] ^
                r[(o >>> 8) & 255] ^
                n[255 & a] ^
                s[g++],
              l =
                i[f >>> 24] ^
                h[(o >>> 16) & 255] ^
                r[(a >>> 8) & 255] ^
                n[255 & p] ^
                s[g++];
            (o = t), (a = e), (p = c), (f = l);
          }
          let d =
              ((c[o >>> 24] << 24) |
                (c[(a >>> 16) & 255] << 16) |
                (c[(p >>> 8) & 255] << 8) |
                c[255 & f]) ^
              s[g++],
            _ =
              ((c[a >>> 24] << 24) |
                (c[(p >>> 16) & 255] << 16) |
                (c[(f >>> 8) & 255] << 8) |
                c[255 & o]) ^
              s[g++],
            u =
              ((c[p >>> 24] << 24) |
                (c[(f >>> 16) & 255] << 16) |
                (c[(o >>> 8) & 255] << 8) |
                c[255 & a]) ^
              s[g++],
            y =
              ((c[f >>> 24] << 24) |
                (c[(o >>> 16) & 255] << 16) |
                (c[(a >>> 8) & 255] << 8) |
                c[255 & p]) ^
              s[g++];
          (t[e] = d), (t[e + 1] = _), (t[e + 2] = u), (t[e + 3] = y);
        },
        ks: 8,
      }));
    t.AES = e.ch(_);
  }
}
