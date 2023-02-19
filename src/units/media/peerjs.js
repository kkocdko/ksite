(() => {
  var __create = Object.create;
  var __defProp = Object.defineProperty;
  var __getOwnPropDesc = Object.getOwnPropertyDescriptor;
  var __getOwnPropNames = Object.getOwnPropertyNames;
  var __getProtoOf = Object.getPrototypeOf;
  var __hasOwnProp = Object.prototype.hasOwnProperty;
  var __defNormalProp = (obj, key, value) =>
    key in obj
      ? __defProp(obj, key, {
          enumerable: true,
          configurable: true,
          writable: true,
          value,
        })
      : (obj[key] = value);
  var __commonJS = (cb, mod) =>
    function __require() {
      return (
        mod ||
          (0, cb[__getOwnPropNames(cb)[0]])(
            (mod = { exports: {} }).exports,
            mod
          ),
        mod.exports
      );
    };
  var __export = (target, all) => {
    for (var name in all)
      __defProp(target, name, { get: all[name], enumerable: true });
  };
  var __copyProps = (to, from, except, desc) => {
    if ((from && typeof from === "object") || typeof from === "function") {
      for (let key of __getOwnPropNames(from))
        if (!__hasOwnProp.call(to, key) && key !== except)
          __defProp(to, key, {
            get: () => from[key],
            enumerable:
              !(desc = __getOwnPropDesc(from, key)) || desc.enumerable,
          });
    }
    return to;
  };
  var __toESM = (mod, isNodeMode, target) => (
    (target = mod != null ? __create(__getProtoOf(mod)) : {}),
    __copyProps(
      // If the importer is in node compatibility mode or this is not an ESM
      // file that has been converted to a CommonJS file using a Babel-
      // compatible transform (i.e. "__esModule" has not been set), then set
      // "default" to the CommonJS "module.exports" for node compatibility.
      isNodeMode || !mod || !mod.__esModule
        ? __defProp(target, "default", { value: mod, enumerable: true })
        : target,
      mod
    )
  );
  var __publicField = (obj, key, value) => {
    __defNormalProp(obj, typeof key !== "symbol" ? key + "" : key, value);
    return value;
  };

  // node_modules/peerjs-js-binarypack/lib/bufferbuilder.js
  var require_bufferbuilder = __commonJS({
    "node_modules/peerjs-js-binarypack/lib/bufferbuilder.js"(exports, module) {
      var binaryFeatures = {};
      binaryFeatures.useBlobBuilder = (function () {
        try {
          new Blob([]);
          return false;
        } catch (e) {
          return true;
        }
      })();
      binaryFeatures.useArrayBufferView =
        !binaryFeatures.useBlobBuilder &&
        (function () {
          try {
            return new Blob([new Uint8Array([])]).size === 0;
          } catch (e) {
            return true;
          }
        })();
      module.exports.binaryFeatures = binaryFeatures;
      var BlobBuilder = module.exports.BlobBuilder;
      if (typeof window !== "undefined") {
        BlobBuilder = module.exports.BlobBuilder =
          window.WebKitBlobBuilder ||
          window.MozBlobBuilder ||
          window.MSBlobBuilder ||
          window.BlobBuilder;
      }
      function BufferBuilder() {
        this._pieces = [];
        this._parts = [];
      }
      BufferBuilder.prototype.append = function (data) {
        if (typeof data === "number") {
          this._pieces.push(data);
        } else {
          this.flush();
          this._parts.push(data);
        }
      };
      BufferBuilder.prototype.flush = function () {
        if (this._pieces.length > 0) {
          var buf = new Uint8Array(this._pieces);
          if (!binaryFeatures.useArrayBufferView) {
            buf = buf.buffer;
          }
          this._parts.push(buf);
          this._pieces = [];
        }
      };
      BufferBuilder.prototype.getBuffer = function () {
        this.flush();
        if (binaryFeatures.useBlobBuilder) {
          var builder = new BlobBuilder();
          for (var i = 0, ii = this._parts.length; i < ii; i++) {
            builder.append(this._parts[i]);
          }
          return builder.getBlob();
        } else {
          return new Blob(this._parts);
        }
      };
      module.exports.BufferBuilder = BufferBuilder;
    },
  });

  // node_modules/peerjs-js-binarypack/lib/binarypack.js
  var require_binarypack = __commonJS({
    "node_modules/peerjs-js-binarypack/lib/binarypack.js"(exports, module) {
      var BufferBuilder = require_bufferbuilder().BufferBuilder;
      var binaryFeatures = require_bufferbuilder().binaryFeatures;
      var BinaryPack2 = {
        unpack: function (data) {
          var unpacker = new Unpacker(data);
          return unpacker.unpack();
        },
        pack: function (data) {
          var packer = new Packer();
          packer.pack(data);
          var buffer = packer.getBuffer();
          return buffer;
        },
      };
      module.exports = BinaryPack2;
      function Unpacker(data) {
        this.index = 0;
        this.dataBuffer = data;
        this.dataView = new Uint8Array(this.dataBuffer);
        this.length = this.dataBuffer.byteLength;
      }
      Unpacker.prototype.unpack = function () {
        var type = this.unpack_uint8();
        if (type < 128) {
          return type;
        } else if ((type ^ 224) < 32) {
          return (type ^ 224) - 32;
        }
        var size;
        if ((size = type ^ 160) <= 15) {
          return this.unpack_raw(size);
        } else if ((size = type ^ 176) <= 15) {
          return this.unpack_string(size);
        } else if ((size = type ^ 144) <= 15) {
          return this.unpack_array(size);
        } else if ((size = type ^ 128) <= 15) {
          return this.unpack_map(size);
        }
        switch (type) {
          case 192:
            return null;
          case 193:
            return void 0;
          case 194:
            return false;
          case 195:
            return true;
          case 202:
            return this.unpack_float();
          case 203:
            return this.unpack_double();
          case 204:
            return this.unpack_uint8();
          case 205:
            return this.unpack_uint16();
          case 206:
            return this.unpack_uint32();
          case 207:
            return this.unpack_uint64();
          case 208:
            return this.unpack_int8();
          case 209:
            return this.unpack_int16();
          case 210:
            return this.unpack_int32();
          case 211:
            return this.unpack_int64();
          case 212:
            return void 0;
          case 213:
            return void 0;
          case 214:
            return void 0;
          case 215:
            return void 0;
          case 216:
            size = this.unpack_uint16();
            return this.unpack_string(size);
          case 217:
            size = this.unpack_uint32();
            return this.unpack_string(size);
          case 218:
            size = this.unpack_uint16();
            return this.unpack_raw(size);
          case 219:
            size = this.unpack_uint32();
            return this.unpack_raw(size);
          case 220:
            size = this.unpack_uint16();
            return this.unpack_array(size);
          case 221:
            size = this.unpack_uint32();
            return this.unpack_array(size);
          case 222:
            size = this.unpack_uint16();
            return this.unpack_map(size);
          case 223:
            size = this.unpack_uint32();
            return this.unpack_map(size);
        }
      };
      Unpacker.prototype.unpack_uint8 = function () {
        var byte = this.dataView[this.index] & 255;
        this.index++;
        return byte;
      };
      Unpacker.prototype.unpack_uint16 = function () {
        var bytes = this.read(2);
        var uint16 = (bytes[0] & 255) * 256 + (bytes[1] & 255);
        this.index += 2;
        return uint16;
      };
      Unpacker.prototype.unpack_uint32 = function () {
        var bytes = this.read(4);
        var uint32 =
          ((bytes[0] * 256 + bytes[1]) * 256 + bytes[2]) * 256 + bytes[3];
        this.index += 4;
        return uint32;
      };
      Unpacker.prototype.unpack_uint64 = function () {
        var bytes = this.read(8);
        var uint64 =
          ((((((bytes[0] * 256 + bytes[1]) * 256 + bytes[2]) * 256 + bytes[3]) *
            256 +
            bytes[4]) *
            256 +
            bytes[5]) *
            256 +
            bytes[6]) *
            256 +
          bytes[7];
        this.index += 8;
        return uint64;
      };
      Unpacker.prototype.unpack_int8 = function () {
        var uint8 = this.unpack_uint8();
        return uint8 < 128 ? uint8 : uint8 - (1 << 8);
      };
      Unpacker.prototype.unpack_int16 = function () {
        var uint16 = this.unpack_uint16();
        return uint16 < 32768 ? uint16 : uint16 - (1 << 16);
      };
      Unpacker.prototype.unpack_int32 = function () {
        var uint32 = this.unpack_uint32();
        return uint32 < Math.pow(2, 31) ? uint32 : uint32 - Math.pow(2, 32);
      };
      Unpacker.prototype.unpack_int64 = function () {
        var uint64 = this.unpack_uint64();
        return uint64 < Math.pow(2, 63) ? uint64 : uint64 - Math.pow(2, 64);
      };
      Unpacker.prototype.unpack_raw = function (size) {
        if (this.length < this.index + size) {
          throw new Error(
            "BinaryPackFailure: index is out of range " +
              this.index +
              " " +
              size +
              " " +
              this.length
          );
        }
        var buf = this.dataBuffer.slice(this.index, this.index + size);
        this.index += size;
        return buf;
      };
      Unpacker.prototype.unpack_string = function (size) {
        var bytes = this.read(size);
        var i = 0;
        var str = "";
        var c;
        var code;
        while (i < size) {
          c = bytes[i];
          if (c < 128) {
            str += String.fromCharCode(c);
            i++;
          } else if ((c ^ 192) < 32) {
            code = ((c ^ 192) << 6) | (bytes[i + 1] & 63);
            str += String.fromCharCode(code);
            i += 2;
          } else {
            code =
              ((c & 15) << 12) |
              ((bytes[i + 1] & 63) << 6) |
              (bytes[i + 2] & 63);
            str += String.fromCharCode(code);
            i += 3;
          }
        }
        this.index += size;
        return str;
      };
      Unpacker.prototype.unpack_array = function (size) {
        var objects = new Array(size);
        for (var i = 0; i < size; i++) {
          objects[i] = this.unpack();
        }
        return objects;
      };
      Unpacker.prototype.unpack_map = function (size) {
        var map = {};
        for (var i = 0; i < size; i++) {
          var key = this.unpack();
          var value = this.unpack();
          map[key] = value;
        }
        return map;
      };
      Unpacker.prototype.unpack_float = function () {
        var uint32 = this.unpack_uint32();
        var sign = uint32 >> 31;
        var exp = ((uint32 >> 23) & 255) - 127;
        var fraction = (uint32 & 8388607) | 8388608;
        return (sign === 0 ? 1 : -1) * fraction * Math.pow(2, exp - 23);
      };
      Unpacker.prototype.unpack_double = function () {
        var h32 = this.unpack_uint32();
        var l32 = this.unpack_uint32();
        var sign = h32 >> 31;
        var exp = ((h32 >> 20) & 2047) - 1023;
        var hfrac = (h32 & 1048575) | 1048576;
        var frac = hfrac * Math.pow(2, exp - 20) + l32 * Math.pow(2, exp - 52);
        return (sign === 0 ? 1 : -1) * frac;
      };
      Unpacker.prototype.read = function (length) {
        var j = this.index;
        if (j + length <= this.length) {
          return this.dataView.subarray(j, j + length);
        } else {
          throw new Error("BinaryPackFailure: read index out of range");
        }
      };
      function Packer() {
        this.bufferBuilder = new BufferBuilder();
      }
      Packer.prototype.getBuffer = function () {
        return this.bufferBuilder.getBuffer();
      };
      Packer.prototype.pack = function (value) {
        var type = typeof value;
        if (type === "string") {
          this.pack_string(value);
        } else if (type === "number") {
          if (Math.floor(value) === value) {
            this.pack_integer(value);
          } else {
            this.pack_double(value);
          }
        } else if (type === "boolean") {
          if (value === true) {
            this.bufferBuilder.append(195);
          } else if (value === false) {
            this.bufferBuilder.append(194);
          }
        } else if (type === "undefined") {
          this.bufferBuilder.append(192);
        } else if (type === "object") {
          if (value === null) {
            this.bufferBuilder.append(192);
          } else {
            var constructor = value.constructor;
            if (constructor == Array) {
              this.pack_array(value);
            } else if (
              constructor == Blob ||
              constructor == File ||
              value instanceof Blob ||
              value instanceof File
            ) {
              this.pack_bin(value);
            } else if (constructor == ArrayBuffer) {
              if (binaryFeatures.useArrayBufferView) {
                this.pack_bin(new Uint8Array(value));
              } else {
                this.pack_bin(value);
              }
            } else if ("BYTES_PER_ELEMENT" in value) {
              if (binaryFeatures.useArrayBufferView) {
                this.pack_bin(new Uint8Array(value.buffer));
              } else {
                this.pack_bin(value.buffer);
              }
            } else if (
              constructor == Object ||
              constructor.toString().startsWith("class")
            ) {
              this.pack_object(value);
            } else if (constructor == Date) {
              this.pack_string(value.toString());
            } else if (typeof value.toBinaryPack === "function") {
              this.bufferBuilder.append(value.toBinaryPack());
            } else {
              throw new Error(
                'Type "' + constructor.toString() + '" not yet supported'
              );
            }
          }
        } else {
          throw new Error('Type "' + type + '" not yet supported');
        }
        this.bufferBuilder.flush();
      };
      Packer.prototype.pack_bin = function (blob) {
        var length = blob.length || blob.byteLength || blob.size;
        if (length <= 15) {
          this.pack_uint8(160 + length);
        } else if (length <= 65535) {
          this.bufferBuilder.append(218);
          this.pack_uint16(length);
        } else if (length <= 4294967295) {
          this.bufferBuilder.append(219);
          this.pack_uint32(length);
        } else {
          throw new Error("Invalid length");
        }
        this.bufferBuilder.append(blob);
      };
      Packer.prototype.pack_string = function (str) {
        var length = utf8Length(str);
        if (length <= 15) {
          this.pack_uint8(176 + length);
        } else if (length <= 65535) {
          this.bufferBuilder.append(216);
          this.pack_uint16(length);
        } else if (length <= 4294967295) {
          this.bufferBuilder.append(217);
          this.pack_uint32(length);
        } else {
          throw new Error("Invalid length");
        }
        this.bufferBuilder.append(str);
      };
      Packer.prototype.pack_array = function (ary) {
        var length = ary.length;
        if (length <= 15) {
          this.pack_uint8(144 + length);
        } else if (length <= 65535) {
          this.bufferBuilder.append(220);
          this.pack_uint16(length);
        } else if (length <= 4294967295) {
          this.bufferBuilder.append(221);
          this.pack_uint32(length);
        } else {
          throw new Error("Invalid length");
        }
        for (var i = 0; i < length; i++) {
          this.pack(ary[i]);
        }
      };
      Packer.prototype.pack_integer = function (num) {
        if (num >= -32 && num <= 127) {
          this.bufferBuilder.append(num & 255);
        } else if (num >= 0 && num <= 255) {
          this.bufferBuilder.append(204);
          this.pack_uint8(num);
        } else if (num >= -128 && num <= 127) {
          this.bufferBuilder.append(208);
          this.pack_int8(num);
        } else if (num >= 0 && num <= 65535) {
          this.bufferBuilder.append(205);
          this.pack_uint16(num);
        } else if (num >= -32768 && num <= 32767) {
          this.bufferBuilder.append(209);
          this.pack_int16(num);
        } else if (num >= 0 && num <= 4294967295) {
          this.bufferBuilder.append(206);
          this.pack_uint32(num);
        } else if (num >= -2147483648 && num <= 2147483647) {
          this.bufferBuilder.append(210);
          this.pack_int32(num);
        } else if (num >= -9223372036854776e3 && num <= 9223372036854776e3) {
          this.bufferBuilder.append(211);
          this.pack_int64(num);
        } else if (num >= 0 && num <= 18446744073709552e3) {
          this.bufferBuilder.append(207);
          this.pack_uint64(num);
        } else {
          throw new Error("Invalid integer");
        }
      };
      Packer.prototype.pack_double = function (num) {
        var sign = 0;
        if (num < 0) {
          sign = 1;
          num = -num;
        }
        var exp = Math.floor(Math.log(num) / Math.LN2);
        var frac0 = num / Math.pow(2, exp) - 1;
        var frac1 = Math.floor(frac0 * Math.pow(2, 52));
        var b32 = Math.pow(2, 32);
        var h32 =
          (sign << 31) | ((exp + 1023) << 20) | ((frac1 / b32) & 1048575);
        var l32 = frac1 % b32;
        this.bufferBuilder.append(203);
        this.pack_int32(h32);
        this.pack_int32(l32);
      };
      Packer.prototype.pack_object = function (obj) {
        var keys = Object.keys(obj);
        var length = keys.length;
        if (length <= 15) {
          this.pack_uint8(128 + length);
        } else if (length <= 65535) {
          this.bufferBuilder.append(222);
          this.pack_uint16(length);
        } else if (length <= 4294967295) {
          this.bufferBuilder.append(223);
          this.pack_uint32(length);
        } else {
          throw new Error("Invalid length");
        }
        for (var prop in obj) {
          if (obj.hasOwnProperty(prop)) {
            this.pack(prop);
            this.pack(obj[prop]);
          }
        }
      };
      Packer.prototype.pack_uint8 = function (num) {
        this.bufferBuilder.append(num);
      };
      Packer.prototype.pack_uint16 = function (num) {
        this.bufferBuilder.append(num >> 8);
        this.bufferBuilder.append(num & 255);
      };
      Packer.prototype.pack_uint32 = function (num) {
        var n = num & 4294967295;
        this.bufferBuilder.append((n & 4278190080) >>> 24);
        this.bufferBuilder.append((n & 16711680) >>> 16);
        this.bufferBuilder.append((n & 65280) >>> 8);
        this.bufferBuilder.append(n & 255);
      };
      Packer.prototype.pack_uint64 = function (num) {
        var high = num / Math.pow(2, 32);
        var low = num % Math.pow(2, 32);
        this.bufferBuilder.append((high & 4278190080) >>> 24);
        this.bufferBuilder.append((high & 16711680) >>> 16);
        this.bufferBuilder.append((high & 65280) >>> 8);
        this.bufferBuilder.append(high & 255);
        this.bufferBuilder.append((low & 4278190080) >>> 24);
        this.bufferBuilder.append((low & 16711680) >>> 16);
        this.bufferBuilder.append((low & 65280) >>> 8);
        this.bufferBuilder.append(low & 255);
      };
      Packer.prototype.pack_int8 = function (num) {
        this.bufferBuilder.append(num & 255);
      };
      Packer.prototype.pack_int16 = function (num) {
        this.bufferBuilder.append((num & 65280) >> 8);
        this.bufferBuilder.append(num & 255);
      };
      Packer.prototype.pack_int32 = function (num) {
        this.bufferBuilder.append((num >>> 24) & 255);
        this.bufferBuilder.append((num & 16711680) >>> 16);
        this.bufferBuilder.append((num & 65280) >>> 8);
        this.bufferBuilder.append(num & 255);
      };
      Packer.prototype.pack_int64 = function (num) {
        var high = Math.floor(num / Math.pow(2, 32));
        var low = num % Math.pow(2, 32);
        this.bufferBuilder.append((high & 4278190080) >>> 24);
        this.bufferBuilder.append((high & 16711680) >>> 16);
        this.bufferBuilder.append((high & 65280) >>> 8);
        this.bufferBuilder.append(high & 255);
        this.bufferBuilder.append((low & 4278190080) >>> 24);
        this.bufferBuilder.append((low & 16711680) >>> 16);
        this.bufferBuilder.append((low & 65280) >>> 8);
        this.bufferBuilder.append(low & 255);
      };
      function _utf8Replace(m) {
        var code = m.charCodeAt(0);
        if (code <= 2047) return "00";
        if (code <= 65535) return "000";
        if (code <= 2097151) return "0000";
        if (code <= 67108863) return "00000";
        return "000000";
      }
      function utf8Length(str) {
        if (str.length > 600) {
          return new Blob([str]).size;
        } else {
          return str.replace(/[^\u0000-\u007F]/g, _utf8Replace).length;
        }
      }
    },
  });

  // node_modules/sdp/sdp.js
  var require_sdp = __commonJS({
    "node_modules/sdp/sdp.js"(exports, module) {
      "use strict";
      var SDPUtils2 = {};
      SDPUtils2.generateIdentifier = function () {
        return Math.random().toString(36).substring(2, 12);
      };
      SDPUtils2.localCName = SDPUtils2.generateIdentifier();
      SDPUtils2.splitLines = function (blob) {
        return blob
          .trim()
          .split("\n")
          .map((line) => line.trim());
      };
      SDPUtils2.splitSections = function (blob) {
        const parts = blob.split("\nm=");
        return parts.map(
          (part, index) => (index > 0 ? "m=" + part : part).trim() + "\r\n"
        );
      };
      SDPUtils2.getDescription = function (blob) {
        const sections = SDPUtils2.splitSections(blob);
        return sections && sections[0];
      };
      SDPUtils2.getMediaSections = function (blob) {
        const sections = SDPUtils2.splitSections(blob);
        sections.shift();
        return sections;
      };
      SDPUtils2.matchPrefix = function (blob, prefix) {
        return SDPUtils2.splitLines(blob).filter(
          (line) => line.indexOf(prefix) === 0
        );
      };
      SDPUtils2.parseCandidate = function (line) {
        let parts;
        if (line.indexOf("a=candidate:") === 0) {
          parts = line.substring(12).split(" ");
        } else {
          parts = line.substring(10).split(" ");
        }
        const candidate = {
          foundation: parts[0],
          component: { 1: "rtp", 2: "rtcp" }[parts[1]] || parts[1],
          protocol: parts[2].toLowerCase(),
          priority: parseInt(parts[3], 10),
          ip: parts[4],
          address: parts[4],
          // address is an alias for ip.
          port: parseInt(parts[5], 10),
          // skip parts[6] == 'typ'
          type: parts[7],
        };
        for (let i = 8; i < parts.length; i += 2) {
          switch (parts[i]) {
            case "raddr":
              candidate.relatedAddress = parts[i + 1];
              break;
            case "rport":
              candidate.relatedPort = parseInt(parts[i + 1], 10);
              break;
            case "tcptype":
              candidate.tcpType = parts[i + 1];
              break;
            case "ufrag":
              candidate.ufrag = parts[i + 1];
              candidate.usernameFragment = parts[i + 1];
              break;
            default:
              if (candidate[parts[i]] === void 0) {
                candidate[parts[i]] = parts[i + 1];
              }
              break;
          }
        }
        return candidate;
      };
      SDPUtils2.writeCandidate = function (candidate) {
        const sdp2 = [];
        sdp2.push(candidate.foundation);
        const component = candidate.component;
        if (component === "rtp") {
          sdp2.push(1);
        } else if (component === "rtcp") {
          sdp2.push(2);
        } else {
          sdp2.push(component);
        }
        sdp2.push(candidate.protocol.toUpperCase());
        sdp2.push(candidate.priority);
        sdp2.push(candidate.address || candidate.ip);
        sdp2.push(candidate.port);
        const type = candidate.type;
        sdp2.push("typ");
        sdp2.push(type);
        if (
          type !== "host" &&
          candidate.relatedAddress &&
          candidate.relatedPort
        ) {
          sdp2.push("raddr");
          sdp2.push(candidate.relatedAddress);
          sdp2.push("rport");
          sdp2.push(candidate.relatedPort);
        }
        if (candidate.tcpType && candidate.protocol.toLowerCase() === "tcp") {
          sdp2.push("tcptype");
          sdp2.push(candidate.tcpType);
        }
        if (candidate.usernameFragment || candidate.ufrag) {
          sdp2.push("ufrag");
          sdp2.push(candidate.usernameFragment || candidate.ufrag);
        }
        return "candidate:" + sdp2.join(" ");
      };
      SDPUtils2.parseIceOptions = function (line) {
        return line.substring(14).split(" ");
      };
      SDPUtils2.parseRtpMap = function (line) {
        let parts = line.substring(9).split(" ");
        const parsed = {
          payloadType: parseInt(parts.shift(), 10),
          // was: id
        };
        parts = parts[0].split("/");
        parsed.name = parts[0];
        parsed.clockRate = parseInt(parts[1], 10);
        parsed.channels = parts.length === 3 ? parseInt(parts[2], 10) : 1;
        parsed.numChannels = parsed.channels;
        return parsed;
      };
      SDPUtils2.writeRtpMap = function (codec) {
        let pt = codec.payloadType;
        if (codec.preferredPayloadType !== void 0) {
          pt = codec.preferredPayloadType;
        }
        const channels = codec.channels || codec.numChannels || 1;
        return (
          "a=rtpmap:" +
          pt +
          " " +
          codec.name +
          "/" +
          codec.clockRate +
          (channels !== 1 ? "/" + channels : "") +
          "\r\n"
        );
      };
      SDPUtils2.parseExtmap = function (line) {
        const parts = line.substring(9).split(" ");
        return {
          id: parseInt(parts[0], 10),
          direction:
            parts[0].indexOf("/") > 0 ? parts[0].split("/")[1] : "sendrecv",
          uri: parts[1],
          attributes: parts.slice(2).join(" "),
        };
      };
      SDPUtils2.writeExtmap = function (headerExtension) {
        return (
          "a=extmap:" +
          (headerExtension.id || headerExtension.preferredId) +
          (headerExtension.direction && headerExtension.direction !== "sendrecv"
            ? "/" + headerExtension.direction
            : "") +
          " " +
          headerExtension.uri +
          (headerExtension.attributes ? " " + headerExtension.attributes : "") +
          "\r\n"
        );
      };
      SDPUtils2.parseFmtp = function (line) {
        const parsed = {};
        let kv;
        const parts = line.substring(line.indexOf(" ") + 1).split(";");
        for (let j = 0; j < parts.length; j++) {
          kv = parts[j].trim().split("=");
          parsed[kv[0].trim()] = kv[1];
        }
        return parsed;
      };
      SDPUtils2.writeFmtp = function (codec) {
        let line = "";
        let pt = codec.payloadType;
        if (codec.preferredPayloadType !== void 0) {
          pt = codec.preferredPayloadType;
        }
        if (codec.parameters && Object.keys(codec.parameters).length) {
          const params = [];
          Object.keys(codec.parameters).forEach((param) => {
            if (codec.parameters[param] !== void 0) {
              params.push(param + "=" + codec.parameters[param]);
            } else {
              params.push(param);
            }
          });
          line += "a=fmtp:" + pt + " " + params.join(";") + "\r\n";
        }
        return line;
      };
      SDPUtils2.parseRtcpFb = function (line) {
        const parts = line.substring(line.indexOf(" ") + 1).split(" ");
        return {
          type: parts.shift(),
          parameter: parts.join(" "),
        };
      };
      SDPUtils2.writeRtcpFb = function (codec) {
        let lines = "";
        let pt = codec.payloadType;
        if (codec.preferredPayloadType !== void 0) {
          pt = codec.preferredPayloadType;
        }
        if (codec.rtcpFeedback && codec.rtcpFeedback.length) {
          codec.rtcpFeedback.forEach((fb) => {
            lines +=
              "a=rtcp-fb:" +
              pt +
              " " +
              fb.type +
              (fb.parameter && fb.parameter.length ? " " + fb.parameter : "") +
              "\r\n";
          });
        }
        return lines;
      };
      SDPUtils2.parseSsrcMedia = function (line) {
        const sp = line.indexOf(" ");
        const parts = {
          ssrc: parseInt(line.substring(7, sp), 10),
        };
        const colon = line.indexOf(":", sp);
        if (colon > -1) {
          parts.attribute = line.substring(sp + 1, colon);
          parts.value = line.substring(colon + 1);
        } else {
          parts.attribute = line.substring(sp + 1);
        }
        return parts;
      };
      SDPUtils2.parseSsrcGroup = function (line) {
        const parts = line.substring(13).split(" ");
        return {
          semantics: parts.shift(),
          ssrcs: parts.map((ssrc) => parseInt(ssrc, 10)),
        };
      };
      SDPUtils2.getMid = function (mediaSection) {
        const mid = SDPUtils2.matchPrefix(mediaSection, "a=mid:")[0];
        if (mid) {
          return mid.substring(6);
        }
      };
      SDPUtils2.parseFingerprint = function (line) {
        const parts = line.substring(14).split(" ");
        return {
          algorithm: parts[0].toLowerCase(),
          // algorithm is case-sensitive in Edge.
          value: parts[1].toUpperCase(),
          // the definition is upper-case in RFC 4572.
        };
      };
      SDPUtils2.getDtlsParameters = function (mediaSection, sessionpart) {
        const lines = SDPUtils2.matchPrefix(
          mediaSection + sessionpart,
          "a=fingerprint:"
        );
        return {
          role: "auto",
          fingerprints: lines.map(SDPUtils2.parseFingerprint),
        };
      };
      SDPUtils2.writeDtlsParameters = function (params, setupType) {
        let sdp2 = "a=setup:" + setupType + "\r\n";
        params.fingerprints.forEach((fp) => {
          sdp2 += "a=fingerprint:" + fp.algorithm + " " + fp.value + "\r\n";
        });
        return sdp2;
      };
      SDPUtils2.parseCryptoLine = function (line) {
        const parts = line.substring(9).split(" ");
        return {
          tag: parseInt(parts[0], 10),
          cryptoSuite: parts[1],
          keyParams: parts[2],
          sessionParams: parts.slice(3),
        };
      };
      SDPUtils2.writeCryptoLine = function (parameters) {
        return (
          "a=crypto:" +
          parameters.tag +
          " " +
          parameters.cryptoSuite +
          " " +
          (typeof parameters.keyParams === "object"
            ? SDPUtils2.writeCryptoKeyParams(parameters.keyParams)
            : parameters.keyParams) +
          (parameters.sessionParams
            ? " " + parameters.sessionParams.join(" ")
            : "") +
          "\r\n"
        );
      };
      SDPUtils2.parseCryptoKeyParams = function (keyParams) {
        if (keyParams.indexOf("inline:") !== 0) {
          return null;
        }
        const parts = keyParams.substring(7).split("|");
        return {
          keyMethod: "inline",
          keySalt: parts[0],
          lifeTime: parts[1],
          mkiValue: parts[2] ? parts[2].split(":")[0] : void 0,
          mkiLength: parts[2] ? parts[2].split(":")[1] : void 0,
        };
      };
      SDPUtils2.writeCryptoKeyParams = function (keyParams) {
        return (
          keyParams.keyMethod +
          ":" +
          keyParams.keySalt +
          (keyParams.lifeTime ? "|" + keyParams.lifeTime : "") +
          (keyParams.mkiValue && keyParams.mkiLength
            ? "|" + keyParams.mkiValue + ":" + keyParams.mkiLength
            : "")
        );
      };
      SDPUtils2.getCryptoParameters = function (mediaSection, sessionpart) {
        const lines = SDPUtils2.matchPrefix(
          mediaSection + sessionpart,
          "a=crypto:"
        );
        return lines.map(SDPUtils2.parseCryptoLine);
      };
      SDPUtils2.getIceParameters = function (mediaSection, sessionpart) {
        const ufrag = SDPUtils2.matchPrefix(
          mediaSection + sessionpart,
          "a=ice-ufrag:"
        )[0];
        const pwd = SDPUtils2.matchPrefix(
          mediaSection + sessionpart,
          "a=ice-pwd:"
        )[0];
        if (!(ufrag && pwd)) {
          return null;
        }
        return {
          usernameFragment: ufrag.substring(12),
          password: pwd.substring(10),
        };
      };
      SDPUtils2.writeIceParameters = function (params) {
        let sdp2 =
          "a=ice-ufrag:" +
          params.usernameFragment +
          "\r\na=ice-pwd:" +
          params.password +
          "\r\n";
        if (params.iceLite) {
          sdp2 += "a=ice-lite\r\n";
        }
        return sdp2;
      };
      SDPUtils2.parseRtpParameters = function (mediaSection) {
        const description = {
          codecs: [],
          headerExtensions: [],
          fecMechanisms: [],
          rtcp: [],
        };
        const lines = SDPUtils2.splitLines(mediaSection);
        const mline = lines[0].split(" ");
        for (let i = 3; i < mline.length; i++) {
          const pt = mline[i];
          const rtpmapline = SDPUtils2.matchPrefix(
            mediaSection,
            "a=rtpmap:" + pt + " "
          )[0];
          if (rtpmapline) {
            const codec = SDPUtils2.parseRtpMap(rtpmapline);
            const fmtps = SDPUtils2.matchPrefix(
              mediaSection,
              "a=fmtp:" + pt + " "
            );
            codec.parameters = fmtps.length
              ? SDPUtils2.parseFmtp(fmtps[0])
              : {};
            codec.rtcpFeedback = SDPUtils2.matchPrefix(
              mediaSection,
              "a=rtcp-fb:" + pt + " "
            ).map(SDPUtils2.parseRtcpFb);
            description.codecs.push(codec);
            switch (codec.name.toUpperCase()) {
              case "RED":
              case "ULPFEC":
                description.fecMechanisms.push(codec.name.toUpperCase());
                break;
              default:
                break;
            }
          }
        }
        SDPUtils2.matchPrefix(mediaSection, "a=extmap:").forEach((line) => {
          description.headerExtensions.push(SDPUtils2.parseExtmap(line));
        });
        const wildcardRtcpFb = SDPUtils2.matchPrefix(
          mediaSection,
          "a=rtcp-fb:* "
        ).map(SDPUtils2.parseRtcpFb);
        description.codecs.forEach((codec) => {
          wildcardRtcpFb.forEach((fb) => {
            const duplicate = codec.rtcpFeedback.find((existingFeedback) => {
              return (
                existingFeedback.type === fb.type &&
                existingFeedback.parameter === fb.parameter
              );
            });
            if (!duplicate) {
              codec.rtcpFeedback.push(fb);
            }
          });
        });
        return description;
      };
      SDPUtils2.writeRtpDescription = function (kind, caps) {
        let sdp2 = "";
        sdp2 += "m=" + kind + " ";
        sdp2 += caps.codecs.length > 0 ? "9" : "0";
        sdp2 += " UDP/TLS/RTP/SAVPF ";
        sdp2 +=
          caps.codecs
            .map((codec) => {
              if (codec.preferredPayloadType !== void 0) {
                return codec.preferredPayloadType;
              }
              return codec.payloadType;
            })
            .join(" ") + "\r\n";
        sdp2 += "c=IN IP4 0.0.0.0\r\n";
        sdp2 += "a=rtcp:9 IN IP4 0.0.0.0\r\n";
        caps.codecs.forEach((codec) => {
          sdp2 += SDPUtils2.writeRtpMap(codec);
          sdp2 += SDPUtils2.writeFmtp(codec);
          sdp2 += SDPUtils2.writeRtcpFb(codec);
        });
        let maxptime = 0;
        caps.codecs.forEach((codec) => {
          if (codec.maxptime > maxptime) {
            maxptime = codec.maxptime;
          }
        });
        if (maxptime > 0) {
          sdp2 += "a=maxptime:" + maxptime + "\r\n";
        }
        if (caps.headerExtensions) {
          caps.headerExtensions.forEach((extension) => {
            sdp2 += SDPUtils2.writeExtmap(extension);
          });
        }
        return sdp2;
      };
      SDPUtils2.parseRtpEncodingParameters = function (mediaSection) {
        const encodingParameters = [];
        const description = SDPUtils2.parseRtpParameters(mediaSection);
        const hasRed = description.fecMechanisms.indexOf("RED") !== -1;
        const hasUlpfec = description.fecMechanisms.indexOf("ULPFEC") !== -1;
        const ssrcs = SDPUtils2.matchPrefix(mediaSection, "a=ssrc:")
          .map((line) => SDPUtils2.parseSsrcMedia(line))
          .filter((parts) => parts.attribute === "cname");
        const primarySsrc = ssrcs.length > 0 && ssrcs[0].ssrc;
        let secondarySsrc;
        const flows = SDPUtils2.matchPrefix(
          mediaSection,
          "a=ssrc-group:FID"
        ).map((line) => {
          const parts = line.substring(17).split(" ");
          return parts.map((part) => parseInt(part, 10));
        });
        if (
          flows.length > 0 &&
          flows[0].length > 1 &&
          flows[0][0] === primarySsrc
        ) {
          secondarySsrc = flows[0][1];
        }
        description.codecs.forEach((codec) => {
          if (codec.name.toUpperCase() === "RTX" && codec.parameters.apt) {
            let encParam = {
              ssrc: primarySsrc,
              codecPayloadType: parseInt(codec.parameters.apt, 10),
            };
            if (primarySsrc && secondarySsrc) {
              encParam.rtx = { ssrc: secondarySsrc };
            }
            encodingParameters.push(encParam);
            if (hasRed) {
              encParam = JSON.parse(JSON.stringify(encParam));
              encParam.fec = {
                ssrc: primarySsrc,
                mechanism: hasUlpfec ? "red+ulpfec" : "red",
              };
              encodingParameters.push(encParam);
            }
          }
        });
        if (encodingParameters.length === 0 && primarySsrc) {
          encodingParameters.push({
            ssrc: primarySsrc,
          });
        }
        let bandwidth = SDPUtils2.matchPrefix(mediaSection, "b=");
        if (bandwidth.length) {
          if (bandwidth[0].indexOf("b=TIAS:") === 0) {
            bandwidth = parseInt(bandwidth[0].substring(7), 10);
          } else if (bandwidth[0].indexOf("b=AS:") === 0) {
            bandwidth =
              parseInt(bandwidth[0].substring(5), 10) * 1e3 * 0.95 -
              50 * 40 * 8;
          } else {
            bandwidth = void 0;
          }
          encodingParameters.forEach((params) => {
            params.maxBitrate = bandwidth;
          });
        }
        return encodingParameters;
      };
      SDPUtils2.parseRtcpParameters = function (mediaSection) {
        const rtcpParameters = {};
        const remoteSsrc = SDPUtils2.matchPrefix(mediaSection, "a=ssrc:")
          .map((line) => SDPUtils2.parseSsrcMedia(line))
          .filter((obj) => obj.attribute === "cname")[0];
        if (remoteSsrc) {
          rtcpParameters.cname = remoteSsrc.value;
          rtcpParameters.ssrc = remoteSsrc.ssrc;
        }
        const rsize = SDPUtils2.matchPrefix(mediaSection, "a=rtcp-rsize");
        rtcpParameters.reducedSize = rsize.length > 0;
        rtcpParameters.compound = rsize.length === 0;
        const mux = SDPUtils2.matchPrefix(mediaSection, "a=rtcp-mux");
        rtcpParameters.mux = mux.length > 0;
        return rtcpParameters;
      };
      SDPUtils2.writeRtcpParameters = function (rtcpParameters) {
        let sdp2 = "";
        if (rtcpParameters.reducedSize) {
          sdp2 += "a=rtcp-rsize\r\n";
        }
        if (rtcpParameters.mux) {
          sdp2 += "a=rtcp-mux\r\n";
        }
        if (rtcpParameters.ssrc !== void 0 && rtcpParameters.cname) {
          sdp2 +=
            "a=ssrc:" +
            rtcpParameters.ssrc +
            " cname:" +
            rtcpParameters.cname +
            "\r\n";
        }
        return sdp2;
      };
      SDPUtils2.parseMsid = function (mediaSection) {
        let parts;
        const spec = SDPUtils2.matchPrefix(mediaSection, "a=msid:");
        if (spec.length === 1) {
          parts = spec[0].substring(7).split(" ");
          return { stream: parts[0], track: parts[1] };
        }
        const planB = SDPUtils2.matchPrefix(mediaSection, "a=ssrc:")
          .map((line) => SDPUtils2.parseSsrcMedia(line))
          .filter((msidParts) => msidParts.attribute === "msid");
        if (planB.length > 0) {
          parts = planB[0].value.split(" ");
          return { stream: parts[0], track: parts[1] };
        }
      };
      SDPUtils2.parseSctpDescription = function (mediaSection) {
        const mline = SDPUtils2.parseMLine(mediaSection);
        const maxSizeLine = SDPUtils2.matchPrefix(
          mediaSection,
          "a=max-message-size:"
        );
        let maxMessageSize;
        if (maxSizeLine.length > 0) {
          maxMessageSize = parseInt(maxSizeLine[0].substring(19), 10);
        }
        if (isNaN(maxMessageSize)) {
          maxMessageSize = 65536;
        }
        const sctpPort = SDPUtils2.matchPrefix(mediaSection, "a=sctp-port:");
        if (sctpPort.length > 0) {
          return {
            port: parseInt(sctpPort[0].substring(12), 10),
            protocol: mline.fmt,
            maxMessageSize,
          };
        }
        const sctpMapLines = SDPUtils2.matchPrefix(mediaSection, "a=sctpmap:");
        if (sctpMapLines.length > 0) {
          const parts = sctpMapLines[0].substring(10).split(" ");
          return {
            port: parseInt(parts[0], 10),
            protocol: parts[1],
            maxMessageSize,
          };
        }
      };
      SDPUtils2.writeSctpDescription = function (media, sctp) {
        let output = [];
        if (media.protocol !== "DTLS/SCTP") {
          output = [
            "m=" +
              media.kind +
              " 9 " +
              media.protocol +
              " " +
              sctp.protocol +
              "\r\n",
            "c=IN IP4 0.0.0.0\r\n",
            "a=sctp-port:" + sctp.port + "\r\n",
          ];
        } else {
          output = [
            "m=" +
              media.kind +
              " 9 " +
              media.protocol +
              " " +
              sctp.port +
              "\r\n",
            "c=IN IP4 0.0.0.0\r\n",
            "a=sctpmap:" + sctp.port + " " + sctp.protocol + " 65535\r\n",
          ];
        }
        if (sctp.maxMessageSize !== void 0) {
          output.push("a=max-message-size:" + sctp.maxMessageSize + "\r\n");
        }
        return output.join("");
      };
      SDPUtils2.generateSessionId = function () {
        return Math.random().toString().substr(2, 22);
      };
      SDPUtils2.writeSessionBoilerplate = function (sessId, sessVer, sessUser) {
        let sessionId;
        const version2 = sessVer !== void 0 ? sessVer : 2;
        if (sessId) {
          sessionId = sessId;
        } else {
          sessionId = SDPUtils2.generateSessionId();
        }
        const user = sessUser || "thisisadapterortc";
        return (
          "v=0\r\no=" +
          user +
          " " +
          sessionId +
          " " +
          version2 +
          " IN IP4 127.0.0.1\r\ns=-\r\nt=0 0\r\n"
        );
      };
      SDPUtils2.getDirection = function (mediaSection, sessionpart) {
        const lines = SDPUtils2.splitLines(mediaSection);
        for (let i = 0; i < lines.length; i++) {
          switch (lines[i]) {
            case "a=sendrecv":
            case "a=sendonly":
            case "a=recvonly":
            case "a=inactive":
              return lines[i].substring(2);
            default:
          }
        }
        if (sessionpart) {
          return SDPUtils2.getDirection(sessionpart);
        }
        return "sendrecv";
      };
      SDPUtils2.getKind = function (mediaSection) {
        const lines = SDPUtils2.splitLines(mediaSection);
        const mline = lines[0].split(" ");
        return mline[0].substring(2);
      };
      SDPUtils2.isRejected = function (mediaSection) {
        return mediaSection.split(" ", 2)[1] === "0";
      };
      SDPUtils2.parseMLine = function (mediaSection) {
        const lines = SDPUtils2.splitLines(mediaSection);
        const parts = lines[0].substring(2).split(" ");
        return {
          kind: parts[0],
          port: parseInt(parts[1], 10),
          protocol: parts[2],
          fmt: parts.slice(3).join(" "),
        };
      };
      SDPUtils2.parseOLine = function (mediaSection) {
        const line = SDPUtils2.matchPrefix(mediaSection, "o=")[0];
        const parts = line.substring(2).split(" ");
        return {
          username: parts[0],
          sessionId: parts[1],
          sessionVersion: parseInt(parts[2], 10),
          netType: parts[3],
          addressType: parts[4],
          address: parts[5],
        };
      };
      SDPUtils2.isValidSDP = function (blob) {
        if (typeof blob !== "string" || blob.length === 0) {
          return false;
        }
        const lines = SDPUtils2.splitLines(blob);
        for (let i = 0; i < lines.length; i++) {
          if (lines[i].length < 2 || lines[i].charAt(1) !== "=") {
            return false;
          }
        }
        return true;
      };
      if (typeof module === "object") {
        module.exports = SDPUtils2;
      }
    },
  });

  // node_modules/eventemitter3/index.js
  var require_eventemitter3 = __commonJS({
    "node_modules/eventemitter3/index.js"(exports, module) {
      "use strict";
      var has = Object.prototype.hasOwnProperty;
      var prefix = "~";
      function Events() {}
      if (Object.create) {
        Events.prototype = /* @__PURE__ */ Object.create(null);
        if (!new Events().__proto__) prefix = false;
      }
      function EE(fn, context, once) {
        this.fn = fn;
        this.context = context;
        this.once = once || false;
      }
      function addListener(emitter, event, fn, context, once) {
        if (typeof fn !== "function") {
          throw new TypeError("The listener must be a function");
        }
        var listener = new EE(fn, context || emitter, once),
          evt = prefix ? prefix + event : event;
        if (!emitter._events[evt])
          (emitter._events[evt] = listener), emitter._eventsCount++;
        else if (!emitter._events[evt].fn) emitter._events[evt].push(listener);
        else emitter._events[evt] = [emitter._events[evt], listener];
        return emitter;
      }
      function clearEvent(emitter, evt) {
        if (--emitter._eventsCount === 0) emitter._events = new Events();
        else delete emitter._events[evt];
      }
      function EventEmitter5() {
        this._events = new Events();
        this._eventsCount = 0;
      }
      EventEmitter5.prototype.eventNames = function eventNames() {
        var names = [],
          events,
          name;
        if (this._eventsCount === 0) return names;
        for (name in (events = this._events)) {
          if (has.call(events, name)) names.push(prefix ? name.slice(1) : name);
        }
        if (Object.getOwnPropertySymbols) {
          return names.concat(Object.getOwnPropertySymbols(events));
        }
        return names;
      };
      EventEmitter5.prototype.listeners = function listeners(event) {
        var evt = prefix ? prefix + event : event,
          handlers = this._events[evt];
        if (!handlers) return [];
        if (handlers.fn) return [handlers.fn];
        for (var i = 0, l = handlers.length, ee = new Array(l); i < l; i++) {
          ee[i] = handlers[i].fn;
        }
        return ee;
      };
      EventEmitter5.prototype.listenerCount = function listenerCount(event) {
        var evt = prefix ? prefix + event : event,
          listeners = this._events[evt];
        if (!listeners) return 0;
        if (listeners.fn) return 1;
        return listeners.length;
      };
      EventEmitter5.prototype.emit = function emit(event, a1, a2, a3, a4, a5) {
        var evt = prefix ? prefix + event : event;
        if (!this._events[evt]) return false;
        var listeners = this._events[evt],
          len = arguments.length,
          args,
          i;
        if (listeners.fn) {
          if (listeners.once)
            this.removeListener(event, listeners.fn, void 0, true);
          switch (len) {
            case 1:
              return listeners.fn.call(listeners.context), true;
            case 2:
              return listeners.fn.call(listeners.context, a1), true;
            case 3:
              return listeners.fn.call(listeners.context, a1, a2), true;
            case 4:
              return listeners.fn.call(listeners.context, a1, a2, a3), true;
            case 5:
              return listeners.fn.call(listeners.context, a1, a2, a3, a4), true;
            case 6:
              return (
                listeners.fn.call(listeners.context, a1, a2, a3, a4, a5), true
              );
          }
          for (i = 1, args = new Array(len - 1); i < len; i++) {
            args[i - 1] = arguments[i];
          }
          listeners.fn.apply(listeners.context, args);
        } else {
          var length = listeners.length,
            j;
          for (i = 0; i < length; i++) {
            if (listeners[i].once)
              this.removeListener(event, listeners[i].fn, void 0, true);
            switch (len) {
              case 1:
                listeners[i].fn.call(listeners[i].context);
                break;
              case 2:
                listeners[i].fn.call(listeners[i].context, a1);
                break;
              case 3:
                listeners[i].fn.call(listeners[i].context, a1, a2);
                break;
              case 4:
                listeners[i].fn.call(listeners[i].context, a1, a2, a3);
                break;
              default:
                if (!args)
                  for (j = 1, args = new Array(len - 1); j < len; j++) {
                    args[j - 1] = arguments[j];
                  }
                listeners[i].fn.apply(listeners[i].context, args);
            }
          }
        }
        return true;
      };
      EventEmitter5.prototype.on = function on(event, fn, context) {
        return addListener(this, event, fn, context, false);
      };
      EventEmitter5.prototype.once = function once(event, fn, context) {
        return addListener(this, event, fn, context, true);
      };
      EventEmitter5.prototype.removeListener = function removeListener(
        event,
        fn,
        context,
        once
      ) {
        var evt = prefix ? prefix + event : event;
        if (!this._events[evt]) return this;
        if (!fn) {
          clearEvent(this, evt);
          return this;
        }
        var listeners = this._events[evt];
        if (listeners.fn) {
          if (
            listeners.fn === fn &&
            (!once || listeners.once) &&
            (!context || listeners.context === context)
          ) {
            clearEvent(this, evt);
          }
        } else {
          for (
            var i = 0, events = [], length = listeners.length;
            i < length;
            i++
          ) {
            if (
              listeners[i].fn !== fn ||
              (once && !listeners[i].once) ||
              (context && listeners[i].context !== context)
            ) {
              events.push(listeners[i]);
            }
          }
          if (events.length)
            this._events[evt] = events.length === 1 ? events[0] : events;
          else clearEvent(this, evt);
        }
        return this;
      };
      EventEmitter5.prototype.removeAllListeners = function removeAllListeners(
        event
      ) {
        var evt;
        if (event) {
          evt = prefix ? prefix + event : event;
          if (this._events[evt]) clearEvent(this, evt);
        } else {
          this._events = new Events();
          this._eventsCount = 0;
        }
        return this;
      };
      EventEmitter5.prototype.off = EventEmitter5.prototype.removeListener;
      EventEmitter5.prototype.addListener = EventEmitter5.prototype.on;
      EventEmitter5.prefixed = prefix;
      EventEmitter5.EventEmitter = EventEmitter5;
      if ("undefined" !== typeof module) {
        module.exports = EventEmitter5;
      }
    },
  });

  // lib/util.ts
  var import_peerjs_js_binarypack = __toESM(require_binarypack());

  // node_modules/webrtc-adapter/src/js/utils.js
  var logDisabled_ = true;
  var deprecationWarnings_ = true;
  function extractVersion(uastring, expr, pos) {
    const match = uastring.match(expr);
    return match && match.length >= pos && parseInt(match[pos], 10);
  }
  function wrapPeerConnectionEvent(window2, eventNameToWrap, wrapper) {
    if (!window2.RTCPeerConnection) {
      return;
    }
    const proto = window2.RTCPeerConnection.prototype;
    const nativeAddEventListener = proto.addEventListener;
    proto.addEventListener = function (nativeEventName, cb) {
      if (nativeEventName !== eventNameToWrap) {
        return nativeAddEventListener.apply(this, arguments);
      }
      const wrappedCallback = (e) => {
        const modifiedEvent = wrapper(e);
        if (modifiedEvent) {
          if (cb.handleEvent) {
            cb.handleEvent(modifiedEvent);
          } else {
            cb(modifiedEvent);
          }
        }
      };
      this._eventMap = this._eventMap || {};
      if (!this._eventMap[eventNameToWrap]) {
        this._eventMap[eventNameToWrap] = /* @__PURE__ */ new Map();
      }
      this._eventMap[eventNameToWrap].set(cb, wrappedCallback);
      return nativeAddEventListener.apply(this, [
        nativeEventName,
        wrappedCallback,
      ]);
    };
    const nativeRemoveEventListener = proto.removeEventListener;
    proto.removeEventListener = function (nativeEventName, cb) {
      if (
        nativeEventName !== eventNameToWrap ||
        !this._eventMap ||
        !this._eventMap[eventNameToWrap]
      ) {
        return nativeRemoveEventListener.apply(this, arguments);
      }
      if (!this._eventMap[eventNameToWrap].has(cb)) {
        return nativeRemoveEventListener.apply(this, arguments);
      }
      const unwrappedCb = this._eventMap[eventNameToWrap].get(cb);
      this._eventMap[eventNameToWrap].delete(cb);
      if (this._eventMap[eventNameToWrap].size === 0) {
        delete this._eventMap[eventNameToWrap];
      }
      if (Object.keys(this._eventMap).length === 0) {
        delete this._eventMap;
      }
      return nativeRemoveEventListener.apply(this, [
        nativeEventName,
        unwrappedCb,
      ]);
    };
    Object.defineProperty(proto, "on" + eventNameToWrap, {
      get() {
        return this["_on" + eventNameToWrap];
      },
      set(cb) {
        if (this["_on" + eventNameToWrap]) {
          this.removeEventListener(
            eventNameToWrap,
            this["_on" + eventNameToWrap]
          );
          delete this["_on" + eventNameToWrap];
        }
        if (cb) {
          this.addEventListener(
            eventNameToWrap,
            (this["_on" + eventNameToWrap] = cb)
          );
        }
      },
      enumerable: true,
      configurable: true,
    });
  }
  function disableLog(bool) {
    if (typeof bool !== "boolean") {
      return new Error(
        "Argument type: " + typeof bool + ". Please use a boolean."
      );
    }
    logDisabled_ = bool;
    return bool ? "adapter.js logging disabled" : "adapter.js logging enabled";
  }
  function disableWarnings(bool) {
    if (typeof bool !== "boolean") {
      return new Error(
        "Argument type: " + typeof bool + ". Please use a boolean."
      );
    }
    deprecationWarnings_ = !bool;
    return "adapter.js deprecation warnings " + (bool ? "disabled" : "enabled");
  }
  function log() {
    if (typeof window === "object") {
      if (logDisabled_) {
        return;
      }
      if (typeof console !== "undefined" && typeof console.log === "function") {
        console.log.apply(console, arguments);
      }
    }
  }
  function deprecated(oldMethod, newMethod) {
    if (!deprecationWarnings_) {
      return;
    }
    console.warn(
      oldMethod + " is deprecated, please use " + newMethod + " instead."
    );
  }
  function detectBrowser(window2) {
    const result = { browser: null, version: null };
    if (typeof window2 === "undefined" || !window2.navigator) {
      result.browser = "Not a browser.";
      return result;
    }
    const { navigator: navigator2 } = window2;
    if (navigator2.mozGetUserMedia) {
      result.browser = "firefox";
      result.version = extractVersion(
        navigator2.userAgent,
        /Firefox\/(\d+)\./,
        1
      );
    } else if (
      navigator2.webkitGetUserMedia ||
      (window2.isSecureContext === false && window2.webkitRTCPeerConnection)
    ) {
      result.browser = "chrome";
      result.version = extractVersion(
        navigator2.userAgent,
        /Chrom(e|ium)\/(\d+)\./,
        2
      );
    } else if (
      window2.RTCPeerConnection &&
      navigator2.userAgent.match(/AppleWebKit\/(\d+)\./)
    ) {
      result.browser = "safari";
      result.version = extractVersion(
        navigator2.userAgent,
        /AppleWebKit\/(\d+)\./,
        1
      );
      result.supportsUnifiedPlan =
        window2.RTCRtpTransceiver &&
        "currentDirection" in window2.RTCRtpTransceiver.prototype;
    } else {
      result.browser = "Not a supported browser.";
      return result;
    }
    return result;
  }
  function isObject(val) {
    return Object.prototype.toString.call(val) === "[object Object]";
  }
  function compactObject(data) {
    if (!isObject(data)) {
      return data;
    }
    return Object.keys(data).reduce(function (accumulator, key) {
      const isObj = isObject(data[key]);
      const value = isObj ? compactObject(data[key]) : data[key];
      const isEmptyObject = isObj && !Object.keys(value).length;
      if (value === void 0 || isEmptyObject) {
        return accumulator;
      }
      return Object.assign(accumulator, { [key]: value });
    }, {});
  }
  function walkStats(stats, base, resultSet) {
    if (!base || resultSet.has(base.id)) {
      return;
    }
    resultSet.set(base.id, base);
    Object.keys(base).forEach((name) => {
      if (name.endsWith("Id")) {
        walkStats(stats, stats.get(base[name]), resultSet);
      } else if (name.endsWith("Ids")) {
        base[name].forEach((id) => {
          walkStats(stats, stats.get(id), resultSet);
        });
      }
    });
  }
  function filterStats(result, track, outbound) {
    const streamStatsType = outbound ? "outbound-rtp" : "inbound-rtp";
    const filteredResult = /* @__PURE__ */ new Map();
    if (track === null) {
      return filteredResult;
    }
    const trackStats = [];
    result.forEach((value) => {
      if (value.type === "track" && value.trackIdentifier === track.id) {
        trackStats.push(value);
      }
    });
    trackStats.forEach((trackStat) => {
      result.forEach((stats) => {
        if (stats.type === streamStatsType && stats.trackId === trackStat.id) {
          walkStats(result, stats, filteredResult);
        }
      });
    });
    return filteredResult;
  }

  // node_modules/webrtc-adapter/src/js/chrome/chrome_shim.js
  var chrome_shim_exports = {};
  __export(chrome_shim_exports, {
    fixNegotiationNeeded: () => fixNegotiationNeeded,
    shimAddTrackRemoveTrack: () => shimAddTrackRemoveTrack,
    shimAddTrackRemoveTrackWithNative: () => shimAddTrackRemoveTrackWithNative,
    shimGetDisplayMedia: () => shimGetDisplayMedia,
    shimGetSendersWithDtmf: () => shimGetSendersWithDtmf,
    shimGetStats: () => shimGetStats,
    shimGetUserMedia: () => shimGetUserMedia,
    shimMediaStream: () => shimMediaStream,
    shimOnTrack: () => shimOnTrack,
    shimPeerConnection: () => shimPeerConnection,
    shimSenderReceiverGetStats: () => shimSenderReceiverGetStats,
  });

  // node_modules/webrtc-adapter/src/js/chrome/getusermedia.js
  var logging = log;
  function shimGetUserMedia(window2, browserDetails) {
    const navigator2 = window2 && window2.navigator;
    if (!navigator2.mediaDevices) {
      return;
    }
    const constraintsToChrome_ = function (c) {
      if (typeof c !== "object" || c.mandatory || c.optional) {
        return c;
      }
      const cc = {};
      Object.keys(c).forEach((key) => {
        if (key === "require" || key === "advanced" || key === "mediaSource") {
          return;
        }
        const r = typeof c[key] === "object" ? c[key] : { ideal: c[key] };
        if (r.exact !== void 0 && typeof r.exact === "number") {
          r.min = r.max = r.exact;
        }
        const oldname_ = function (prefix, name) {
          if (prefix) {
            return prefix + name.charAt(0).toUpperCase() + name.slice(1);
          }
          return name === "deviceId" ? "sourceId" : name;
        };
        if (r.ideal !== void 0) {
          cc.optional = cc.optional || [];
          let oc = {};
          if (typeof r.ideal === "number") {
            oc[oldname_("min", key)] = r.ideal;
            cc.optional.push(oc);
            oc = {};
            oc[oldname_("max", key)] = r.ideal;
            cc.optional.push(oc);
          } else {
            oc[oldname_("", key)] = r.ideal;
            cc.optional.push(oc);
          }
        }
        if (r.exact !== void 0 && typeof r.exact !== "number") {
          cc.mandatory = cc.mandatory || {};
          cc.mandatory[oldname_("", key)] = r.exact;
        } else {
          ["min", "max"].forEach((mix) => {
            if (r[mix] !== void 0) {
              cc.mandatory = cc.mandatory || {};
              cc.mandatory[oldname_(mix, key)] = r[mix];
            }
          });
        }
      });
      if (c.advanced) {
        cc.optional = (cc.optional || []).concat(c.advanced);
      }
      return cc;
    };
    const shimConstraints_ = function (constraints, func) {
      if (browserDetails.version >= 61) {
        return func(constraints);
      }
      constraints = JSON.parse(JSON.stringify(constraints));
      if (constraints && typeof constraints.audio === "object") {
        const remap = function (obj, a, b) {
          if (a in obj && !(b in obj)) {
            obj[b] = obj[a];
            delete obj[a];
          }
        };
        constraints = JSON.parse(JSON.stringify(constraints));
        remap(constraints.audio, "autoGainControl", "googAutoGainControl");
        remap(constraints.audio, "noiseSuppression", "googNoiseSuppression");
        constraints.audio = constraintsToChrome_(constraints.audio);
      }
      if (constraints && typeof constraints.video === "object") {
        let face = constraints.video.facingMode;
        face = face && (typeof face === "object" ? face : { ideal: face });
        const getSupportedFacingModeLies = browserDetails.version < 66;
        if (
          face &&
          (face.exact === "user" ||
            face.exact === "environment" ||
            face.ideal === "user" ||
            face.ideal === "environment") &&
          !(
            navigator2.mediaDevices.getSupportedConstraints &&
            navigator2.mediaDevices.getSupportedConstraints().facingMode &&
            !getSupportedFacingModeLies
          )
        ) {
          delete constraints.video.facingMode;
          let matches;
          if (face.exact === "environment" || face.ideal === "environment") {
            matches = ["back", "rear"];
          } else if (face.exact === "user" || face.ideal === "user") {
            matches = ["front"];
          }
          if (matches) {
            return navigator2.mediaDevices
              .enumerateDevices()
              .then((devices) => {
                devices = devices.filter((d) => d.kind === "videoinput");
                let dev = devices.find((d) =>
                  matches.some((match) => d.label.toLowerCase().includes(match))
                );
                if (!dev && devices.length && matches.includes("back")) {
                  dev = devices[devices.length - 1];
                }
                if (dev) {
                  constraints.video.deviceId = face.exact
                    ? { exact: dev.deviceId }
                    : { ideal: dev.deviceId };
                }
                constraints.video = constraintsToChrome_(constraints.video);
                logging("chrome: " + JSON.stringify(constraints));
                return func(constraints);
              });
          }
        }
        constraints.video = constraintsToChrome_(constraints.video);
      }
      logging("chrome: " + JSON.stringify(constraints));
      return func(constraints);
    };
    const shimError_ = function (e) {
      if (browserDetails.version >= 64) {
        return e;
      }
      return {
        name:
          {
            PermissionDeniedError: "NotAllowedError",
            PermissionDismissedError: "NotAllowedError",
            InvalidStateError: "NotAllowedError",
            DevicesNotFoundError: "NotFoundError",
            ConstraintNotSatisfiedError: "OverconstrainedError",
            TrackStartError: "NotReadableError",
            MediaDeviceFailedDueToShutdown: "NotAllowedError",
            MediaDeviceKillSwitchOn: "NotAllowedError",
            TabCaptureError: "AbortError",
            ScreenCaptureError: "AbortError",
            DeviceCaptureError: "AbortError",
          }[e.name] || e.name,
        message: e.message,
        constraint: e.constraint || e.constraintName,
        toString() {
          return this.name + (this.message && ": ") + this.message;
        },
      };
    };
    const getUserMedia_ = function (constraints, onSuccess, onError) {
      shimConstraints_(constraints, (c) => {
        navigator2.webkitGetUserMedia(c, onSuccess, (e) => {
          if (onError) {
            onError(shimError_(e));
          }
        });
      });
    };
    navigator2.getUserMedia = getUserMedia_.bind(navigator2);
    if (navigator2.mediaDevices.getUserMedia) {
      const origGetUserMedia = navigator2.mediaDevices.getUserMedia.bind(
        navigator2.mediaDevices
      );
      navigator2.mediaDevices.getUserMedia = function (cs) {
        return shimConstraints_(cs, (c) =>
          origGetUserMedia(c).then(
            (stream) => {
              if (
                (c.audio && !stream.getAudioTracks().length) ||
                (c.video && !stream.getVideoTracks().length)
              ) {
                stream.getTracks().forEach((track) => {
                  track.stop();
                });
                throw new DOMException("", "NotFoundError");
              }
              return stream;
            },
            (e) => Promise.reject(shimError_(e))
          )
        );
      };
    }
  }

  // node_modules/webrtc-adapter/src/js/chrome/getdisplaymedia.js
  function shimGetDisplayMedia(window2, getSourceId) {
    if (
      window2.navigator.mediaDevices &&
      "getDisplayMedia" in window2.navigator.mediaDevices
    ) {
      return;
    }
    if (!window2.navigator.mediaDevices) {
      return;
    }
    if (typeof getSourceId !== "function") {
      console.error(
        "shimGetDisplayMedia: getSourceId argument is not a function"
      );
      return;
    }
    window2.navigator.mediaDevices.getDisplayMedia = function getDisplayMedia(
      constraints
    ) {
      return getSourceId(constraints).then((sourceId) => {
        const widthSpecified = constraints.video && constraints.video.width;
        const heightSpecified = constraints.video && constraints.video.height;
        const frameRateSpecified =
          constraints.video && constraints.video.frameRate;
        constraints.video = {
          mandatory: {
            chromeMediaSource: "desktop",
            chromeMediaSourceId: sourceId,
            maxFrameRate: frameRateSpecified || 3,
          },
        };
        if (widthSpecified) {
          constraints.video.mandatory.maxWidth = widthSpecified;
        }
        if (heightSpecified) {
          constraints.video.mandatory.maxHeight = heightSpecified;
        }
        return window2.navigator.mediaDevices.getUserMedia(constraints);
      });
    };
  }

  // node_modules/webrtc-adapter/src/js/chrome/chrome_shim.js
  function shimMediaStream(window2) {
    window2.MediaStream = window2.MediaStream || window2.webkitMediaStream;
  }
  function shimOnTrack(window2) {
    if (
      typeof window2 === "object" &&
      window2.RTCPeerConnection &&
      !("ontrack" in window2.RTCPeerConnection.prototype)
    ) {
      Object.defineProperty(window2.RTCPeerConnection.prototype, "ontrack", {
        get() {
          return this._ontrack;
        },
        set(f) {
          if (this._ontrack) {
            this.removeEventListener("track", this._ontrack);
          }
          this.addEventListener("track", (this._ontrack = f));
        },
        enumerable: true,
        configurable: true,
      });
      const origSetRemoteDescription =
        window2.RTCPeerConnection.prototype.setRemoteDescription;
      window2.RTCPeerConnection.prototype.setRemoteDescription =
        function setRemoteDescription() {
          if (!this._ontrackpoly) {
            this._ontrackpoly = (e) => {
              e.stream.addEventListener("addtrack", (te) => {
                let receiver;
                if (window2.RTCPeerConnection.prototype.getReceivers) {
                  receiver = this.getReceivers().find(
                    (r) => r.track && r.track.id === te.track.id
                  );
                } else {
                  receiver = { track: te.track };
                }
                const event = new Event("track");
                event.track = te.track;
                event.receiver = receiver;
                event.transceiver = { receiver };
                event.streams = [e.stream];
                this.dispatchEvent(event);
              });
              e.stream.getTracks().forEach((track) => {
                let receiver;
                if (window2.RTCPeerConnection.prototype.getReceivers) {
                  receiver = this.getReceivers().find(
                    (r) => r.track && r.track.id === track.id
                  );
                } else {
                  receiver = { track };
                }
                const event = new Event("track");
                event.track = track;
                event.receiver = receiver;
                event.transceiver = { receiver };
                event.streams = [e.stream];
                this.dispatchEvent(event);
              });
            };
            this.addEventListener("addstream", this._ontrackpoly);
          }
          return origSetRemoteDescription.apply(this, arguments);
        };
    } else {
      wrapPeerConnectionEvent(window2, "track", (e) => {
        if (!e.transceiver) {
          Object.defineProperty(e, "transceiver", {
            value: { receiver: e.receiver },
          });
        }
        return e;
      });
    }
  }
  function shimGetSendersWithDtmf(window2) {
    if (
      typeof window2 === "object" &&
      window2.RTCPeerConnection &&
      !("getSenders" in window2.RTCPeerConnection.prototype) &&
      "createDTMFSender" in window2.RTCPeerConnection.prototype
    ) {
      const shimSenderWithDtmf = function (pc, track) {
        return {
          track,
          get dtmf() {
            if (this._dtmf === void 0) {
              if (track.kind === "audio") {
                this._dtmf = pc.createDTMFSender(track);
              } else {
                this._dtmf = null;
              }
            }
            return this._dtmf;
          },
          _pc: pc,
        };
      };
      if (!window2.RTCPeerConnection.prototype.getSenders) {
        window2.RTCPeerConnection.prototype.getSenders = function getSenders() {
          this._senders = this._senders || [];
          return this._senders.slice();
        };
        const origAddTrack = window2.RTCPeerConnection.prototype.addTrack;
        window2.RTCPeerConnection.prototype.addTrack = function addTrack(
          track,
          stream
        ) {
          let sender = origAddTrack.apply(this, arguments);
          if (!sender) {
            sender = shimSenderWithDtmf(this, track);
            this._senders.push(sender);
          }
          return sender;
        };
        const origRemoveTrack = window2.RTCPeerConnection.prototype.removeTrack;
        window2.RTCPeerConnection.prototype.removeTrack = function removeTrack(
          sender
        ) {
          origRemoveTrack.apply(this, arguments);
          const idx = this._senders.indexOf(sender);
          if (idx !== -1) {
            this._senders.splice(idx, 1);
          }
        };
      }
      const origAddStream = window2.RTCPeerConnection.prototype.addStream;
      window2.RTCPeerConnection.prototype.addStream = function addStream(
        stream
      ) {
        this._senders = this._senders || [];
        origAddStream.apply(this, [stream]);
        stream.getTracks().forEach((track) => {
          this._senders.push(shimSenderWithDtmf(this, track));
        });
      };
      const origRemoveStream = window2.RTCPeerConnection.prototype.removeStream;
      window2.RTCPeerConnection.prototype.removeStream = function removeStream(
        stream
      ) {
        this._senders = this._senders || [];
        origRemoveStream.apply(this, [stream]);
        stream.getTracks().forEach((track) => {
          const sender = this._senders.find((s) => s.track === track);
          if (sender) {
            this._senders.splice(this._senders.indexOf(sender), 1);
          }
        });
      };
    } else if (
      typeof window2 === "object" &&
      window2.RTCPeerConnection &&
      "getSenders" in window2.RTCPeerConnection.prototype &&
      "createDTMFSender" in window2.RTCPeerConnection.prototype &&
      window2.RTCRtpSender &&
      !("dtmf" in window2.RTCRtpSender.prototype)
    ) {
      const origGetSenders = window2.RTCPeerConnection.prototype.getSenders;
      window2.RTCPeerConnection.prototype.getSenders = function getSenders() {
        const senders = origGetSenders.apply(this, []);
        senders.forEach((sender) => (sender._pc = this));
        return senders;
      };
      Object.defineProperty(window2.RTCRtpSender.prototype, "dtmf", {
        get() {
          if (this._dtmf === void 0) {
            if (this.track.kind === "audio") {
              this._dtmf = this._pc.createDTMFSender(this.track);
            } else {
              this._dtmf = null;
            }
          }
          return this._dtmf;
        },
      });
    }
  }
  function shimGetStats(window2) {
    if (!window2.RTCPeerConnection) {
      return;
    }
    const origGetStats = window2.RTCPeerConnection.prototype.getStats;
    window2.RTCPeerConnection.prototype.getStats = function getStats() {
      const [selector, onSucc, onErr] = arguments;
      if (arguments.length > 0 && typeof selector === "function") {
        return origGetStats.apply(this, arguments);
      }
      if (
        origGetStats.length === 0 &&
        (arguments.length === 0 || typeof selector !== "function")
      ) {
        return origGetStats.apply(this, []);
      }
      const fixChromeStats_ = function (response) {
        const standardReport = {};
        const reports = response.result();
        reports.forEach((report) => {
          const standardStats = {
            id: report.id,
            timestamp: report.timestamp,
            type:
              {
                localcandidate: "local-candidate",
                remotecandidate: "remote-candidate",
              }[report.type] || report.type,
          };
          report.names().forEach((name) => {
            standardStats[name] = report.stat(name);
          });
          standardReport[standardStats.id] = standardStats;
        });
        return standardReport;
      };
      const makeMapStats = function (stats) {
        return new Map(Object.keys(stats).map((key) => [key, stats[key]]));
      };
      if (arguments.length >= 2) {
        const successCallbackWrapper_ = function (response) {
          onSucc(makeMapStats(fixChromeStats_(response)));
        };
        return origGetStats.apply(this, [successCallbackWrapper_, selector]);
      }
      return new Promise((resolve, reject) => {
        origGetStats.apply(this, [
          function (response) {
            resolve(makeMapStats(fixChromeStats_(response)));
          },
          reject,
        ]);
      }).then(onSucc, onErr);
    };
  }
  function shimSenderReceiverGetStats(window2) {
    if (
      !(
        typeof window2 === "object" &&
        window2.RTCPeerConnection &&
        window2.RTCRtpSender &&
        window2.RTCRtpReceiver
      )
    ) {
      return;
    }
    if (!("getStats" in window2.RTCRtpSender.prototype)) {
      const origGetSenders = window2.RTCPeerConnection.prototype.getSenders;
      if (origGetSenders) {
        window2.RTCPeerConnection.prototype.getSenders = function getSenders() {
          const senders = origGetSenders.apply(this, []);
          senders.forEach((sender) => (sender._pc = this));
          return senders;
        };
      }
      const origAddTrack = window2.RTCPeerConnection.prototype.addTrack;
      if (origAddTrack) {
        window2.RTCPeerConnection.prototype.addTrack = function addTrack() {
          const sender = origAddTrack.apply(this, arguments);
          sender._pc = this;
          return sender;
        };
      }
      window2.RTCRtpSender.prototype.getStats = function getStats() {
        const sender = this;
        return this._pc.getStats().then((result) =>
          /* Note: this will include stats of all senders that
           *   send a track with the same id as sender.track as
           *   it is not possible to identify the RTCRtpSender.
           */
          filterStats(result, sender.track, true)
        );
      };
    }
    if (!("getStats" in window2.RTCRtpReceiver.prototype)) {
      const origGetReceivers = window2.RTCPeerConnection.prototype.getReceivers;
      if (origGetReceivers) {
        window2.RTCPeerConnection.prototype.getReceivers =
          function getReceivers() {
            const receivers = origGetReceivers.apply(this, []);
            receivers.forEach((receiver) => (receiver._pc = this));
            return receivers;
          };
      }
      wrapPeerConnectionEvent(window2, "track", (e) => {
        e.receiver._pc = e.srcElement;
        return e;
      });
      window2.RTCRtpReceiver.prototype.getStats = function getStats() {
        const receiver = this;
        return this._pc
          .getStats()
          .then((result) => filterStats(result, receiver.track, false));
      };
    }
    if (
      !(
        "getStats" in window2.RTCRtpSender.prototype &&
        "getStats" in window2.RTCRtpReceiver.prototype
      )
    ) {
      return;
    }
    const origGetStats = window2.RTCPeerConnection.prototype.getStats;
    window2.RTCPeerConnection.prototype.getStats = function getStats() {
      if (
        arguments.length > 0 &&
        arguments[0] instanceof window2.MediaStreamTrack
      ) {
        const track = arguments[0];
        let sender;
        let receiver;
        let err;
        this.getSenders().forEach((s) => {
          if (s.track === track) {
            if (sender) {
              err = true;
            } else {
              sender = s;
            }
          }
        });
        this.getReceivers().forEach((r) => {
          if (r.track === track) {
            if (receiver) {
              err = true;
            } else {
              receiver = r;
            }
          }
          return r.track === track;
        });
        if (err || (sender && receiver)) {
          return Promise.reject(
            new DOMException(
              "There are more than one sender or receiver for the track.",
              "InvalidAccessError"
            )
          );
        } else if (sender) {
          return sender.getStats();
        } else if (receiver) {
          return receiver.getStats();
        }
        return Promise.reject(
          new DOMException(
            "There is no sender or receiver for the track.",
            "InvalidAccessError"
          )
        );
      }
      return origGetStats.apply(this, arguments);
    };
  }
  function shimAddTrackRemoveTrackWithNative(window2) {
    window2.RTCPeerConnection.prototype.getLocalStreams =
      function getLocalStreams() {
        this._shimmedLocalStreams = this._shimmedLocalStreams || {};
        return Object.keys(this._shimmedLocalStreams).map(
          (streamId) => this._shimmedLocalStreams[streamId][0]
        );
      };
    const origAddTrack = window2.RTCPeerConnection.prototype.addTrack;
    window2.RTCPeerConnection.prototype.addTrack = function addTrack(
      track,
      stream
    ) {
      if (!stream) {
        return origAddTrack.apply(this, arguments);
      }
      this._shimmedLocalStreams = this._shimmedLocalStreams || {};
      const sender = origAddTrack.apply(this, arguments);
      if (!this._shimmedLocalStreams[stream.id]) {
        this._shimmedLocalStreams[stream.id] = [stream, sender];
      } else if (this._shimmedLocalStreams[stream.id].indexOf(sender) === -1) {
        this._shimmedLocalStreams[stream.id].push(sender);
      }
      return sender;
    };
    const origAddStream = window2.RTCPeerConnection.prototype.addStream;
    window2.RTCPeerConnection.prototype.addStream = function addStream(stream) {
      this._shimmedLocalStreams = this._shimmedLocalStreams || {};
      stream.getTracks().forEach((track) => {
        const alreadyExists = this.getSenders().find((s) => s.track === track);
        if (alreadyExists) {
          throw new DOMException("Track already exists.", "InvalidAccessError");
        }
      });
      const existingSenders = this.getSenders();
      origAddStream.apply(this, arguments);
      const newSenders = this.getSenders().filter(
        (newSender) => existingSenders.indexOf(newSender) === -1
      );
      this._shimmedLocalStreams[stream.id] = [stream].concat(newSenders);
    };
    const origRemoveStream = window2.RTCPeerConnection.prototype.removeStream;
    window2.RTCPeerConnection.prototype.removeStream = function removeStream(
      stream
    ) {
      this._shimmedLocalStreams = this._shimmedLocalStreams || {};
      delete this._shimmedLocalStreams[stream.id];
      return origRemoveStream.apply(this, arguments);
    };
    const origRemoveTrack = window2.RTCPeerConnection.prototype.removeTrack;
    window2.RTCPeerConnection.prototype.removeTrack = function removeTrack(
      sender
    ) {
      this._shimmedLocalStreams = this._shimmedLocalStreams || {};
      if (sender) {
        Object.keys(this._shimmedLocalStreams).forEach((streamId) => {
          const idx = this._shimmedLocalStreams[streamId].indexOf(sender);
          if (idx !== -1) {
            this._shimmedLocalStreams[streamId].splice(idx, 1);
          }
          if (this._shimmedLocalStreams[streamId].length === 1) {
            delete this._shimmedLocalStreams[streamId];
          }
        });
      }
      return origRemoveTrack.apply(this, arguments);
    };
  }
  function shimAddTrackRemoveTrack(window2, browserDetails) {
    if (!window2.RTCPeerConnection) {
      return;
    }
    if (
      window2.RTCPeerConnection.prototype.addTrack &&
      browserDetails.version >= 65
    ) {
      return shimAddTrackRemoveTrackWithNative(window2);
    }
    const origGetLocalStreams =
      window2.RTCPeerConnection.prototype.getLocalStreams;
    window2.RTCPeerConnection.prototype.getLocalStreams =
      function getLocalStreams() {
        const nativeStreams = origGetLocalStreams.apply(this);
        this._reverseStreams = this._reverseStreams || {};
        return nativeStreams.map((stream) => this._reverseStreams[stream.id]);
      };
    const origAddStream = window2.RTCPeerConnection.prototype.addStream;
    window2.RTCPeerConnection.prototype.addStream = function addStream(stream) {
      this._streams = this._streams || {};
      this._reverseStreams = this._reverseStreams || {};
      stream.getTracks().forEach((track) => {
        const alreadyExists = this.getSenders().find((s) => s.track === track);
        if (alreadyExists) {
          throw new DOMException("Track already exists.", "InvalidAccessError");
        }
      });
      if (!this._reverseStreams[stream.id]) {
        const newStream = new window2.MediaStream(stream.getTracks());
        this._streams[stream.id] = newStream;
        this._reverseStreams[newStream.id] = stream;
        stream = newStream;
      }
      origAddStream.apply(this, [stream]);
    };
    const origRemoveStream = window2.RTCPeerConnection.prototype.removeStream;
    window2.RTCPeerConnection.prototype.removeStream = function removeStream(
      stream
    ) {
      this._streams = this._streams || {};
      this._reverseStreams = this._reverseStreams || {};
      origRemoveStream.apply(this, [this._streams[stream.id] || stream]);
      delete this._reverseStreams[
        this._streams[stream.id] ? this._streams[stream.id].id : stream.id
      ];
      delete this._streams[stream.id];
    };
    window2.RTCPeerConnection.prototype.addTrack = function addTrack(
      track,
      stream
    ) {
      if (this.signalingState === "closed") {
        throw new DOMException(
          "The RTCPeerConnection's signalingState is 'closed'.",
          "InvalidStateError"
        );
      }
      const streams = [].slice.call(arguments, 1);
      if (
        streams.length !== 1 ||
        !streams[0].getTracks().find((t) => t === track)
      ) {
        throw new DOMException(
          "The adapter.js addTrack polyfill only supports a single  stream which is associated with the specified track.",
          "NotSupportedError"
        );
      }
      const alreadyExists = this.getSenders().find((s) => s.track === track);
      if (alreadyExists) {
        throw new DOMException("Track already exists.", "InvalidAccessError");
      }
      this._streams = this._streams || {};
      this._reverseStreams = this._reverseStreams || {};
      const oldStream = this._streams[stream.id];
      if (oldStream) {
        oldStream.addTrack(track);
        Promise.resolve().then(() => {
          this.dispatchEvent(new Event("negotiationneeded"));
        });
      } else {
        const newStream = new window2.MediaStream([track]);
        this._streams[stream.id] = newStream;
        this._reverseStreams[newStream.id] = stream;
        this.addStream(newStream);
      }
      return this.getSenders().find((s) => s.track === track);
    };
    function replaceInternalStreamId(pc, description) {
      let sdp2 = description.sdp;
      Object.keys(pc._reverseStreams || []).forEach((internalId) => {
        const externalStream = pc._reverseStreams[internalId];
        const internalStream = pc._streams[externalStream.id];
        sdp2 = sdp2.replace(
          new RegExp(internalStream.id, "g"),
          externalStream.id
        );
      });
      return new RTCSessionDescription({
        type: description.type,
        sdp: sdp2,
      });
    }
    function replaceExternalStreamId(pc, description) {
      let sdp2 = description.sdp;
      Object.keys(pc._reverseStreams || []).forEach((internalId) => {
        const externalStream = pc._reverseStreams[internalId];
        const internalStream = pc._streams[externalStream.id];
        sdp2 = sdp2.replace(
          new RegExp(externalStream.id, "g"),
          internalStream.id
        );
      });
      return new RTCSessionDescription({
        type: description.type,
        sdp: sdp2,
      });
    }
    ["createOffer", "createAnswer"].forEach(function (method) {
      const nativeMethod = window2.RTCPeerConnection.prototype[method];
      const methodObj = {
        [method]() {
          const args = arguments;
          const isLegacyCall =
            arguments.length && typeof arguments[0] === "function";
          if (isLegacyCall) {
            return nativeMethod.apply(this, [
              (description) => {
                const desc = replaceInternalStreamId(this, description);
                args[0].apply(null, [desc]);
              },
              (err) => {
                if (args[1]) {
                  args[1].apply(null, err);
                }
              },
              arguments[2],
            ]);
          }
          return nativeMethod
            .apply(this, arguments)
            .then((description) => replaceInternalStreamId(this, description));
        },
      };
      window2.RTCPeerConnection.prototype[method] = methodObj[method];
    });
    const origSetLocalDescription =
      window2.RTCPeerConnection.prototype.setLocalDescription;
    window2.RTCPeerConnection.prototype.setLocalDescription =
      function setLocalDescription() {
        if (!arguments.length || !arguments[0].type) {
          return origSetLocalDescription.apply(this, arguments);
        }
        arguments[0] = replaceExternalStreamId(this, arguments[0]);
        return origSetLocalDescription.apply(this, arguments);
      };
    const origLocalDescription = Object.getOwnPropertyDescriptor(
      window2.RTCPeerConnection.prototype,
      "localDescription"
    );
    Object.defineProperty(
      window2.RTCPeerConnection.prototype,
      "localDescription",
      {
        get() {
          const description = origLocalDescription.get.apply(this);
          if (description.type === "") {
            return description;
          }
          return replaceInternalStreamId(this, description);
        },
      }
    );
    window2.RTCPeerConnection.prototype.removeTrack = function removeTrack(
      sender
    ) {
      if (this.signalingState === "closed") {
        throw new DOMException(
          "The RTCPeerConnection's signalingState is 'closed'.",
          "InvalidStateError"
        );
      }
      if (!sender._pc) {
        throw new DOMException(
          "Argument 1 of RTCPeerConnection.removeTrack does not implement interface RTCRtpSender.",
          "TypeError"
        );
      }
      const isLocal = sender._pc === this;
      if (!isLocal) {
        throw new DOMException(
          "Sender was not created by this connection.",
          "InvalidAccessError"
        );
      }
      this._streams = this._streams || {};
      let stream;
      Object.keys(this._streams).forEach((streamid) => {
        const hasTrack = this._streams[streamid]
          .getTracks()
          .find((track) => sender.track === track);
        if (hasTrack) {
          stream = this._streams[streamid];
        }
      });
      if (stream) {
        if (stream.getTracks().length === 1) {
          this.removeStream(this._reverseStreams[stream.id]);
        } else {
          stream.removeTrack(sender.track);
        }
        this.dispatchEvent(new Event("negotiationneeded"));
      }
    };
  }
  function shimPeerConnection(window2, browserDetails) {
    if (!window2.RTCPeerConnection && window2.webkitRTCPeerConnection) {
      window2.RTCPeerConnection = window2.webkitRTCPeerConnection;
    }
    if (!window2.RTCPeerConnection) {
      return;
    }
    if (browserDetails.version < 53) {
      [
        "setLocalDescription",
        "setRemoteDescription",
        "addIceCandidate",
      ].forEach(function (method) {
        const nativeMethod = window2.RTCPeerConnection.prototype[method];
        const methodObj = {
          [method]() {
            arguments[0] = new (
              method === "addIceCandidate"
                ? window2.RTCIceCandidate
                : window2.RTCSessionDescription
            )(arguments[0]);
            return nativeMethod.apply(this, arguments);
          },
        };
        window2.RTCPeerConnection.prototype[method] = methodObj[method];
      });
    }
  }
  function fixNegotiationNeeded(window2, browserDetails) {
    wrapPeerConnectionEvent(window2, "negotiationneeded", (e) => {
      const pc = e.target;
      if (
        browserDetails.version < 72 ||
        (pc.getConfiguration && pc.getConfiguration().sdpSemantics === "plan-b")
      ) {
        if (pc.signalingState !== "stable") {
          return;
        }
      }
      return e;
    });
  }

  // node_modules/webrtc-adapter/src/js/firefox/firefox_shim.js
  var firefox_shim_exports = {};
  __export(firefox_shim_exports, {
    shimAddTransceiver: () => shimAddTransceiver,
    shimCreateAnswer: () => shimCreateAnswer,
    shimCreateOffer: () => shimCreateOffer,
    shimGetDisplayMedia: () => shimGetDisplayMedia2,
    shimGetParameters: () => shimGetParameters,
    shimGetUserMedia: () => shimGetUserMedia2,
    shimOnTrack: () => shimOnTrack2,
    shimPeerConnection: () => shimPeerConnection2,
    shimRTCDataChannel: () => shimRTCDataChannel,
    shimReceiverGetStats: () => shimReceiverGetStats,
    shimRemoveStream: () => shimRemoveStream,
    shimSenderGetStats: () => shimSenderGetStats,
  });

  // node_modules/webrtc-adapter/src/js/firefox/getusermedia.js
  function shimGetUserMedia2(window2, browserDetails) {
    const navigator2 = window2 && window2.navigator;
    const MediaStreamTrack = window2 && window2.MediaStreamTrack;
    navigator2.getUserMedia = function (constraints, onSuccess, onError) {
      deprecated(
        "navigator.getUserMedia",
        "navigator.mediaDevices.getUserMedia"
      );
      navigator2.mediaDevices
        .getUserMedia(constraints)
        .then(onSuccess, onError);
    };
    if (
      !(
        browserDetails.version > 55 &&
        "autoGainControl" in navigator2.mediaDevices.getSupportedConstraints()
      )
    ) {
      const remap = function (obj, a, b) {
        if (a in obj && !(b in obj)) {
          obj[b] = obj[a];
          delete obj[a];
        }
      };
      const nativeGetUserMedia = navigator2.mediaDevices.getUserMedia.bind(
        navigator2.mediaDevices
      );
      navigator2.mediaDevices.getUserMedia = function (c) {
        if (typeof c === "object" && typeof c.audio === "object") {
          c = JSON.parse(JSON.stringify(c));
          remap(c.audio, "autoGainControl", "mozAutoGainControl");
          remap(c.audio, "noiseSuppression", "mozNoiseSuppression");
        }
        return nativeGetUserMedia(c);
      };
      if (MediaStreamTrack && MediaStreamTrack.prototype.getSettings) {
        const nativeGetSettings = MediaStreamTrack.prototype.getSettings;
        MediaStreamTrack.prototype.getSettings = function () {
          const obj = nativeGetSettings.apply(this, arguments);
          remap(obj, "mozAutoGainControl", "autoGainControl");
          remap(obj, "mozNoiseSuppression", "noiseSuppression");
          return obj;
        };
      }
      if (MediaStreamTrack && MediaStreamTrack.prototype.applyConstraints) {
        const nativeApplyConstraints =
          MediaStreamTrack.prototype.applyConstraints;
        MediaStreamTrack.prototype.applyConstraints = function (c) {
          if (this.kind === "audio" && typeof c === "object") {
            c = JSON.parse(JSON.stringify(c));
            remap(c, "autoGainControl", "mozAutoGainControl");
            remap(c, "noiseSuppression", "mozNoiseSuppression");
          }
          return nativeApplyConstraints.apply(this, [c]);
        };
      }
    }
  }

  // node_modules/webrtc-adapter/src/js/firefox/getdisplaymedia.js
  function shimGetDisplayMedia2(window2, preferredMediaSource) {
    if (
      window2.navigator.mediaDevices &&
      "getDisplayMedia" in window2.navigator.mediaDevices
    ) {
      return;
    }
    if (!window2.navigator.mediaDevices) {
      return;
    }
    window2.navigator.mediaDevices.getDisplayMedia = function getDisplayMedia(
      constraints
    ) {
      if (!(constraints && constraints.video)) {
        const err = new DOMException(
          "getDisplayMedia without video constraints is undefined"
        );
        err.name = "NotFoundError";
        err.code = 8;
        return Promise.reject(err);
      }
      if (constraints.video === true) {
        constraints.video = { mediaSource: preferredMediaSource };
      } else {
        constraints.video.mediaSource = preferredMediaSource;
      }
      return window2.navigator.mediaDevices.getUserMedia(constraints);
    };
  }

  // node_modules/webrtc-adapter/src/js/firefox/firefox_shim.js
  function shimOnTrack2(window2) {
    if (
      typeof window2 === "object" &&
      window2.RTCTrackEvent &&
      "receiver" in window2.RTCTrackEvent.prototype &&
      !("transceiver" in window2.RTCTrackEvent.prototype)
    ) {
      Object.defineProperty(window2.RTCTrackEvent.prototype, "transceiver", {
        get() {
          return { receiver: this.receiver };
        },
      });
    }
  }
  function shimPeerConnection2(window2, browserDetails) {
    if (
      typeof window2 !== "object" ||
      !(window2.RTCPeerConnection || window2.mozRTCPeerConnection)
    ) {
      return;
    }
    if (!window2.RTCPeerConnection && window2.mozRTCPeerConnection) {
      window2.RTCPeerConnection = window2.mozRTCPeerConnection;
    }
    if (browserDetails.version < 53) {
      [
        "setLocalDescription",
        "setRemoteDescription",
        "addIceCandidate",
      ].forEach(function (method) {
        const nativeMethod = window2.RTCPeerConnection.prototype[method];
        const methodObj = {
          [method]() {
            arguments[0] = new (
              method === "addIceCandidate"
                ? window2.RTCIceCandidate
                : window2.RTCSessionDescription
            )(arguments[0]);
            return nativeMethod.apply(this, arguments);
          },
        };
        window2.RTCPeerConnection.prototype[method] = methodObj[method];
      });
    }
    const modernStatsTypes = {
      inboundrtp: "inbound-rtp",
      outboundrtp: "outbound-rtp",
      candidatepair: "candidate-pair",
      localcandidate: "local-candidate",
      remotecandidate: "remote-candidate",
    };
    const nativeGetStats = window2.RTCPeerConnection.prototype.getStats;
    window2.RTCPeerConnection.prototype.getStats = function getStats() {
      const [selector, onSucc, onErr] = arguments;
      return nativeGetStats
        .apply(this, [selector || null])
        .then((stats) => {
          if (browserDetails.version < 53 && !onSucc) {
            try {
              stats.forEach((stat) => {
                stat.type = modernStatsTypes[stat.type] || stat.type;
              });
            } catch (e) {
              if (e.name !== "TypeError") {
                throw e;
              }
              stats.forEach((stat, i) => {
                stats.set(
                  i,
                  Object.assign({}, stat, {
                    type: modernStatsTypes[stat.type] || stat.type,
                  })
                );
              });
            }
          }
          return stats;
        })
        .then(onSucc, onErr);
    };
  }
  function shimSenderGetStats(window2) {
    if (
      !(
        typeof window2 === "object" &&
        window2.RTCPeerConnection &&
        window2.RTCRtpSender
      )
    ) {
      return;
    }
    if (window2.RTCRtpSender && "getStats" in window2.RTCRtpSender.prototype) {
      return;
    }
    const origGetSenders = window2.RTCPeerConnection.prototype.getSenders;
    if (origGetSenders) {
      window2.RTCPeerConnection.prototype.getSenders = function getSenders() {
        const senders = origGetSenders.apply(this, []);
        senders.forEach((sender) => (sender._pc = this));
        return senders;
      };
    }
    const origAddTrack = window2.RTCPeerConnection.prototype.addTrack;
    if (origAddTrack) {
      window2.RTCPeerConnection.prototype.addTrack = function addTrack() {
        const sender = origAddTrack.apply(this, arguments);
        sender._pc = this;
        return sender;
      };
    }
    window2.RTCRtpSender.prototype.getStats = function getStats() {
      return this.track
        ? this._pc.getStats(this.track)
        : Promise.resolve(/* @__PURE__ */ new Map());
    };
  }
  function shimReceiverGetStats(window2) {
    if (
      !(
        typeof window2 === "object" &&
        window2.RTCPeerConnection &&
        window2.RTCRtpSender
      )
    ) {
      return;
    }
    if (
      window2.RTCRtpSender &&
      "getStats" in window2.RTCRtpReceiver.prototype
    ) {
      return;
    }
    const origGetReceivers = window2.RTCPeerConnection.prototype.getReceivers;
    if (origGetReceivers) {
      window2.RTCPeerConnection.prototype.getReceivers =
        function getReceivers() {
          const receivers = origGetReceivers.apply(this, []);
          receivers.forEach((receiver) => (receiver._pc = this));
          return receivers;
        };
    }
    wrapPeerConnectionEvent(window2, "track", (e) => {
      e.receiver._pc = e.srcElement;
      return e;
    });
    window2.RTCRtpReceiver.prototype.getStats = function getStats() {
      return this._pc.getStats(this.track);
    };
  }
  function shimRemoveStream(window2) {
    if (
      !window2.RTCPeerConnection ||
      "removeStream" in window2.RTCPeerConnection.prototype
    ) {
      return;
    }
    window2.RTCPeerConnection.prototype.removeStream = function removeStream(
      stream
    ) {
      deprecated("removeStream", "removeTrack");
      this.getSenders().forEach((sender) => {
        if (sender.track && stream.getTracks().includes(sender.track)) {
          this.removeTrack(sender);
        }
      });
    };
  }
  function shimRTCDataChannel(window2) {
    if (window2.DataChannel && !window2.RTCDataChannel) {
      window2.RTCDataChannel = window2.DataChannel;
    }
  }
  function shimAddTransceiver(window2) {
    if (!(typeof window2 === "object" && window2.RTCPeerConnection)) {
      return;
    }
    const origAddTransceiver =
      window2.RTCPeerConnection.prototype.addTransceiver;
    if (origAddTransceiver) {
      window2.RTCPeerConnection.prototype.addTransceiver =
        function addTransceiver() {
          this.setParametersPromises = [];
          let sendEncodings = arguments[1] && arguments[1].sendEncodings;
          if (sendEncodings === void 0) {
            sendEncodings = [];
          }
          sendEncodings = [...sendEncodings];
          const shouldPerformCheck = sendEncodings.length > 0;
          if (shouldPerformCheck) {
            sendEncodings.forEach((encodingParam) => {
              if ("rid" in encodingParam) {
                const ridRegex = /^[a-z0-9]{0,16}$/i;
                if (!ridRegex.test(encodingParam.rid)) {
                  throw new TypeError("Invalid RID value provided.");
                }
              }
              if ("scaleResolutionDownBy" in encodingParam) {
                if (!(parseFloat(encodingParam.scaleResolutionDownBy) >= 1)) {
                  throw new RangeError(
                    "scale_resolution_down_by must be >= 1.0"
                  );
                }
              }
              if ("maxFramerate" in encodingParam) {
                if (!(parseFloat(encodingParam.maxFramerate) >= 0)) {
                  throw new RangeError("max_framerate must be >= 0.0");
                }
              }
            });
          }
          const transceiver = origAddTransceiver.apply(this, arguments);
          if (shouldPerformCheck) {
            const { sender } = transceiver;
            const params = sender.getParameters();
            if (
              !("encodings" in params) || // Avoid being fooled by patched getParameters() below.
              (params.encodings.length === 1 &&
                Object.keys(params.encodings[0]).length === 0)
            ) {
              params.encodings = sendEncodings;
              sender.sendEncodings = sendEncodings;
              this.setParametersPromises.push(
                sender
                  .setParameters(params)
                  .then(() => {
                    delete sender.sendEncodings;
                  })
                  .catch(() => {
                    delete sender.sendEncodings;
                  })
              );
            }
          }
          return transceiver;
        };
    }
  }
  function shimGetParameters(window2) {
    if (!(typeof window2 === "object" && window2.RTCRtpSender)) {
      return;
    }
    const origGetParameters = window2.RTCRtpSender.prototype.getParameters;
    if (origGetParameters) {
      window2.RTCRtpSender.prototype.getParameters = function getParameters() {
        const params = origGetParameters.apply(this, arguments);
        if (!("encodings" in params)) {
          params.encodings = [].concat(this.sendEncodings || [{}]);
        }
        return params;
      };
    }
  }
  function shimCreateOffer(window2) {
    if (!(typeof window2 === "object" && window2.RTCPeerConnection)) {
      return;
    }
    const origCreateOffer = window2.RTCPeerConnection.prototype.createOffer;
    window2.RTCPeerConnection.prototype.createOffer = function createOffer() {
      if (this.setParametersPromises && this.setParametersPromises.length) {
        return Promise.all(this.setParametersPromises)
          .then(() => {
            return origCreateOffer.apply(this, arguments);
          })
          .finally(() => {
            this.setParametersPromises = [];
          });
      }
      return origCreateOffer.apply(this, arguments);
    };
  }
  function shimCreateAnswer(window2) {
    if (!(typeof window2 === "object" && window2.RTCPeerConnection)) {
      return;
    }
    const origCreateAnswer = window2.RTCPeerConnection.prototype.createAnswer;
    window2.RTCPeerConnection.prototype.createAnswer = function createAnswer() {
      if (this.setParametersPromises && this.setParametersPromises.length) {
        return Promise.all(this.setParametersPromises)
          .then(() => {
            return origCreateAnswer.apply(this, arguments);
          })
          .finally(() => {
            this.setParametersPromises = [];
          });
      }
      return origCreateAnswer.apply(this, arguments);
    };
  }

  // node_modules/webrtc-adapter/src/js/safari/safari_shim.js
  var safari_shim_exports = {};
  __export(safari_shim_exports, {
    shimAudioContext: () => shimAudioContext,
    shimCallbacksAPI: () => shimCallbacksAPI,
    shimConstraints: () => shimConstraints,
    shimCreateOfferLegacy: () => shimCreateOfferLegacy,
    shimGetUserMedia: () => shimGetUserMedia3,
    shimLocalStreamsAPI: () => shimLocalStreamsAPI,
    shimRTCIceServerUrls: () => shimRTCIceServerUrls,
    shimRemoteStreamsAPI: () => shimRemoteStreamsAPI,
    shimTrackEventTransceiver: () => shimTrackEventTransceiver,
  });
  function shimLocalStreamsAPI(window2) {
    if (typeof window2 !== "object" || !window2.RTCPeerConnection) {
      return;
    }
    if (!("getLocalStreams" in window2.RTCPeerConnection.prototype)) {
      window2.RTCPeerConnection.prototype.getLocalStreams =
        function getLocalStreams() {
          if (!this._localStreams) {
            this._localStreams = [];
          }
          return this._localStreams;
        };
    }
    if (!("addStream" in window2.RTCPeerConnection.prototype)) {
      const _addTrack = window2.RTCPeerConnection.prototype.addTrack;
      window2.RTCPeerConnection.prototype.addStream = function addStream(
        stream
      ) {
        if (!this._localStreams) {
          this._localStreams = [];
        }
        if (!this._localStreams.includes(stream)) {
          this._localStreams.push(stream);
        }
        stream
          .getAudioTracks()
          .forEach((track) => _addTrack.call(this, track, stream));
        stream
          .getVideoTracks()
          .forEach((track) => _addTrack.call(this, track, stream));
      };
      window2.RTCPeerConnection.prototype.addTrack = function addTrack(
        track,
        ...streams
      ) {
        if (streams) {
          streams.forEach((stream) => {
            if (!this._localStreams) {
              this._localStreams = [stream];
            } else if (!this._localStreams.includes(stream)) {
              this._localStreams.push(stream);
            }
          });
        }
        return _addTrack.apply(this, arguments);
      };
    }
    if (!("removeStream" in window2.RTCPeerConnection.prototype)) {
      window2.RTCPeerConnection.prototype.removeStream = function removeStream(
        stream
      ) {
        if (!this._localStreams) {
          this._localStreams = [];
        }
        const index = this._localStreams.indexOf(stream);
        if (index === -1) {
          return;
        }
        this._localStreams.splice(index, 1);
        const tracks = stream.getTracks();
        this.getSenders().forEach((sender) => {
          if (tracks.includes(sender.track)) {
            this.removeTrack(sender);
          }
        });
      };
    }
  }
  function shimRemoteStreamsAPI(window2) {
    if (typeof window2 !== "object" || !window2.RTCPeerConnection) {
      return;
    }
    if (!("getRemoteStreams" in window2.RTCPeerConnection.prototype)) {
      window2.RTCPeerConnection.prototype.getRemoteStreams =
        function getRemoteStreams() {
          return this._remoteStreams ? this._remoteStreams : [];
        };
    }
    if (!("onaddstream" in window2.RTCPeerConnection.prototype)) {
      Object.defineProperty(
        window2.RTCPeerConnection.prototype,
        "onaddstream",
        {
          get() {
            return this._onaddstream;
          },
          set(f) {
            if (this._onaddstream) {
              this.removeEventListener("addstream", this._onaddstream);
              this.removeEventListener("track", this._onaddstreampoly);
            }
            this.addEventListener("addstream", (this._onaddstream = f));
            this.addEventListener(
              "track",
              (this._onaddstreampoly = (e) => {
                e.streams.forEach((stream) => {
                  if (!this._remoteStreams) {
                    this._remoteStreams = [];
                  }
                  if (this._remoteStreams.includes(stream)) {
                    return;
                  }
                  this._remoteStreams.push(stream);
                  const event = new Event("addstream");
                  event.stream = stream;
                  this.dispatchEvent(event);
                });
              })
            );
          },
        }
      );
      const origSetRemoteDescription =
        window2.RTCPeerConnection.prototype.setRemoteDescription;
      window2.RTCPeerConnection.prototype.setRemoteDescription =
        function setRemoteDescription() {
          const pc = this;
          if (!this._onaddstreampoly) {
            this.addEventListener(
              "track",
              (this._onaddstreampoly = function (e) {
                e.streams.forEach((stream) => {
                  if (!pc._remoteStreams) {
                    pc._remoteStreams = [];
                  }
                  if (pc._remoteStreams.indexOf(stream) >= 0) {
                    return;
                  }
                  pc._remoteStreams.push(stream);
                  const event = new Event("addstream");
                  event.stream = stream;
                  pc.dispatchEvent(event);
                });
              })
            );
          }
          return origSetRemoteDescription.apply(pc, arguments);
        };
    }
  }
  function shimCallbacksAPI(window2) {
    if (typeof window2 !== "object" || !window2.RTCPeerConnection) {
      return;
    }
    const prototype = window2.RTCPeerConnection.prototype;
    const origCreateOffer = prototype.createOffer;
    const origCreateAnswer = prototype.createAnswer;
    const setLocalDescription = prototype.setLocalDescription;
    const setRemoteDescription = prototype.setRemoteDescription;
    const addIceCandidate = prototype.addIceCandidate;
    prototype.createOffer = function createOffer(
      successCallback,
      failureCallback
    ) {
      const options = arguments.length >= 2 ? arguments[2] : arguments[0];
      const promise = origCreateOffer.apply(this, [options]);
      if (!failureCallback) {
        return promise;
      }
      promise.then(successCallback, failureCallback);
      return Promise.resolve();
    };
    prototype.createAnswer = function createAnswer(
      successCallback,
      failureCallback
    ) {
      const options = arguments.length >= 2 ? arguments[2] : arguments[0];
      const promise = origCreateAnswer.apply(this, [options]);
      if (!failureCallback) {
        return promise;
      }
      promise.then(successCallback, failureCallback);
      return Promise.resolve();
    };
    let withCallback = function (
      description,
      successCallback,
      failureCallback
    ) {
      const promise = setLocalDescription.apply(this, [description]);
      if (!failureCallback) {
        return promise;
      }
      promise.then(successCallback, failureCallback);
      return Promise.resolve();
    };
    prototype.setLocalDescription = withCallback;
    withCallback = function (description, successCallback, failureCallback) {
      const promise = setRemoteDescription.apply(this, [description]);
      if (!failureCallback) {
        return promise;
      }
      promise.then(successCallback, failureCallback);
      return Promise.resolve();
    };
    prototype.setRemoteDescription = withCallback;
    withCallback = function (candidate, successCallback, failureCallback) {
      const promise = addIceCandidate.apply(this, [candidate]);
      if (!failureCallback) {
        return promise;
      }
      promise.then(successCallback, failureCallback);
      return Promise.resolve();
    };
    prototype.addIceCandidate = withCallback;
  }
  function shimGetUserMedia3(window2) {
    const navigator2 = window2 && window2.navigator;
    if (navigator2.mediaDevices && navigator2.mediaDevices.getUserMedia) {
      const mediaDevices = navigator2.mediaDevices;
      const _getUserMedia = mediaDevices.getUserMedia.bind(mediaDevices);
      navigator2.mediaDevices.getUserMedia = (constraints) => {
        return _getUserMedia(shimConstraints(constraints));
      };
    }
    if (
      !navigator2.getUserMedia &&
      navigator2.mediaDevices &&
      navigator2.mediaDevices.getUserMedia
    ) {
      navigator2.getUserMedia = function getUserMedia(constraints, cb, errcb) {
        navigator2.mediaDevices.getUserMedia(constraints).then(cb, errcb);
      }.bind(navigator2);
    }
  }
  function shimConstraints(constraints) {
    if (constraints && constraints.video !== void 0) {
      return Object.assign({}, constraints, {
        video: compactObject(constraints.video),
      });
    }
    return constraints;
  }
  function shimRTCIceServerUrls(window2) {
    if (!window2.RTCPeerConnection) {
      return;
    }
    const OrigPeerConnection = window2.RTCPeerConnection;
    window2.RTCPeerConnection = function RTCPeerConnection2(
      pcConfig,
      pcConstraints
    ) {
      if (pcConfig && pcConfig.iceServers) {
        const newIceServers = [];
        for (let i = 0; i < pcConfig.iceServers.length; i++) {
          let server = pcConfig.iceServers[i];
          if (!server.hasOwnProperty("urls") && server.hasOwnProperty("url")) {
            deprecated("RTCIceServer.url", "RTCIceServer.urls");
            server = JSON.parse(JSON.stringify(server));
            server.urls = server.url;
            delete server.url;
            newIceServers.push(server);
          } else {
            newIceServers.push(pcConfig.iceServers[i]);
          }
        }
        pcConfig.iceServers = newIceServers;
      }
      return new OrigPeerConnection(pcConfig, pcConstraints);
    };
    window2.RTCPeerConnection.prototype = OrigPeerConnection.prototype;
    if ("generateCertificate" in OrigPeerConnection) {
      Object.defineProperty(window2.RTCPeerConnection, "generateCertificate", {
        get() {
          return OrigPeerConnection.generateCertificate;
        },
      });
    }
  }
  function shimTrackEventTransceiver(window2) {
    if (
      typeof window2 === "object" &&
      window2.RTCTrackEvent &&
      "receiver" in window2.RTCTrackEvent.prototype &&
      !("transceiver" in window2.RTCTrackEvent.prototype)
    ) {
      Object.defineProperty(window2.RTCTrackEvent.prototype, "transceiver", {
        get() {
          return { receiver: this.receiver };
        },
      });
    }
  }
  function shimCreateOfferLegacy(window2) {
    const origCreateOffer = window2.RTCPeerConnection.prototype.createOffer;
    window2.RTCPeerConnection.prototype.createOffer = function createOffer(
      offerOptions
    ) {
      if (offerOptions) {
        if (typeof offerOptions.offerToReceiveAudio !== "undefined") {
          offerOptions.offerToReceiveAudio = !!offerOptions.offerToReceiveAudio;
        }
        const audioTransceiver = this.getTransceivers().find(
          (transceiver) => transceiver.receiver.track.kind === "audio"
        );
        if (offerOptions.offerToReceiveAudio === false && audioTransceiver) {
          if (audioTransceiver.direction === "sendrecv") {
            if (audioTransceiver.setDirection) {
              audioTransceiver.setDirection("sendonly");
            } else {
              audioTransceiver.direction = "sendonly";
            }
          } else if (audioTransceiver.direction === "recvonly") {
            if (audioTransceiver.setDirection) {
              audioTransceiver.setDirection("inactive");
            } else {
              audioTransceiver.direction = "inactive";
            }
          }
        } else if (
          offerOptions.offerToReceiveAudio === true &&
          !audioTransceiver
        ) {
          this.addTransceiver("audio", { direction: "recvonly" });
        }
        if (typeof offerOptions.offerToReceiveVideo !== "undefined") {
          offerOptions.offerToReceiveVideo = !!offerOptions.offerToReceiveVideo;
        }
        const videoTransceiver = this.getTransceivers().find(
          (transceiver) => transceiver.receiver.track.kind === "video"
        );
        if (offerOptions.offerToReceiveVideo === false && videoTransceiver) {
          if (videoTransceiver.direction === "sendrecv") {
            if (videoTransceiver.setDirection) {
              videoTransceiver.setDirection("sendonly");
            } else {
              videoTransceiver.direction = "sendonly";
            }
          } else if (videoTransceiver.direction === "recvonly") {
            if (videoTransceiver.setDirection) {
              videoTransceiver.setDirection("inactive");
            } else {
              videoTransceiver.direction = "inactive";
            }
          }
        } else if (
          offerOptions.offerToReceiveVideo === true &&
          !videoTransceiver
        ) {
          this.addTransceiver("video", { direction: "recvonly" });
        }
      }
      return origCreateOffer.apply(this, arguments);
    };
  }
  function shimAudioContext(window2) {
    if (typeof window2 !== "object" || window2.AudioContext) {
      return;
    }
    window2.AudioContext = window2.webkitAudioContext;
  }

  // node_modules/webrtc-adapter/src/js/common_shim.js
  var common_shim_exports = {};
  __export(common_shim_exports, {
    removeExtmapAllowMixed: () => removeExtmapAllowMixed,
    shimAddIceCandidateNullOrEmpty: () => shimAddIceCandidateNullOrEmpty,
    shimConnectionState: () => shimConnectionState,
    shimMaxMessageSize: () => shimMaxMessageSize,
    shimParameterlessSetLocalDescription: () =>
      shimParameterlessSetLocalDescription,
    shimRTCIceCandidate: () => shimRTCIceCandidate,
    shimRTCIceCandidateRelayProtocol: () => shimRTCIceCandidateRelayProtocol,
    shimSendThrowTypeError: () => shimSendThrowTypeError,
  });
  var import_sdp = __toESM(require_sdp());
  function shimRTCIceCandidate(window2) {
    if (
      !window2.RTCIceCandidate ||
      (window2.RTCIceCandidate &&
        "foundation" in window2.RTCIceCandidate.prototype)
    ) {
      return;
    }
    const NativeRTCIceCandidate = window2.RTCIceCandidate;
    window2.RTCIceCandidate = function RTCIceCandidate2(args) {
      if (
        typeof args === "object" &&
        args.candidate &&
        args.candidate.indexOf("a=") === 0
      ) {
        args = JSON.parse(JSON.stringify(args));
        args.candidate = args.candidate.substr(2);
      }
      if (args.candidate && args.candidate.length) {
        const nativeCandidate = new NativeRTCIceCandidate(args);
        const parsedCandidate = import_sdp.default.parseCandidate(
          args.candidate
        );
        const augmentedCandidate = Object.assign(
          nativeCandidate,
          parsedCandidate
        );
        augmentedCandidate.toJSON = function toJSON() {
          return {
            candidate: augmentedCandidate.candidate,
            sdpMid: augmentedCandidate.sdpMid,
            sdpMLineIndex: augmentedCandidate.sdpMLineIndex,
            usernameFragment: augmentedCandidate.usernameFragment,
          };
        };
        return augmentedCandidate;
      }
      return new NativeRTCIceCandidate(args);
    };
    window2.RTCIceCandidate.prototype = NativeRTCIceCandidate.prototype;
    wrapPeerConnectionEvent(window2, "icecandidate", (e) => {
      if (e.candidate) {
        Object.defineProperty(e, "candidate", {
          value: new window2.RTCIceCandidate(e.candidate),
          writable: "false",
        });
      }
      return e;
    });
  }
  function shimRTCIceCandidateRelayProtocol(window2) {
    if (
      !window2.RTCIceCandidate ||
      (window2.RTCIceCandidate &&
        "relayProtocol" in window2.RTCIceCandidate.prototype)
    ) {
      return;
    }
    wrapPeerConnectionEvent(window2, "icecandidate", (e) => {
      if (e.candidate) {
        const parsedCandidate = import_sdp.default.parseCandidate(
          e.candidate.candidate
        );
        if (parsedCandidate.type === "relay") {
          e.candidate.relayProtocol = {
            0: "tls",
            1: "tcp",
            2: "udp",
          }[parsedCandidate.priority >> 24];
        }
      }
      return e;
    });
  }
  function shimMaxMessageSize(window2, browserDetails) {
    if (!window2.RTCPeerConnection) {
      return;
    }
    if (!("sctp" in window2.RTCPeerConnection.prototype)) {
      Object.defineProperty(window2.RTCPeerConnection.prototype, "sctp", {
        get() {
          return typeof this._sctp === "undefined" ? null : this._sctp;
        },
      });
    }
    const sctpInDescription = function (description) {
      if (!description || !description.sdp) {
        return false;
      }
      const sections = import_sdp.default.splitSections(description.sdp);
      sections.shift();
      return sections.some((mediaSection) => {
        const mLine = import_sdp.default.parseMLine(mediaSection);
        return (
          mLine &&
          mLine.kind === "application" &&
          mLine.protocol.indexOf("SCTP") !== -1
        );
      });
    };
    const getRemoteFirefoxVersion = function (description) {
      const match = description.sdp.match(/mozilla...THIS_IS_SDPARTA-(\d+)/);
      if (match === null || match.length < 2) {
        return -1;
      }
      const version2 = parseInt(match[1], 10);
      return version2 !== version2 ? -1 : version2;
    };
    const getCanSendMaxMessageSize = function (remoteIsFirefox) {
      let canSendMaxMessageSize = 65536;
      if (browserDetails.browser === "firefox") {
        if (browserDetails.version < 57) {
          if (remoteIsFirefox === -1) {
            canSendMaxMessageSize = 16384;
          } else {
            canSendMaxMessageSize = 2147483637;
          }
        } else if (browserDetails.version < 60) {
          canSendMaxMessageSize = browserDetails.version === 57 ? 65535 : 65536;
        } else {
          canSendMaxMessageSize = 2147483637;
        }
      }
      return canSendMaxMessageSize;
    };
    const getMaxMessageSize = function (description, remoteIsFirefox) {
      let maxMessageSize = 65536;
      if (
        browserDetails.browser === "firefox" &&
        browserDetails.version === 57
      ) {
        maxMessageSize = 65535;
      }
      const match = import_sdp.default.matchPrefix(
        description.sdp,
        "a=max-message-size:"
      );
      if (match.length > 0) {
        maxMessageSize = parseInt(match[0].substr(19), 10);
      } else if (
        browserDetails.browser === "firefox" &&
        remoteIsFirefox !== -1
      ) {
        maxMessageSize = 2147483637;
      }
      return maxMessageSize;
    };
    const origSetRemoteDescription =
      window2.RTCPeerConnection.prototype.setRemoteDescription;
    window2.RTCPeerConnection.prototype.setRemoteDescription =
      function setRemoteDescription() {
        this._sctp = null;
        if (
          browserDetails.browser === "chrome" &&
          browserDetails.version >= 76
        ) {
          const { sdpSemantics } = this.getConfiguration();
          if (sdpSemantics === "plan-b") {
            Object.defineProperty(this, "sctp", {
              get() {
                return typeof this._sctp === "undefined" ? null : this._sctp;
              },
              enumerable: true,
              configurable: true,
            });
          }
        }
        if (sctpInDescription(arguments[0])) {
          const isFirefox = getRemoteFirefoxVersion(arguments[0]);
          const canSendMMS = getCanSendMaxMessageSize(isFirefox);
          const remoteMMS = getMaxMessageSize(arguments[0], isFirefox);
          let maxMessageSize;
          if (canSendMMS === 0 && remoteMMS === 0) {
            maxMessageSize = Number.POSITIVE_INFINITY;
          } else if (canSendMMS === 0 || remoteMMS === 0) {
            maxMessageSize = Math.max(canSendMMS, remoteMMS);
          } else {
            maxMessageSize = Math.min(canSendMMS, remoteMMS);
          }
          const sctp = {};
          Object.defineProperty(sctp, "maxMessageSize", {
            get() {
              return maxMessageSize;
            },
          });
          this._sctp = sctp;
        }
        return origSetRemoteDescription.apply(this, arguments);
      };
  }
  function shimSendThrowTypeError(window2) {
    if (
      !(
        window2.RTCPeerConnection &&
        "createDataChannel" in window2.RTCPeerConnection.prototype
      )
    ) {
      return;
    }
    function wrapDcSend(dc, pc) {
      const origDataChannelSend = dc.send;
      dc.send = function send() {
        const data = arguments[0];
        const length = data.length || data.size || data.byteLength;
        if (
          dc.readyState === "open" &&
          pc.sctp &&
          length > pc.sctp.maxMessageSize
        ) {
          throw new TypeError(
            "Message too large (can send a maximum of " +
              pc.sctp.maxMessageSize +
              " bytes)"
          );
        }
        return origDataChannelSend.apply(dc, arguments);
      };
    }
    const origCreateDataChannel =
      window2.RTCPeerConnection.prototype.createDataChannel;
    window2.RTCPeerConnection.prototype.createDataChannel =
      function createDataChannel() {
        const dataChannel = origCreateDataChannel.apply(this, arguments);
        wrapDcSend(dataChannel, this);
        return dataChannel;
      };
    wrapPeerConnectionEvent(window2, "datachannel", (e) => {
      wrapDcSend(e.channel, e.target);
      return e;
    });
  }
  function shimConnectionState(window2) {
    if (
      !window2.RTCPeerConnection ||
      "connectionState" in window2.RTCPeerConnection.prototype
    ) {
      return;
    }
    const proto = window2.RTCPeerConnection.prototype;
    Object.defineProperty(proto, "connectionState", {
      get() {
        return (
          {
            completed: "connected",
            checking: "connecting",
          }[this.iceConnectionState] || this.iceConnectionState
        );
      },
      enumerable: true,
      configurable: true,
    });
    Object.defineProperty(proto, "onconnectionstatechange", {
      get() {
        return this._onconnectionstatechange || null;
      },
      set(cb) {
        if (this._onconnectionstatechange) {
          this.removeEventListener(
            "connectionstatechange",
            this._onconnectionstatechange
          );
          delete this._onconnectionstatechange;
        }
        if (cb) {
          this.addEventListener(
            "connectionstatechange",
            (this._onconnectionstatechange = cb)
          );
        }
      },
      enumerable: true,
      configurable: true,
    });
    ["setLocalDescription", "setRemoteDescription"].forEach((method) => {
      const origMethod = proto[method];
      proto[method] = function () {
        if (!this._connectionstatechangepoly) {
          this._connectionstatechangepoly = (e) => {
            const pc = e.target;
            if (pc._lastConnectionState !== pc.connectionState) {
              pc._lastConnectionState = pc.connectionState;
              const newEvent = new Event("connectionstatechange", e);
              pc.dispatchEvent(newEvent);
            }
            return e;
          };
          this.addEventListener(
            "iceconnectionstatechange",
            this._connectionstatechangepoly
          );
        }
        return origMethod.apply(this, arguments);
      };
    });
  }
  function removeExtmapAllowMixed(window2, browserDetails) {
    if (!window2.RTCPeerConnection) {
      return;
    }
    if (browserDetails.browser === "chrome" && browserDetails.version >= 71) {
      return;
    }
    if (browserDetails.browser === "safari" && browserDetails.version >= 605) {
      return;
    }
    const nativeSRD = window2.RTCPeerConnection.prototype.setRemoteDescription;
    window2.RTCPeerConnection.prototype.setRemoteDescription =
      function setRemoteDescription(desc) {
        if (
          desc &&
          desc.sdp &&
          desc.sdp.indexOf("\na=extmap-allow-mixed") !== -1
        ) {
          const sdp2 = desc.sdp
            .split("\n")
            .filter((line) => {
              return line.trim() !== "a=extmap-allow-mixed";
            })
            .join("\n");
          if (
            window2.RTCSessionDescription &&
            desc instanceof window2.RTCSessionDescription
          ) {
            arguments[0] = new window2.RTCSessionDescription({
              type: desc.type,
              sdp: sdp2,
            });
          } else {
            desc.sdp = sdp2;
          }
        }
        return nativeSRD.apply(this, arguments);
      };
  }
  function shimAddIceCandidateNullOrEmpty(window2, browserDetails) {
    if (!(window2.RTCPeerConnection && window2.RTCPeerConnection.prototype)) {
      return;
    }
    const nativeAddIceCandidate =
      window2.RTCPeerConnection.prototype.addIceCandidate;
    if (!nativeAddIceCandidate || nativeAddIceCandidate.length === 0) {
      return;
    }
    window2.RTCPeerConnection.prototype.addIceCandidate =
      function addIceCandidate() {
        if (!arguments[0]) {
          if (arguments[1]) {
            arguments[1].apply(null);
          }
          return Promise.resolve();
        }
        if (
          ((browserDetails.browser === "chrome" &&
            browserDetails.version < 78) ||
            (browserDetails.browser === "firefox" &&
              browserDetails.version < 68) ||
            browserDetails.browser === "safari") &&
          arguments[0] &&
          arguments[0].candidate === ""
        ) {
          return Promise.resolve();
        }
        return nativeAddIceCandidate.apply(this, arguments);
      };
  }
  function shimParameterlessSetLocalDescription(window2, browserDetails) {
    if (!(window2.RTCPeerConnection && window2.RTCPeerConnection.prototype)) {
      return;
    }
    const nativeSetLocalDescription =
      window2.RTCPeerConnection.prototype.setLocalDescription;
    if (!nativeSetLocalDescription || nativeSetLocalDescription.length === 0) {
      return;
    }
    window2.RTCPeerConnection.prototype.setLocalDescription =
      function setLocalDescription() {
        let desc = arguments[0] || {};
        if (typeof desc !== "object" || (desc.type && desc.sdp)) {
          return nativeSetLocalDescription.apply(this, arguments);
        }
        desc = { type: desc.type, sdp: desc.sdp };
        if (!desc.type) {
          switch (this.signalingState) {
            case "stable":
            case "have-local-offer":
            case "have-remote-pranswer":
              desc.type = "offer";
              break;
            default:
              desc.type = "answer";
              break;
          }
        }
        if (desc.sdp || (desc.type !== "offer" && desc.type !== "answer")) {
          return nativeSetLocalDescription.apply(this, [desc]);
        }
        const func =
          desc.type === "offer" ? this.createOffer : this.createAnswer;
        return func
          .apply(this)
          .then((d) => nativeSetLocalDescription.apply(this, [d]));
      };
  }

  // node_modules/webrtc-adapter/src/js/adapter_factory.js
  var sdp = __toESM(require_sdp());
  function adapterFactory(
    { window: window2 } = {},
    options = {
      shimChrome: true,
      shimFirefox: true,
      shimSafari: true,
    }
  ) {
    const logging2 = log;
    const browserDetails = detectBrowser(window2);
    const adapter2 = {
      browserDetails,
      commonShim: common_shim_exports,
      extractVersion,
      disableLog,
      disableWarnings,
      // Expose sdp as a convenience. For production apps include directly.
      sdp,
    };
    switch (browserDetails.browser) {
      case "chrome":
        if (
          !chrome_shim_exports ||
          !shimPeerConnection ||
          !options.shimChrome
        ) {
          logging2("Chrome shim is not included in this adapter release.");
          return adapter2;
        }
        if (browserDetails.version === null) {
          logging2("Chrome shim can not determine version, not shimming.");
          return adapter2;
        }
        logging2("adapter.js shimming chrome.");
        adapter2.browserShim = chrome_shim_exports;
        shimAddIceCandidateNullOrEmpty(window2, browserDetails);
        shimParameterlessSetLocalDescription(window2, browserDetails);
        shimGetUserMedia(window2, browserDetails);
        shimMediaStream(window2, browserDetails);
        shimPeerConnection(window2, browserDetails);
        shimOnTrack(window2, browserDetails);
        shimAddTrackRemoveTrack(window2, browserDetails);
        shimGetSendersWithDtmf(window2, browserDetails);
        shimGetStats(window2, browserDetails);
        shimSenderReceiverGetStats(window2, browserDetails);
        fixNegotiationNeeded(window2, browserDetails);
        shimRTCIceCandidate(window2, browserDetails);
        shimRTCIceCandidateRelayProtocol(window2, browserDetails);
        shimConnectionState(window2, browserDetails);
        shimMaxMessageSize(window2, browserDetails);
        shimSendThrowTypeError(window2, browserDetails);
        removeExtmapAllowMixed(window2, browserDetails);
        break;
      case "firefox":
        if (
          !firefox_shim_exports ||
          !shimPeerConnection2 ||
          !options.shimFirefox
        ) {
          logging2("Firefox shim is not included in this adapter release.");
          return adapter2;
        }
        logging2("adapter.js shimming firefox.");
        adapter2.browserShim = firefox_shim_exports;
        shimAddIceCandidateNullOrEmpty(window2, browserDetails);
        shimParameterlessSetLocalDescription(window2, browserDetails);
        shimGetUserMedia2(window2, browserDetails);
        shimPeerConnection2(window2, browserDetails);
        shimOnTrack2(window2, browserDetails);
        shimRemoveStream(window2, browserDetails);
        shimSenderGetStats(window2, browserDetails);
        shimReceiverGetStats(window2, browserDetails);
        shimRTCDataChannel(window2, browserDetails);
        shimAddTransceiver(window2, browserDetails);
        shimGetParameters(window2, browserDetails);
        shimCreateOffer(window2, browserDetails);
        shimCreateAnswer(window2, browserDetails);
        shimRTCIceCandidate(window2, browserDetails);
        shimConnectionState(window2, browserDetails);
        shimMaxMessageSize(window2, browserDetails);
        shimSendThrowTypeError(window2, browserDetails);
        break;
      case "safari":
        if (!safari_shim_exports || !options.shimSafari) {
          logging2("Safari shim is not included in this adapter release.");
          return adapter2;
        }
        logging2("adapter.js shimming safari.");
        adapter2.browserShim = safari_shim_exports;
        shimAddIceCandidateNullOrEmpty(window2, browserDetails);
        shimParameterlessSetLocalDescription(window2, browserDetails);
        shimRTCIceServerUrls(window2, browserDetails);
        shimCreateOfferLegacy(window2, browserDetails);
        shimCallbacksAPI(window2, browserDetails);
        shimLocalStreamsAPI(window2, browserDetails);
        shimRemoteStreamsAPI(window2, browserDetails);
        shimTrackEventTransceiver(window2, browserDetails);
        shimGetUserMedia3(window2, browserDetails);
        shimAudioContext(window2, browserDetails);
        shimRTCIceCandidate(window2, browserDetails);
        shimRTCIceCandidateRelayProtocol(window2, browserDetails);
        shimMaxMessageSize(window2, browserDetails);
        shimSendThrowTypeError(window2, browserDetails);
        removeExtmapAllowMixed(window2, browserDetails);
        break;
      default:
        logging2("Unsupported browser!");
        break;
    }
    return adapter2;
  }

  // node_modules/webrtc-adapter/src/js/adapter_core.js
  var adapter = adapterFactory({
    window: typeof window === "undefined" ? void 0 : window,
  });
  var adapter_core_default = adapter;

  // lib/supports.ts
  var webRTCAdapter =
    //@ts-ignore
    adapter_core_default.default || adapter_core_default;
  var Supports = new (class {
    isIOS = ["iPad", "iPhone", "iPod"].includes(navigator.platform);
    supportedBrowsers = ["firefox", "chrome", "safari"];
    minFirefoxVersion = 59;
    minChromeVersion = 72;
    minSafariVersion = 605;
    isWebRTCSupported() {
      return typeof RTCPeerConnection !== "undefined";
    }
    isBrowserSupported() {
      const browser = this.getBrowser();
      const version2 = this.getVersion();
      const validBrowser = this.supportedBrowsers.includes(browser);
      if (!validBrowser) return false;
      if (browser === "chrome") return version2 >= this.minChromeVersion;
      if (browser === "firefox") return version2 >= this.minFirefoxVersion;
      if (browser === "safari")
        return !this.isIOS && version2 >= this.minSafariVersion;
      return false;
    }
    getBrowser() {
      return webRTCAdapter.browserDetails.browser;
    }
    getVersion() {
      return webRTCAdapter.browserDetails.version || 0;
    }
    isUnifiedPlanSupported() {
      const browser = this.getBrowser();
      const version2 = webRTCAdapter.browserDetails.version || 0;
      if (browser === "chrome" && version2 < this.minChromeVersion)
        return false;
      if (browser === "firefox" && version2 >= this.minFirefoxVersion)
        return true;
      if (
        !window.RTCRtpTransceiver ||
        !("currentDirection" in RTCRtpTransceiver.prototype)
      )
        return false;
      let tempPc;
      let supported = false;
      try {
        tempPc = new RTCPeerConnection();
        tempPc.addTransceiver("audio");
        supported = true;
      } catch (e) {
      } finally {
        if (tempPc) {
          tempPc.close();
        }
      }
      return supported;
    }
    toString() {
      return `Supports:
    browser:${this.getBrowser()}
    version:${this.getVersion()}
    isIOS:${this.isIOS}
    isWebRTCSupported:${this.isWebRTCSupported()}
    isBrowserSupported:${this.isBrowserSupported()}
    isUnifiedPlanSupported:${this.isUnifiedPlanSupported()}`;
    }
  })();

  // lib/util.ts
  var DEFAULT_CONFIG = {
    iceServers: [
      { urls: "stun:stun.l.google.com:19302" },
      {
        urls: [
          "turn:eu-0.turn.peerjs.com:3478",
          "turn:us-0.turn.peerjs.com:3478",
        ],
        username: "peerjs",
        credential: "peerjsp",
      },
    ],
    sdpSemantics: "unified-plan",
  };
  var Util = class {
    noop() {}
    CLOUD_HOST = "0.peerjs.com";
    CLOUD_PORT = 443;
    // Browsers that need chunking:
    chunkedBrowsers = { Chrome: 1, chrome: 1 };
    chunkedMTU = 16300;
    // The original 60000 bytes setting does not work when sending data from Firefox to Chrome, which is "cut off" after 16384 bytes and delivered individually.
    // Returns browser-agnostic default config
    defaultConfig = DEFAULT_CONFIG;
    browser = Supports.getBrowser();
    browserVersion = Supports.getVersion();
    // Lists which features are supported
    supports = (function () {
      const supported = {
        browser: Supports.isBrowserSupported(),
        webRTC: Supports.isWebRTCSupported(),
        audioVideo: false,
        data: false,
        binaryBlob: false,
        reliable: false,
      };
      if (!supported.webRTC) return supported;
      let pc;
      try {
        pc = new RTCPeerConnection(DEFAULT_CONFIG);
        supported.audioVideo = true;
        let dc;
        try {
          dc = pc.createDataChannel("_PEERJSTEST", { ordered: true });
          supported.data = true;
          supported.reliable = !!dc.ordered;
          try {
            dc.binaryType = "blob";
            supported.binaryBlob = !Supports.isIOS;
          } catch (e) {}
        } catch (e) {
        } finally {
          if (dc) {
            dc.close();
          }
        }
      } catch (e) {
      } finally {
        if (pc) {
          pc.close();
        }
      }
      return supported;
    })();
    // Ensure alphanumeric ids
    validateId(id) {
      return !id || /^[A-Za-z0-9]+(?:[ _-][A-Za-z0-9]+)*$/.test(id);
    }
    pack = import_peerjs_js_binarypack.default.pack;
    unpack = import_peerjs_js_binarypack.default.unpack;
    // Binary stuff
    _dataCount = 1;
    chunk(blob) {
      const chunks = [];
      const size = blob.size;
      const total = Math.ceil(size / util.chunkedMTU);
      let index = 0;
      let start = 0;
      while (start < size) {
        const end = Math.min(size, start + util.chunkedMTU);
        const b = blob.slice(start, end);
        const chunk = {
          __peerData: this._dataCount,
          n: index,
          data: b,
          total,
        };
        chunks.push(chunk);
        start = end;
        index++;
      }
      this._dataCount++;
      return chunks;
    }
    blobToArrayBuffer(blob, cb) {
      const fr = new FileReader();
      fr.onload = function (evt) {
        if (evt.target) {
          cb(evt.target.result);
        }
      };
      fr.readAsArrayBuffer(blob);
      return fr;
    }
    binaryStringToArrayBuffer(binary) {
      const byteArray = new Uint8Array(binary.length);
      for (let i = 0; i < binary.length; i++) {
        byteArray[i] = binary.charCodeAt(i) & 255;
      }
      return byteArray.buffer;
    }
    randomToken() {
      return Math.random().toString(36).slice(2);
    }
    isSecure() {
      return location.protocol === "https:";
    }
  };
  var util = new Util();

  // lib/peer.ts
  var import_eventemitter34 = __toESM(require_eventemitter3());

  // lib/logger.ts
  var LOG_PREFIX = "PeerJS: ";
  var Logger = class {
    _logLevel = 0 /* Disabled */;
    get logLevel() {
      return this._logLevel;
    }
    set logLevel(logLevel) {
      this._logLevel = logLevel;
    }
    log(...args) {
      if (this._logLevel >= 3 /* All */) {
        this._print(3 /* All */, ...args);
      }
    }
    warn(...args) {
      if (this._logLevel >= 2 /* Warnings */) {
        this._print(2 /* Warnings */, ...args);
      }
    }
    error(...args) {
      if (this._logLevel >= 1 /* Errors */) {
        this._print(1 /* Errors */, ...args);
      }
    }
    setLogFunction(fn) {
      this._print = fn;
    }
    _print(logLevel, ...rest) {
      const copy = [LOG_PREFIX, ...rest];
      for (let i in copy) {
        if (copy[i] instanceof Error) {
          copy[i] = "(" + copy[i].name + ") " + copy[i].message;
        }
      }
      if (logLevel >= 3 /* All */) {
        console.log(...copy);
      } else if (logLevel >= 2 /* Warnings */) {
        console.warn("WARNING", ...copy);
      } else if (logLevel >= 1 /* Errors */) {
        console.error("ERROR", ...copy);
      }
    }
  };
  var logger_default = new Logger();

  // lib/socket.ts
  var import_eventemitter3 = __toESM(require_eventemitter3());

  // package.json
  var version = "1.4.7";

  // lib/socket.ts
  var Socket = class extends import_eventemitter3.EventEmitter {
    constructor(secure, host, port, path, key, pingInterval = 5e3) {
      super();
      this.pingInterval = pingInterval;
      const wsProtocol = secure ? "wss://" : "ws://";
      this._baseUrl =
        wsProtocol + host + ":" + port + path + "peerjs?key=" + key;
    }
    _disconnected = true;
    _id;
    _messagesQueue = [];
    _socket;
    _wsPingTimer;
    _baseUrl;
    start(id, token) {
      this._id = id;
      const wsUrl = `${this._baseUrl}&id=${id}&token=${token}`;
      if (!!this._socket || !this._disconnected) {
        return;
      }
      this._socket = new WebSocket(wsUrl + "&version=" + version);
      this._disconnected = false;
      this._socket.onmessage = (event) => {
        let data;
        try {
          data = JSON.parse(event.data);
          logger_default.log("Server message received:", data);
        } catch (e) {
          logger_default.log("Invalid server message", event.data);
          return;
        }
        this.emit("message" /* Message */, data);
      };
      this._socket.onclose = (event) => {
        if (this._disconnected) {
          return;
        }
        logger_default.log("Socket closed.", event);
        this._cleanup();
        this._disconnected = true;
        this.emit("disconnected" /* Disconnected */);
      };
      this._socket.onopen = () => {
        if (this._disconnected) {
          return;
        }
        this._sendQueuedMessages();
        logger_default.log("Socket open");
        this._scheduleHeartbeat();
      };
    }
    _scheduleHeartbeat() {
      this._wsPingTimer = setTimeout(() => {
        this._sendHeartbeat();
      }, this.pingInterval);
    }
    _sendHeartbeat() {
      if (!this._wsOpen()) {
        logger_default.log(`Cannot send heartbeat, because socket closed`);
        return;
      }
      const message = JSON.stringify({ type: "HEARTBEAT" /* Heartbeat */ });
      this._socket.send(message);
      this._scheduleHeartbeat();
    }
    /** Is the websocket currently open? */
    _wsOpen() {
      return !!this._socket && this._socket.readyState === 1;
    }
    /** Send queued messages. */
    _sendQueuedMessages() {
      const copiedQueue = [...this._messagesQueue];
      this._messagesQueue = [];
      for (const message of copiedQueue) {
        this.send(message);
      }
    }
    /** Exposed send for DC & Peer. */
    send(data) {
      if (this._disconnected) {
        return;
      }
      if (!this._id) {
        this._messagesQueue.push(data);
        return;
      }
      if (!data.type) {
        this.emit("error" /* Error */, "Invalid message");
        return;
      }
      if (!this._wsOpen()) {
        return;
      }
      const message = JSON.stringify(data);
      this._socket.send(message);
    }
    close() {
      if (this._disconnected) {
        return;
      }
      this._cleanup();
      this._disconnected = true;
    }
    _cleanup() {
      if (this._socket) {
        this._socket.onopen =
          this._socket.onmessage =
          this._socket.onclose =
            null;
        this._socket.close();
        this._socket = void 0;
      }
      clearTimeout(this._wsPingTimer);
    }
  };

  // lib/negotiator.ts
  var Negotiator = class {
    constructor(connection) {
      this.connection = connection;
    }
    /** Returns a PeerConnection object set up correctly (for data, media). */
    startConnection(options) {
      const peerConnection = this._startPeerConnection();
      this.connection.peerConnection = peerConnection;
      if (this.connection.type === "media" /* Media */ && options._stream) {
        this._addTracksToConnection(options._stream, peerConnection);
      }
      if (options.originator) {
        if (this.connection.type === "data" /* Data */) {
          const dataConnection = this.connection;
          const config = { ordered: !!options.reliable };
          const dataChannel = peerConnection.createDataChannel(
            dataConnection.label,
            config
          );
          dataConnection.initialize(dataChannel);
        }
        this._makeOffer();
      } else {
        this.handleSDP("OFFER", options.sdp);
      }
    }
    /** Start a PC. */
    _startPeerConnection() {
      logger_default.log("Creating RTCPeerConnection.");
      const peerConnection = new RTCPeerConnection(
        this.connection.provider.options.config
      );
      this._setupListeners(peerConnection);
      return peerConnection;
    }
    /** Set up various WebRTC listeners. */
    _setupListeners(peerConnection) {
      const peerId = this.connection.peer;
      const connectionId = this.connection.connectionId;
      const connectionType = this.connection.type;
      const provider = this.connection.provider;
      logger_default.log("Listening for ICE candidates.");
      peerConnection.onicecandidate = (evt) => {
        if (!evt.candidate || !evt.candidate.candidate) return;
        logger_default.log(
          `Received ICE candidates for ${peerId}:`,
          evt.candidate
        );
        provider.socket.send({
          type: "CANDIDATE" /* Candidate */,
          payload: {
            candidate: evt.candidate,
            type: connectionType,
            connectionId,
          },
          dst: peerId,
        });
      };
      peerConnection.oniceconnectionstatechange = () => {
        switch (peerConnection.iceConnectionState) {
          case "failed":
            logger_default.log(
              "iceConnectionState is failed, closing connections to " + peerId
            );
            this.connection.emit(
              "error",
              new Error("Negotiation of connection to " + peerId + " failed.")
            );
            this.connection.close();
            break;
          case "closed":
            logger_default.log(
              "iceConnectionState is closed, closing connections to " + peerId
            );
            this.connection.emit(
              "error",
              new Error("Connection to " + peerId + " closed.")
            );
            this.connection.close();
            break;
          case "disconnected":
            logger_default.log(
              "iceConnectionState changed to disconnected on the connection with " +
                peerId
            );
            break;
          case "completed":
            peerConnection.onicecandidate = util.noop;
            break;
        }
        this.connection.emit(
          "iceStateChanged",
          peerConnection.iceConnectionState
        );
      };
      logger_default.log("Listening for data channel");
      peerConnection.ondatachannel = (evt) => {
        logger_default.log("Received data channel");
        const dataChannel = evt.channel;
        const connection = provider.getConnection(peerId, connectionId);
        connection.initialize(dataChannel);
      };
      logger_default.log("Listening for remote stream");
      peerConnection.ontrack = (evt) => {
        logger_default.log("Received remote stream");
        const stream = evt.streams[0];
        const connection = provider.getConnection(peerId, connectionId);
        if (connection.type === "media" /* Media */) {
          const mediaConnection = connection;
          this._addStreamToMediaConnection(stream, mediaConnection);
        }
      };
    }
    cleanup() {
      logger_default.log(
        "Cleaning up PeerConnection to " + this.connection.peer
      );
      const peerConnection = this.connection.peerConnection;
      if (!peerConnection) {
        return;
      }
      this.connection.peerConnection = null;
      peerConnection.onicecandidate =
        peerConnection.oniceconnectionstatechange =
        peerConnection.ondatachannel =
        peerConnection.ontrack =
          () => {};
      const peerConnectionNotClosed =
        peerConnection.signalingState !== "closed";
      let dataChannelNotClosed = false;
      if (this.connection.type === "data" /* Data */) {
        const dataConnection = this.connection;
        const dataChannel = dataConnection.dataChannel;
        if (dataChannel) {
          dataChannelNotClosed =
            !!dataChannel.readyState && dataChannel.readyState !== "closed";
        }
      }
      if (peerConnectionNotClosed || dataChannelNotClosed) {
        peerConnection.close();
      }
    }
    async _makeOffer() {
      const peerConnection = this.connection.peerConnection;
      const provider = this.connection.provider;
      try {
        const offer = await peerConnection.createOffer(
          this.connection.options.constraints
        );
        logger_default.log("Created offer.");
        if (
          this.connection.options.sdpTransform &&
          typeof this.connection.options.sdpTransform === "function"
        ) {
          offer.sdp =
            this.connection.options.sdpTransform(offer.sdp) || offer.sdp;
        }
        try {
          await peerConnection.setLocalDescription(offer);
          logger_default.log(
            "Set localDescription:",
            offer,
            `for:${this.connection.peer}`
          );
          let payload = {
            sdp: offer,
            type: this.connection.type,
            connectionId: this.connection.connectionId,
            metadata: this.connection.metadata,
            browser: util.browser,
          };
          if (this.connection.type === "data" /* Data */) {
            const dataConnection = this.connection;
            payload = {
              ...payload,
              label: dataConnection.label,
              reliable: dataConnection.reliable,
              serialization: dataConnection.serialization,
            };
          }
          provider.socket.send({
            type: "OFFER" /* Offer */,
            payload,
            dst: this.connection.peer,
          });
        } catch (err) {
          if (
            err !=
            "OperationError: Failed to set local offer sdp: Called in wrong state: kHaveRemoteOffer"
          ) {
            provider.emitError("webrtc" /* WebRTC */, err);
            logger_default.log("Failed to setLocalDescription, ", err);
          }
        }
      } catch (err_1) {
        provider.emitError("webrtc" /* WebRTC */, err_1);
        logger_default.log("Failed to createOffer, ", err_1);
      }
    }
    async _makeAnswer() {
      const peerConnection = this.connection.peerConnection;
      const provider = this.connection.provider;
      try {
        const answer = await peerConnection.createAnswer();
        logger_default.log("Created answer.");
        if (
          this.connection.options.sdpTransform &&
          typeof this.connection.options.sdpTransform === "function"
        ) {
          answer.sdp =
            this.connection.options.sdpTransform(answer.sdp) || answer.sdp;
        }
        try {
          await peerConnection.setLocalDescription(answer);
          logger_default.log(
            `Set localDescription:`,
            answer,
            `for:${this.connection.peer}`
          );
          provider.socket.send({
            type: "ANSWER" /* Answer */,
            payload: {
              sdp: answer,
              type: this.connection.type,
              connectionId: this.connection.connectionId,
              browser: util.browser,
            },
            dst: this.connection.peer,
          });
        } catch (err) {
          provider.emitError("webrtc" /* WebRTC */, err);
          logger_default.log("Failed to setLocalDescription, ", err);
        }
      } catch (err_1) {
        provider.emitError("webrtc" /* WebRTC */, err_1);
        logger_default.log("Failed to create answer, ", err_1);
      }
    }
    /** Handle an SDP. */
    async handleSDP(type, sdp2) {
      sdp2 = new RTCSessionDescription(sdp2);
      const peerConnection = this.connection.peerConnection;
      const provider = this.connection.provider;
      logger_default.log("Setting remote description", sdp2);
      const self = this;
      try {
        await peerConnection.setRemoteDescription(sdp2);
        logger_default.log(
          `Set remoteDescription:${type} for:${this.connection.peer}`
        );
        if (type === "OFFER") {
          await self._makeAnswer();
        }
      } catch (err) {
        provider.emitError("webrtc" /* WebRTC */, err);
        logger_default.log("Failed to setRemoteDescription, ", err);
      }
    }
    /** Handle a candidate. */
    async handleCandidate(ice) {
      logger_default.log(`handleCandidate:`, ice);
      const candidate = ice.candidate;
      const sdpMLineIndex = ice.sdpMLineIndex;
      const sdpMid = ice.sdpMid;
      const peerConnection = this.connection.peerConnection;
      const provider = this.connection.provider;
      try {
        await peerConnection.addIceCandidate(
          new RTCIceCandidate({
            sdpMid,
            sdpMLineIndex,
            candidate,
          })
        );
        logger_default.log(`Added ICE candidate for:${this.connection.peer}`);
      } catch (err) {
        provider.emitError("webrtc" /* WebRTC */, err);
        logger_default.log("Failed to handleCandidate, ", err);
      }
    }
    _addTracksToConnection(stream, peerConnection) {
      logger_default.log(
        `add tracks from stream ${stream.id} to peer connection`
      );
      if (!peerConnection.addTrack) {
        return logger_default.error(
          `Your browser does't support RTCPeerConnection#addTrack. Ignored.`
        );
      }
      stream.getTracks().forEach((track) => {
        peerConnection.addTrack(track, stream);
      });
    }
    _addStreamToMediaConnection(stream, mediaConnection) {
      logger_default.log(
        `add stream ${stream.id} to media connection ${mediaConnection.connectionId}`
      );
      mediaConnection.addStream(stream);
    }
  };

  // lib/baseconnection.ts
  var import_eventemitter32 = __toESM(require_eventemitter3());
  var BaseConnection = class extends import_eventemitter32.EventEmitter {
    constructor(peer, provider, options) {
      super();
      this.peer = peer;
      this.provider = provider;
      this.options = options;
      this.metadata = options.metadata;
    }
    _open = false;
    metadata;
    connectionId;
    peerConnection;
    get open() {
      return this._open;
    }
  };

  // lib/mediaconnection.ts
  var _MediaConnection = class extends BaseConnection {
    _negotiator;
    _localStream;
    _remoteStream;
    get type() {
      return "media" /* Media */;
    }
    get localStream() {
      return this._localStream;
    }
    get remoteStream() {
      return this._remoteStream;
    }
    constructor(peerId, provider, options) {
      super(peerId, provider, options);
      this._localStream = this.options._stream;
      this.connectionId =
        this.options.connectionId ||
        _MediaConnection.ID_PREFIX + util.randomToken();
      this._negotiator = new Negotiator(this);
      if (this._localStream) {
        this._negotiator.startConnection({
          _stream: this._localStream,
          originator: true,
        });
      }
    }
    addStream(remoteStream) {
      logger_default.log("Receiving stream", remoteStream);
      this._remoteStream = remoteStream;
      super.emit("stream", remoteStream);
    }
    handleMessage(message) {
      const type = message.type;
      const payload = message.payload;
      switch (message.type) {
        case "ANSWER" /* Answer */:
          this._negotiator.handleSDP(type, payload.sdp);
          this._open = true;
          break;
        case "CANDIDATE" /* Candidate */:
          this._negotiator.handleCandidate(payload.candidate);
          break;
        default:
          logger_default.warn(
            `Unrecognized message type:${type} from peer:${this.peer}`
          );
          break;
      }
    }
    answer(stream, options = {}) {
      if (this._localStream) {
        logger_default.warn(
          "Local stream already exists on this MediaConnection. Are you answering a call twice?"
        );
        return;
      }
      this._localStream = stream;
      if (options && options.sdpTransform) {
        this.options.sdpTransform = options.sdpTransform;
      }
      this._negotiator.startConnection({
        ...this.options._payload,
        _stream: stream,
      });
      const messages = this.provider._getMessages(this.connectionId);
      for (let message of messages) {
        this.handleMessage(message);
      }
      this._open = true;
    }
    /**
     * Exposed functionality for users.
     */
    /** Allows user to close connection. */
    close() {
      if (this._negotiator) {
        this._negotiator.cleanup();
        this._negotiator = null;
      }
      this._localStream = null;
      this._remoteStream = null;
      if (this.provider) {
        this.provider._removeConnection(this);
        this.provider = null;
      }
      if (this.options && this.options._stream) {
        this.options._stream = null;
      }
      if (!this.open) return;
      this._open = false;
      super.emit("close");
    }
  };
  var MediaConnection = _MediaConnection;
  __publicField(MediaConnection, "ID_PREFIX", "mc_");

  // lib/encodingQueue.ts
  var import_eventemitter33 = __toESM(require_eventemitter3());
  var EncodingQueue = class extends import_eventemitter33.EventEmitter {
    fileReader = new FileReader();
    _queue = [];
    _processing = false;
    constructor() {
      super();
      this.fileReader.onload = (evt) => {
        this._processing = false;
        if (evt.target) {
          this.emit("done", evt.target.result);
        }
        this.doNextTask();
      };
      this.fileReader.onerror = (evt) => {
        logger_default.error(`EncodingQueue error:`, evt);
        this._processing = false;
        this.destroy();
        this.emit("error", evt);
      };
    }
    get queue() {
      return this._queue;
    }
    get size() {
      return this.queue.length;
    }
    get processing() {
      return this._processing;
    }
    enque(blob) {
      this.queue.push(blob);
      if (this.processing) return;
      this.doNextTask();
    }
    destroy() {
      this.fileReader.abort();
      this._queue = [];
    }
    doNextTask() {
      if (this.size === 0) return;
      if (this.processing) return;
      this._processing = true;
      this.fileReader.readAsArrayBuffer(this.queue.shift());
    }
  };

  // lib/dataconnection.ts
  var _DataConnection = class extends BaseConnection {
    _negotiator;
    label;
    serialization;
    reliable;
    stringify = JSON.stringify;
    parse = JSON.parse;
    get type() {
      return "data" /* Data */;
    }
    _buffer = [];
    _bufferSize = 0;
    _buffering = false;
    _chunkedData = {};
    _dc;
    _encodingQueue = new EncodingQueue();
    get dataChannel() {
      return this._dc;
    }
    get bufferSize() {
      return this._bufferSize;
    }
    constructor(peerId, provider, options) {
      super(peerId, provider, options);
      this.connectionId =
        this.options.connectionId ||
        _DataConnection.ID_PREFIX + util.randomToken();
      this.label = this.options.label || this.connectionId;
      this.serialization = this.options.serialization || "binary" /* Binary */;
      this.reliable = !!this.options.reliable;
      this._encodingQueue.on("done", (ab) => {
        this._bufferedSend(ab);
      });
      this._encodingQueue.on("error", () => {
        logger_default.error(
          `DC#${this.connectionId}: Error occured in encoding from blob to arraybuffer, close DC`
        );
        this.close();
      });
      this._negotiator = new Negotiator(this);
      this._negotiator.startConnection(
        this.options._payload || {
          originator: true,
        }
      );
    }
    /** Called by the Negotiator when the DataChannel is ready. */
    initialize(dc) {
      this._dc = dc;
      this._configureDataChannel();
    }
    _configureDataChannel() {
      if (!util.supports.binaryBlob || util.supports.reliable) {
        this.dataChannel.binaryType = "arraybuffer";
      }
      this.dataChannel.onopen = () => {
        logger_default.log(`DC#${this.connectionId} dc connection success`);
        this._open = true;
        this.emit("open");
      };
      this.dataChannel.onmessage = (e) => {
        logger_default.log(`DC#${this.connectionId} dc onmessage:`, e.data);
        this._handleDataMessage(e);
      };
      this.dataChannel.onclose = () => {
        logger_default.log(`DC#${this.connectionId} dc closed for:`, this.peer);
        this.close();
      };
    }
    // Handles a DataChannel message.
    _handleDataMessage({ data }) {
      const datatype = data.constructor;
      const isBinarySerialization =
        this.serialization === "binary" /* Binary */ ||
        this.serialization === "binary-utf8"; /* BinaryUTF8 */
      let deserializedData = data;
      if (isBinarySerialization) {
        if (datatype === Blob) {
          util.blobToArrayBuffer(data, (ab) => {
            const unpackedData = util.unpack(ab);
            this.emit("data", unpackedData);
          });
          return;
        } else if (datatype === ArrayBuffer) {
          deserializedData = util.unpack(data);
        } else if (datatype === String) {
          const ab = util.binaryStringToArrayBuffer(data);
          deserializedData = util.unpack(ab);
        }
      } else if (this.serialization === "json" /* JSON */) {
        deserializedData = this.parse(data);
      }
      if (deserializedData.__peerData) {
        this._handleChunk(deserializedData);
        return;
      }
      super.emit("data", deserializedData);
    }
    _handleChunk(data) {
      const id = data.__peerData;
      const chunkInfo = this._chunkedData[id] || {
        data: [],
        count: 0,
        total: data.total,
      };
      chunkInfo.data[data.n] = data.data;
      chunkInfo.count++;
      this._chunkedData[id] = chunkInfo;
      if (chunkInfo.total === chunkInfo.count) {
        delete this._chunkedData[id];
        const data2 = new Blob(chunkInfo.data);
        this._handleDataMessage({ data: data2 });
      }
    }
    /**
     * Exposed functionality for users.
     */
    /** Allows user to close connection. */
    close() {
      this._buffer = [];
      this._bufferSize = 0;
      this._chunkedData = {};
      if (this._negotiator) {
        this._negotiator.cleanup();
        this._negotiator = null;
      }
      if (this.provider) {
        this.provider._removeConnection(this);
        this.provider = null;
      }
      if (this.dataChannel) {
        this.dataChannel.onopen = null;
        this.dataChannel.onmessage = null;
        this.dataChannel.onclose = null;
        this._dc = null;
      }
      if (this._encodingQueue) {
        this._encodingQueue.destroy();
        this._encodingQueue.removeAllListeners();
        this._encodingQueue = null;
      }
      if (!this.open) return;
      this._open = false;
      super.emit("close");
    }
    /** Allows user to send data. */
    send(data, chunked) {
      if (!this.open) {
        super.emit(
          "error",
          new Error(
            "Connection is not open. You should listen for the `open` event before sending messages."
          )
        );
        return;
      }
      if (this.serialization === "json" /* JSON */) {
        this._bufferedSend(this.stringify(data));
      } else if (
        this.serialization === "binary" /* Binary */ ||
        this.serialization === "binary-utf8" /* BinaryUTF8 */
      ) {
        const blob = util.pack(data);
        if (!chunked && blob.size > util.chunkedMTU) {
          this._sendChunks(blob);
          return;
        }
        if (!util.supports.binaryBlob) {
          this._encodingQueue.enque(blob);
        } else {
          this._bufferedSend(blob);
        }
      } else {
        this._bufferedSend(data);
      }
    }
    _bufferedSend(msg) {
      if (this._buffering || !this._trySend(msg)) {
        this._buffer.push(msg);
        this._bufferSize = this._buffer.length;
      }
    }
    // Returns true if the send succeeds.
    _trySend(msg) {
      if (!this.open) return false;
      if (
        this.dataChannel.bufferedAmount > _DataConnection.MAX_BUFFERED_AMOUNT
      ) {
        this._buffering = true;
        setTimeout(() => {
          this._buffering = false;
          this._tryBuffer();
        }, 50);
        return false;
      }
      try {
        this.dataChannel.send(msg);
      } catch (e) {
        logger_default.error(`DC#:${this.connectionId} Error when sending:`, e);
        this._buffering = true;
        this.close();
        return false;
      }
      return true;
    }
    // Try to send the first message in the buffer.
    _tryBuffer() {
      if (!this.open) return;
      if (this._buffer.length === 0) return;
      const msg = this._buffer[0];
      if (this._trySend(msg)) {
        this._buffer.shift();
        this._bufferSize = this._buffer.length;
        this._tryBuffer();
      }
    }
    _sendChunks(blob) {
      const blobs = util.chunk(blob);
      logger_default.log(
        `DC#${this.connectionId} Try to send ${blobs.length} chunks...`
      );
      for (let blob2 of blobs) this.send(blob2, true);
    }
    handleMessage(message) {
      const payload = message.payload;
      switch (message.type) {
        case "ANSWER" /* Answer */:
          this._negotiator.handleSDP(message.type, payload.sdp);
          break;
        case "CANDIDATE" /* Candidate */:
          this._negotiator.handleCandidate(payload.candidate);
          break;
        default:
          logger_default.warn(
            "Unrecognized message type:",
            message.type,
            "from peer:",
            this.peer
          );
          break;
      }
    }
  };
  var DataConnection = _DataConnection;
  __publicField(DataConnection, "ID_PREFIX", "dc_");
  __publicField(DataConnection, "MAX_BUFFERED_AMOUNT", 8 * 1024 * 1024);

  // lib/api.ts
  var API = class {
    constructor(_options) {
      this._options = _options;
    }
    _buildRequest(method) {
      const protocol = this._options.secure ? "https" : "http";
      const { host, port, path, key } = this._options;
      const url = new URL(
        `${protocol}://${host}:${port}${path}${key}/${method}`
      );
      url.searchParams.set("ts", `${Date.now()}${Math.random()}`);
      url.searchParams.set("version", version);
      return fetch(url.href, {
        referrerPolicy: this._options.referrerPolicy,
      });
    }
    /** Get a unique ID from the server via XHR and initialize with it. */
    async retrieveId() {
      try {
        const response = await this._buildRequest("id");
        if (response.status !== 200)
          throw new Error(`Error. Status:${response.status}`);
        return response.text();
      } catch (error) {
        logger_default.error("Error retrieving ID", error);
        let pathError = "";
        if (
          this._options.path === "/" &&
          this._options.host !== util.CLOUD_HOST
        ) {
          pathError =
            " If you passed in a `path` to your self-hosted PeerServer, you'll also need to pass in that same path when creating a new Peer.";
        }
        throw new Error("Could not get an ID from the server." + pathError);
      }
    }
    /** @deprecated */
    async listAllPeers() {
      try {
        const response = await this._buildRequest("peers");
        if (response.status !== 200) {
          if (response.status === 401) {
            let helpfulError = "";
            if (this._options.host === util.CLOUD_HOST) {
              helpfulError =
                "It looks like you're using the cloud server. You can email team@peerjs.com to enable peer listing for your API key.";
            } else {
              helpfulError =
                "You need to enable `allow_discovery` on your self-hosted PeerServer to use this feature.";
            }
            throw new Error(
              "It doesn't look like you have permission to list peers IDs. " +
                helpfulError
            );
          }
          throw new Error(`Error. Status:${response.status}`);
        }
        return response.json();
      } catch (error) {
        logger_default.error("Error retrieving list peers", error);
        throw new Error("Could not get list peers from the server." + error);
      }
    }
  };

  // lib/peer.ts
  var _Peer = class extends import_eventemitter34.EventEmitter {
    _options;
    _api;
    _socket;
    _id = null;
    _lastServerId = null;
    // States.
    _destroyed = false;
    // Connections have been killed
    _disconnected = false;
    // Connection to PeerServer killed but P2P connections still active
    _open = false;
    // Sockets and such are not yet open.
    _connections = /* @__PURE__ */ new Map();
    // All connections for this peer.
    _lostMessages = /* @__PURE__ */ new Map();
    // src => [list of messages]
    /**
     * The brokering ID of this peer
     */
    get id() {
      return this._id;
    }
    get options() {
      return this._options;
    }
    get open() {
      return this._open;
    }
    get socket() {
      return this._socket;
    }
    /**
     * A hash of all connections associated with this peer, keyed by the remote peer's ID.
     * @deprecated
     * Return type will change from Object to Map<string,[]>
     */
    get connections() {
      const plainConnections = /* @__PURE__ */ Object.create(null);
      for (let [k, v] of this._connections) {
        plainConnections[k] = v;
      }
      return plainConnections;
    }
    /**
     * true if this peer and all of its connections can no longer be used.
     */
    get destroyed() {
      return this._destroyed;
    }
    /**
     * false if there is an active connection to the PeerServer.
     */
    get disconnected() {
      return this._disconnected;
    }
    constructor(id, options) {
      super();
      let userId;
      if (id && id.constructor == Object) options = id;
      else if (id) userId = id.toString();
      options = {
        debug: 0,
        // 1: Errors, 2: Warnings, 3: All logs
        host: util.CLOUD_HOST,
        port: util.CLOUD_PORT,
        path: "/",
        key: _Peer.DEFAULT_KEY,
        token: util.randomToken(),
        config: util.defaultConfig,
        referrerPolicy: "strict-origin-when-cross-origin",
        ...options,
      };
      this._options = options;
      if (this._options.host === "/") {
        this._options.host = window.location.hostname;
      }
      if (this._options.path) {
        if (this._options.path[0] !== "/") {
          this._options.path = "/" + this._options.path;
        }
        if (this._options.path[this._options.path.length - 1] !== "/") {
          this._options.path += "/";
        }
      }
      if (
        this._options.secure === void 0 &&
        this._options.host !== util.CLOUD_HOST
      ) {
        this._options.secure = util.isSecure();
      } else if (this._options.host == util.CLOUD_HOST) {
        this._options.secure = true;
      }
      if (this._options.logFunction) {
        logger_default.setLogFunction(this._options.logFunction);
      }
      logger_default.logLevel = this._options.debug || 0;
      this._api = new API(options);
      this._socket = this._createServerConnection();
      if (!util.supports.audioVideo && !util.supports.data) {
        this._delayedAbort(
          "browser-incompatible" /* BrowserIncompatible */,
          "The current browser does not support WebRTC"
        );
        return;
      }
      if (!!userId && !util.validateId(userId)) {
        this._delayedAbort(
          "invalid-id" /* InvalidID */,
          `ID "${userId}" is invalid`
        );
        return;
      }
      if (userId) {
        this._initialize(userId);
      } else {
        this._api
          .retrieveId()
          .then((id2) => this._initialize(id2))
          .catch((error) =>
            this._abort("server-error" /* ServerError */, error)
          );
      }
    }
    _createServerConnection() {
      const socket = new Socket(
        this._options.secure,
        this._options.host,
        this._options.port,
        this._options.path,
        this._options.key,
        this._options.pingInterval
      );
      socket.on("message" /* Message */, (data) => {
        this._handleMessage(data);
      });
      socket.on("error" /* Error */, (error) => {
        this._abort("socket-error" /* SocketError */, error);
      });
      socket.on("disconnected" /* Disconnected */, () => {
        if (this.disconnected) return;
        this.emitError("network" /* Network */, "Lost connection to server.");
        this.disconnect();
      });
      socket.on("close" /* Close */, () => {
        if (this.disconnected) return;
        this._abort(
          "socket-closed" /* SocketClosed */,
          "Underlying socket is already closed."
        );
      });
      return socket;
    }
    /** Initialize a connection with the server. */
    _initialize(id) {
      this._id = id;
      this.socket.start(id, this._options.token);
    }
    /** Handles messages from the server. */
    _handleMessage(message) {
      const type = message.type;
      const payload = message.payload;
      const peerId = message.src;
      switch (type) {
        case "OPEN" /* Open */:
          this._lastServerId = this.id;
          this._open = true;
          this.emit("open", this.id);
          break;
        case "ERROR" /* Error */:
          this._abort("server-error" /* ServerError */, payload.msg);
          break;
        case "ID-TAKEN" /* IdTaken */:
          this._abort(
            "unavailable-id" /* UnavailableID */,
            `ID "${this.id}" is taken`
          );
          break;
        case "INVALID-KEY" /* InvalidKey */:
          this._abort(
            "invalid-key" /* InvalidKey */,
            `API KEY "${this._options.key}" is invalid`
          );
          break;
        case "LEAVE" /* Leave */:
          logger_default.log(`Received leave message from ${peerId}`);
          this._cleanupPeer(peerId);
          this._connections.delete(peerId);
          break;
        case "EXPIRE" /* Expire */:
          this.emitError(
            "peer-unavailable" /* PeerUnavailable */,
            `Could not connect to peer ${peerId}`
          );
          break;
        case "OFFER" /* Offer */: {
          const connectionId = payload.connectionId;
          let connection = this.getConnection(peerId, connectionId);
          if (connection) {
            connection.close();
            logger_default.warn(
              `Offer received for existing Connection ID:${connectionId}`
            );
          }
          if (payload.type === "media" /* Media */) {
            const mediaConnection = new MediaConnection(peerId, this, {
              connectionId,
              _payload: payload,
              metadata: payload.metadata,
            });
            connection = mediaConnection;
            this._addConnection(peerId, connection);
            this.emit("call", mediaConnection);
          } else if (payload.type === "data" /* Data */) {
            const dataConnection = new DataConnection(peerId, this, {
              connectionId,
              _payload: payload,
              metadata: payload.metadata,
              label: payload.label,
              serialization: payload.serialization,
              reliable: payload.reliable,
            });
            connection = dataConnection;
            this._addConnection(peerId, connection);
            this.emit("connection", dataConnection);
          } else {
            logger_default.warn(
              `Received malformed connection type:${payload.type}`
            );
            return;
          }
          const messages = this._getMessages(connectionId);
          for (let message2 of messages) connection.handleMessage(message2);

          break;
        }
        default: {
          if (!payload) {
            logger_default.warn(
              `You received a malformed message from ${peerId} of type ${type}`
            );
            return;
          }
          const connectionId = payload.connectionId;
          const connection = this.getConnection(peerId, connectionId);
          if (connection && connection.peerConnection) {
            connection.handleMessage(message);
          } else if (connectionId) {
            this._storeMessage(connectionId, message);
          } else {
            logger_default.warn(
              "You received an unrecognized message:",
              message
            );
          }
          break;
        }
      }
    }
    /** Stores messages without a set up connection, to be claimed later. */
    _storeMessage(connectionId, message) {
      if (!this._lostMessages.has(connectionId))
        this._lostMessages.set(connectionId, []);

      this._lostMessages.get(connectionId).push(message);
    }
    /** Retrieve messages from lost message store */
    //TODO Change it to private
    _getMessages(connectionId) {
      const messages = this._lostMessages.get(connectionId);
      if (messages) {
        this._lostMessages.delete(connectionId);
        return messages;
      }
      return [];
    }
    /**
     * Connects to the remote peer specified by id and returns a data connection.
     * @param peer The brokering ID of the remote peer (their peer.id).
     * @param options for specifying details about Peer Connection
     */
    connect(peer, options = {}) {
      if (this.disconnected) {
        logger_default.warn(
          "You cannot connect to a new Peer because you called .disconnect() on this Peer and ended your connection with the server. You can create a new Peer to reconnect, or call reconnect on this peer if you believe its ID to still be available."
        );
        this.emitError(
          "disconnected" /* Disconnected */,
          "Cannot connect to new Peer after disconnecting from server."
        );
        return;
      }
      const dataConnection = new DataConnection(peer, this, options);
      this._addConnection(peer, dataConnection);
      return dataConnection;
    }
    /**
     * Calls the remote peer specified by id and returns a media connection.
     * @param peer The brokering ID of the remote peer (their peer.id).
     * @param stream The caller's media stream
     * @param options Metadata associated with the connection, passed in by whoever initiated the connection.
     */
    call(peer, stream, options = {}) {
      if (this.disconnected) {
        logger_default.warn(
          "You cannot connect to a new Peer because you called .disconnect() on this Peer and ended your connection with the server. You can create a new Peer to reconnect."
        );
        this.emitError(
          "disconnected" /* Disconnected */,
          "Cannot connect to new Peer after disconnecting from server."
        );
        return;
      }
      if (!stream) {
        logger_default.error(
          "To call a peer, you must provide a stream from your browser's `getUserMedia`."
        );
        return;
      }
      const mediaConnection = new MediaConnection(peer, this, {
        ...options,
        _stream: stream,
      });
      this._addConnection(peer, mediaConnection);
      return mediaConnection;
    }
    /** Add a data/media connection to this peer. */
    _addConnection(peerId, connection) {
      logger_default.log(
        `add connection ${connection.type}:${connection.connectionId} to peerId:${peerId}`
      );
      if (!this._connections.has(peerId)) this._connections.set(peerId, []);
      this._connections.get(peerId).push(connection);
    }
    //TODO should be private
    _removeConnection(connection) {
      const connections = this._connections.get(connection.peer);
      if (connections) {
        const index = connections.indexOf(connection);
        if (index !== -1) connections.splice(index, 1);
      }
      this._lostMessages.delete(connection.connectionId);
    }
    /** Retrieve a data/media connection for this peer. */
    getConnection(peerId, connectionId) {
      const connections = this._connections.get(peerId);
      if (!connections) return null;
      for (let connection of connections)
        if (connection.connectionId === connectionId) return connection;
      return null;
    }
    _delayedAbort(type, message) {
      setTimeout(() => {
        this._abort(type, message);
      }, 0);
    }
    /**
     * Emits an error message and destroys the Peer.
     * The Peer is not destroyed if it's in a disconnected state, in which case
     * it retains its disconnected state and its existing connections.
     */
    _abort(type, message) {
      logger_default.error("Aborting!");
      this.emitError(type, message);
      if (!this._lastServerId) this.destroy();
      else this.disconnect();
    }
    /** Emits a typed error message. */
    emitError(type, err) {
      logger_default.error("Error:", err);
      let error;
      if (typeof err === "string") error = new Error(err);
      else error = err;
      error.type = type;
      this.emit("error", error);
    }
    /**
     * Destroys the Peer: closes all active connections as well as the connection
     *  to the server.
     * Warning: The peer can no longer create or accept connections after being
     *  destroyed.
     */
    destroy() {
      if (this.destroyed) return;

      logger_default.log(`Destroy peer with ID:${this.id}`);
      this.disconnect();
      this._cleanup();
      this._destroyed = true;
      this.emit("close");
    }
    /** Disconnects every connection on this peer. */
    _cleanup() {
      for (let peerId of this._connections.keys()) {
        this._cleanupPeer(peerId);
        this._connections.delete(peerId);
      }
      this.socket.removeAllListeners();
    }
    /** Closes all connections to this peer. */
    _cleanupPeer(peerId) {
      const connections = this._connections.get(peerId);
      if (!connections) return;
      for (let connection of connections) connection.close();
    }
    /**
     * Disconnects the Peer's connection to the PeerServer. Does not close any
     *  active connections.
     * Warning: The peer can no longer create or accept connections after being
     *  disconnected. It also cannot reconnect to the server.
     */
    disconnect() {
      if (this.disconnected) return;
      const currentId = this.id;
      logger_default.log(`Disconnect peer with ID:${currentId}`);
      this._disconnected = true;
      this._open = false;
      this.socket.close();
      this._lastServerId = currentId;
      this._id = null;
      this.emit("disconnected", currentId);
    }
    /** Attempts to reconnect with the same ID. */
    reconnect() {
      if (this.disconnected && !this.destroyed) {
        logger_default.log(
          `Attempting reconnection to server with ID ${this._lastServerId}`
        );
        this._disconnected = false;
        this._initialize(this._lastServerId);
      } else if (this.destroyed) {
        throw new Error(
          "This peer cannot reconnect to the server. It has already been destroyed."
        );
      } else if (!this.disconnected && !this.open) {
        logger_default.error(
          "In a hurry? We're still trying to make the initial connection!"
        );
      } else {
        throw new Error(
          `Peer ${this.id} cannot reconnect because it is not disconnected from the server!`
        );
      }
    }
    /**
     * Get a list of available peer IDs. If you're running your own server, you'll
     * want to set allow_discovery: true in the PeerServer options. If you're using
     * the cloud server, email team@peerjs.com to get the functionality enabled for
     * your key.
     */
    listAllPeers(cb = (_) => {}) {
      this._api
        .listAllPeers()
        .then((peers) => cb(peers))
        .catch((error) => this._abort("server-error" /* ServerError */, error));
    }
  };
  var Peer = _Peer;
  __publicField(Peer, "DEFAULT_KEY", "peerjs");

  // lib/global.ts
  window.peerjs = { Peer, util };
  window.Peer = Peer;
})();
