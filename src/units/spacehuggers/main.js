"use strict";
// https://github.com/KilledByAPixel/SpaceHuggers , commit hash `bdabe47`
/*
~/misc/code/hello-react/node_modules/.bin/esbuild main.js --outfile=out.js --minify --bundle
~/misc/code/hello-react/node_modules/.bin/esbuild main.js --minify --bundle | wc -c
│a100488
lowGraphicsSettings = false;
startCameraScale = 4 * 8;
defaultCameraScale = 4 * 8;
maxWidth = 1024;
maxHeight = 576;
grenadeCount
keep trying until a valid level is generated
*/
let randCur = 76550; // seed
Math.random = () => (randCur = (25214903917 * randCur) & 65535) / 100000;
const GOD_MODE = true;
const FPS = 60;
const TIME_DELTA = 1 / FPS;
const CAMERA_SCALE = 4 * 8;
const MAX_WIDTH = 1024;
const MAX_HEIGHT = 576;

/* LittleJS Utility Classes and Functions */
const PI = Math.PI;
const abs = (a) => (a < 0 ? -a : a);
const sign = (a) => (a < 0 ? -1 : 1);
const min = (a, b) => (a < b ? a : b);
const max = (a, b) => (a > b ? a : b);
const mod = (a, b) => ((a % b) + b) % b;
const clamp = (v, max = 1, min = 0) => (v < min ? min : v > max ? max : v);
const percent = (v, max = 1, min = 0) =>
  max - min ? clamp((v - min) / (max - min)) : 0;
const lerp = (p, max = 1, min = 0) => min + clamp(p) * (max - min);
const formatTime = (t) =>
  ((t / 60) | 0) + ":" + (t % 60 < 10 ? "0" : "") + (t % 60 | 0);
const isOverlapping = (pA, sA, pB, sB) =>
  (abs(pA.x - pB.x) * 2 < sA.x + sB.x) & (abs(pA.y - pB.y) * 2 < sA.y + sB.y);
const rand = (a = 1, b = 0) => b + (a - b) * Math.random();
const randSign = () => (rand(2) | 0) * 2 - 1;
const randInCircle = (radius = 1, minRadius = 0) =>
  radius > 0
    ? randVector(radius * rand(minRadius / radius, 1) ** 0.5)
    : new Vector2();
const randVector = (length = 1) => new Vector2().setAngle(rand(2 * PI), length);
const randColor = (cA = new Color(), cB = new Color(0, 0, 0, 1), linear) =>
  linear
    ? cA.lerp(cB, rand())
    : new Color(
        rand(cA.r, cB.r),
        rand(cA.g, cB.g),
        rand(cA.b, cB.b),
        rand(cA.a, cB.a)
      );

// seeded random numbers - Xorshift
let randSeed = 0;
const sinMono = Array.from(Array(360), (_, i) => Math.sin(i));
const randSeeded = (a = 1.0, b = 0.0) =>
  b + (a - b) * ((sinMono[++randSeed % 360] ** 2.0 * 1e9) % 1.0);

// create a 2d vector, can take another Vector2 to copy, 2 scalars, or 1 scalar
const vec2 = (x = 0, y) =>
  x.x == undefined
    ? new Vector2(x, y == undefined ? x : y)
    : new Vector2(x.x, x.y);

class Vector2 {
  constructor(x = 0, y = 0) {
    this.x = x;
    this.y = y;
  }

  // basic math operators, a vector or scaler can be passed in
  copy() {
    return new Vector2(this.x, this.y);
  }
  scale(s) {
    return new Vector2(this.x * s, this.y * s);
  }
  add(v) {
    return new Vector2(this.x + v.x, this.y + v.y);
  }
  subtract(v) {
    return new Vector2(this.x - v.x, this.y - v.y);
  }
  multiply(v) {
    return new Vector2(this.x * v.x, this.y * v.y);
  }
  divide(v) {
    return new Vector2(this.x / v.x, this.y / v.y);
  }

  // vector math operators
  length() {
    return this.lengthSquared() ** 0.5;
  }
  lengthSquared() {
    return this.x ** 2 + this.y ** 2;
  }
  distance(p) {
    return this.distanceSquared(p) ** 0.5;
  }
  distanceSquared(p) {
    return (this.x - p.x) ** 2 + (this.y - p.y) ** 2;
  }
  normalize(length = 1) {
    const l = this.length();
    return l ? this.scale(length / l) : new Vector2(length);
  }
  angle() {
    return Math.atan2(this.x, this.y);
  }
  setAngle(a, length = 1) {
    this.x = length * Math.sin(a);
    this.y = length * Math.cos(a);
    return this;
  }
  rotate(a) {
    const c = Math.cos(a),
      s = Math.sin(a);
    return new Vector2(this.x * c - this.y * s, this.x * s + this.y * c);
  }
  flip() {
    return new Vector2(this.y, this.x);
  }
  lerp(v, p) {
    return this.add(v.subtract(this).scale(clamp(p)));
  }
  int() {
    return new Vector2(this.x | 0, this.y | 0);
  }
  area() {
    return this.x * this.y;
  }
  arrayCheck(arraySize) {
    return (
      this.x >= 0 && this.y >= 0 && this.x < arraySize.x && this.y < arraySize.y
    );
  }
}

class Color {
  constructor(r = 1, g = 1, b = 1, a = 1) {
    (this.r = r), (this.g = g), (this.b = b), (this.a = a);
  }
  add(c) {
    return new Color(this.r + c.r, this.g + c.g, this.b + c.b, this.a + c.a);
  }
  subtract(c) {
    return new Color(this.r - c.r, this.g - c.g, this.b - c.b, this.a - c.a);
  }
  multiply(c) {
    return new Color(this.r * c.r, this.g * c.g, this.b * c.b, this.a * c.a);
  }
  scale(s, a = s) {
    return new Color(this.r * s, this.g * s, this.b * s, this.a * a);
  }
  clamp() {
    return new Color(
      clamp(this.r),
      clamp(this.g),
      clamp(this.b),
      clamp(this.a)
    );
  }
  lerp(c, p) {
    return this.add(c.subtract(this).scale(clamp(p)));
  }
  mutate(amount = 0.05, alphaAmount = 0) {
    return new Color(
      this.r + rand(amount, -amount),
      this.g + rand(amount, -amount),
      this.b + rand(amount, -amount),
      this.a + rand(alphaAmount, -alphaAmount)
    ).clamp();
  }
  rgba() {
    return `rgb(${(this.r * 255) | 0},${(this.g * 255) | 0},${
      (this.b * 255) | 0
    },${this.a})`;
  }
  rgbaInt() {
    return (
      ((this.r * 255) | 0) +
      ((this.g * 255) << 8) +
      ((this.b * 255) << 16) +
      ((this.a * 255) << 24)
    );
  }
  setHSLA(h = 0, s = 0, l = 1, a = 1) {
    const q = l < 0.5 ? l * (1 + s) : l + s - l * s,
      p = 2 * l - q,
      f = (p, q, t) =>
        (t = ((t % 1) + 1) % 1) < 1 / 6
          ? p + (q - p) * 6 * t
          : t < 1 / 2
          ? q
          : t < 2 / 3
          ? p + (q - p) * (2 / 3 - t) * 6
          : p;

    this.r = f(p, q, h + 1 / 3);
    this.g = f(p, q, h);
    this.b = f(p, q, h - 1 / 3);
    this.a = a;
    return this;
  }
}

class Timer {
  constructor(timeLeft) {
    this.time = timeLeft == undefined ? undefined : time + timeLeft;
    this.setTime = timeLeft;
  }

  set(timeLeft = 0) {
    this.time = time + timeLeft;
    this.setTime = timeLeft;
  }
  unset() {
    this.time = undefined;
  }
  isSet() {
    return this.time != undefined;
  }
  active() {
    return time <= this.time;
  } // is set and has no time left
  elapsed() {
    return time > this.time;
  } // is set and has time left
  get() {
    return this.isSet() ? time - this.time : 0;
  }
  getPercent() {
    return this.isSet() ? percent(this.time - time, 0, this.setTime) : 0;
  }
}

/* LittleJS v0.74 - The Little JavaScript Game Engine That Can - By Frank Force 2021 */
// tile sheet settings
const defaultTileSize = vec2(16); // default size of tiles in pixels
const tileBleedShrinkFix = 0.3; // prevent tile bleeding from neighbors
// core engine
const gravity = -0.01;
let mainCanvas = 0,
  mainContext = 0,
  mainCanvasSize = vec2();
let engineObjects = [],
  engineCollideObjects = [];
let frame = 0,
  time = 0,
  realTime = 0,
  paused = 0,
  frameTimeLastMS = 0,
  frameTimeBufferMS = 0;
let cameraPos = vec2(),
  cameraScale = 4 * max(defaultTileSize.x, defaultTileSize.y);
let tileImageSize, tileImageSizeInverse, shrinkTilesX, shrinkTilesY, drawCount;

const tileImage = new Image(); // the tile image used by everything
function engineInit(
  appInit,
  appUpdate,
  appUpdatePost,
  appRender,
  appRenderPost
) {
  // init engine when tiles load
  tileImage.onload = () => {
    // save tile image info
    tileImageSizeInverse = vec2(1).divide(
      (tileImageSize = vec2(tileImage.width, tileImage.height))
    );
    shrinkTilesX = tileBleedShrinkFix / tileImageSize.x;
    shrinkTilesY = tileBleedShrinkFix / tileImageSize.y;

    // setup html
    document.body.appendChild((mainCanvas = document.createElement("canvas")));
    document.body.style = "margin:0;overflow:hidden;background:#000";
    mainCanvas.style =
      "position:absolute;top:50%;left:50%;transform:translate(-50%,-50%);image-rendering:crisp-edges;image-rendering:pixelated"; // pixelated rendering
    mainContext = mainCanvas.getContext("2d");

    glInit();
    appInit();
    engineUpdate();
  };

  // main update loop
  const engineUpdate = (frameTimeMS = 0) => {
    requestAnimationFrame(engineUpdate);

    if (!document.hasFocus()) inputData[0].length = 0; // clear input when lost focus

    // prepare to update time
    const realFrameTimeDeltaMS = frameTimeMS - frameTimeLastMS;
    let frameTimeDeltaMS = realFrameTimeDeltaMS;
    frameTimeLastMS = frameTimeMS;
    realTime = frameTimeMS / 1e3;
    if (!paused) frameTimeBufferMS += frameTimeDeltaMS;

    // update frame
    mousePosWorld = screenToWorld(mousePosScreen);
    updateGamepads();

    // apply time delta smoothing, improves smoothness of framerate in some browsers
    let deltaSmooth = 0;
    if (frameTimeBufferMS < 0 && frameTimeBufferMS > -9) {
      // force an update each frame if time is close enough (not just a fast refresh rate)
      deltaSmooth = frameTimeBufferMS;
      frameTimeBufferMS = 0;
    }

    // clamp incase of extra long frames (slow framerate)
    frameTimeBufferMS = min(frameTimeBufferMS, 50);

    // update the frame
    for (; frameTimeBufferMS >= 0; frameTimeBufferMS -= 1e3 / FPS) {
      // main frame update
      appUpdate();
      engineUpdateObjects();
      appUpdatePost();

      // update input
      for (let deviceInputData of inputData)
        deviceInputData.map((k) => (k.r = k.p = 0));
      mouseWheel = 0;
    }

    // add the smoothing back in
    frameTimeBufferMS += deltaSmooth;

    // fill the window
    mainCanvas.width = min(innerWidth, MAX_WIDTH);
    mainCanvas.height = min(innerHeight, MAX_HEIGHT);

    // save canvas size
    mainCanvasSize = vec2(mainCanvas.width, mainCanvas.height);
    mainContext.imageSmoothingEnabled = false; // disable smoothing for pixel art

    // render sort then render while removing destroyed objects
    glPreRender(mainCanvas.width, mainCanvas.height);
    appRender();
    engineObjects.sort((a, b) => a.renderOrder - b.renderOrder);
    for (const o of engineObjects) o.destroyed || o.render();
    glCopyToContext(mainContext, false);
    appRenderPost();
    // copy anything left in the buffer if necessary
    glCopyToContext(mainContext, false);
  };

  //tileImage.src = 'tiles.png';
  tileImage.src = `data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAIAAAABABAMAAAAg+GJMAAAAJ1BMVEUAAAD///+AgID/AAAAAACJT6T/dwBIG11mlD2i91azs7PZ2dlAQEA9UPniAAAAAXRSTlMAQObYZgAAAoRJREFUeNqsk8dZw0AQhWfNiHDbARfgVAB0sNI3LTgcfSLcbasC0o0j6panHIYMT+FX+p8y/UdECJnNJlTn6PJnft4wEvHFasCkis2JakItl9dpemjZxgniywInnhX2hWAxXug0NIwKcduwjRTpF8hVLp7PJDSEuE5X24ofFZAQV7cAEZtDQ4jLJcSKgzuAXD8DDuVDdGOFmWjNaPN0v1zdPtU0BTSIi8cQY60ZPWcQ715r2oIzeqFupBQbZqVY0xY8PT1RN6zIh7QP8TTL+lcwmiE1megkRagif1DgvB8WsAILbQpYgZ2aD8kWsGLMUSybdfMpm1tgDRiJzfr7P5MteGufDpXchoE4jGcKj32C2oBKsM5jlPUlT4VKyN9TdFuSwKsfqrLHTcE5ucQgKL8H+GZW2n0YZptZKUXSXhIUic4nfa4OXrPPNmXTxI8BwzCBFcP6DjGxfwFmmwIwBv548/sccBJqPGRK10uYu5YCtRkD/r3xc8AEELDOk6ZAn3OHXR7h19D8OAckE3ukFiAkyb7V+lPXAqecd/8DYCSAkAGK+r37W3fhDZYCQqg4ICS/HKjNQsACbuYYRHLmHLh1BFfRHikKevd0dwATAQzTW3Dv7n8DKCrsZSF51wLh2hssfCOGQTLoegvZrn/j0iIlVEyKLSDjk0VaWGVjZERCQpHLqwywcEzIlbCChLArx1SbpQBgjITB5+f88l6bw+tLnbzW2eZWJQGEOMwKk3hz4JQnuy8FIH4tw+T2QNkNzSmeA0eA7T0BgFMc8mQox9qsCiQIY+AwDO/bNSOk4zaMIxxyvi8AMAW2xwcH1o+w/hHXf+P6RVq/yo86pjXn/PT09LTsL3dlHSBbTLmdAAAAAElFTkSuQmCC`;
}

function engineUpdateObjects() {
  // recursive object update
  const updateObject = (o) => {
    if (!o.destroyed) {
      o.update();
      for (const child of o.children) updateObject(child);
    }
  };
  for (const o of engineObjects) o.parent || updateObject(o);
  engineObjects = engineObjects.filter((o) => !o.destroyed);
  engineCollideObjects = engineCollideObjects.filter((o) => !o.destroyed);
  time = ++frame / FPS;
}

function forEachObject(
  pos,
  size = 0,
  callbackFunction = (o) => 1,
  collideObjectsOnly = 1
) {
  const objectList = collideObjectsOnly ? engineCollideObjects : engineObjects;
  if (!size) {
    // no overlap test
    for (const o of objectList) callbackFunction(o);
  } else if (size.x != undefined) {
    // aabb test
    for (const o of objectList)
      isOverlapping(pos, size, o.pos, o.size) && callbackFunction(o);
  } else {
    // circle test
    const sizeSquared = size ** 2;
    for (const o of objectList)
      pos.distanceSquared(o.pos) < sizeSquared && callbackFunction(o);
  }
}

/* LittleJS Audio System */
const soundEnable = 1; // all audio can be disabled
const defaultSoundRange = 15; // distance where taper starts
const soundTaperPecent = 0.5; // extra range added for sound taper
const audioVolume = 0.5; // volume for sound, music and speech
let audioContext; // main audio context
// play a zzfx sound in world space with attenuation and culling
function playSound(zzfxSound, pos, range = defaultSoundRange, volumeScale = 1) {
  if (!soundEnable) return;

  const lengthSquared = cameraPos.distanceSquared(pos);
  const maxRange = range * (soundTaperPecent + 1);
  if (lengthSquared > maxRange ** 2) return;

  // copy sound (so volume scale isnt permanant)
  zzfxSound = [...zzfxSound];

  // scale volume
  const scale = volumeScale * percent(lengthSquared ** 0.5, range, maxRange);
  zzfxSound[0] = (zzfxSound[0] || 1) * scale;
  zzfx(...zzfxSound);
}

// ZzFXMicro - Zuper Zmall Zound Zynth - v1.1.8 by Frank Force
const zzfxR = 44100; // sample rate
function zzfx(
  // parameters
  volume = 1,
  randomness = 0.05,
  frequency = 220,
  attack = 0,
  sustain = 0,
  release = 0.1,
  shape = 0,
  shapeCurve = 1,
  slide = 0,
  deltaSlide = 0,
  pitchJump = 0,
  pitchJumpTime = 0,
  repeatTime = 0,
  noise = 0,
  modulation = 0,
  bitCrush = 0,
  delay = 0,
  sustainVolume = 1,
  decay = 0,
  tremolo = 0
) {
  // wait for user input to create audio context
  if (!soundEnable || !hadInput) return;

  // init parameters
  let PI2 = PI * 2,
    sign = (v) => (v > 0 ? 1 : -1),
    startSlide = (slide *= (500 * PI2) / zzfxR / zzfxR),
    b = [],
    startFrequency = (frequency *=
      ((1 + randomness * 2 * Math.random() - randomness) * PI2) / zzfxR),
    t = 0,
    tm = 0,
    i = 0,
    j = 1,
    r = 0,
    c = 0,
    s = 0,
    f,
    length;

  // scale by sample rate
  attack = attack * zzfxR + 9; // minimum attack to prevent pop
  decay *= zzfxR;
  sustain *= zzfxR;
  release *= zzfxR;
  delay *= zzfxR;
  deltaSlide *= (500 * PI2) / zzfxR ** 3;
  modulation *= PI2 / zzfxR;
  pitchJump *= PI2 / zzfxR;
  pitchJumpTime *= zzfxR;
  repeatTime = (repeatTime * zzfxR) | 0;

  // generate waveform
  for (
    length = (attack + decay + sustain + release + delay) | 0;
    i < length;
    b[i++] = s
  ) {
    if (!(++c % ((bitCrush * 100) | 0))) {
      // bit crush
      s = shape
        ? shape > 1
          ? shape > 2
            ? shape > 3 // wave shape
              ? Math.sin((t % PI2) ** 3) // 4 noise
              : Math.max(Math.min(Math.tan(t), 1), -1) // 3 tan
            : 1 - (((((2 * t) / PI2) % 2) + 2) % 2) // 2 saw
          : 1 - 4 * abs(Math.round(t / PI2) - t / PI2) // 1 triangle
        : Math.sin(t); // 0 sin

      s =
        (repeatTime
          ? 1 - tremolo + tremolo * Math.sin((PI2 * i) / repeatTime) // tremolo
          : 1) *
        sign(s) *
        abs(s) ** shapeCurve * // curve 0=square, 2=pointy
        volume *
        audioVolume * // envelope
        (i < attack
          ? i / attack // attack
          : i < attack + decay // decay
          ? 1 - ((i - attack) / decay) * (1 - sustainVolume) // decay falloff
          : i < attack + decay + sustain // sustain
          ? sustainVolume // sustain volume
          : i < length - delay // release
          ? ((length - i - delay) / release) * // release falloff
            sustainVolume // release volume
          : 0); // post release

      s = delay
        ? s / 2 +
          (delay > i
            ? 0 // delay
            : ((i < length - delay ? 1 : (length - i) / delay) * // release delay
                b[(i - delay) | 0]) /
              2)
        : s; // sample delay
    }

    f =
      (frequency += slide += deltaSlide) * // frequency
      Math.cos(modulation * tm++); // modulation
    t += f - f * noise * (1 - (((Math.sin(i) + 1) * 1e9) % 2)); // noise

    if (j && ++j > pitchJumpTime) {
      // pitch jump
      frequency += pitchJump; // apply pitch jump
      startFrequency += pitchJump; // also apply to start
      j = 0; // reset pitch jump time
    }

    if (repeatTime && !(++r % repeatTime)) {
      // repeat
      frequency = startFrequency; // reset frequency
      slide = startSlide; // reset slide
      j = j || 1; // reset pitch jump time
    }
  }

  // create audio context
  if (!audioContext) audioContext = new AudioContext();

  // create buffer and source
  const buffer = audioContext.createBuffer(1, b.length, zzfxR),
    source = audioContext.createBufferSource();

  // copy samples to buffer and play
  buffer.getChannelData(0).set(b);
  source.buffer = buffer;
  source.connect(audioContext.destination);
  source.start();
  return source;
}

/* LittleJS Object Base Class */
// object defaults
const defaultObjectSize = vec2(0.999);
const defaultObjectMass = 1;
const defaultObjectDamping = 0.99;
const defaultObjectAngleDamping = 0.99;
const defaultObjectElasticity = 0;
const defaultObjectFriction = 0.8;
const maxObjectSpeed = 1;

class EngineObject {
  constructor(
    pos,
    size = defaultObjectSize,
    tileIndex = -1,
    tileSize = defaultTileSize,
    angle = 0,
    color
  ) {
    // set passed in params
    this.pos = pos.copy();
    this.size = size;
    this.tileIndex = tileIndex;
    this.tileSize = tileSize;
    this.angle = angle;
    this.color = color;

    // set physics defaults
    this.mass = defaultObjectMass;
    this.damping = defaultObjectDamping;
    this.angleDamping = defaultObjectAngleDamping;
    this.elasticity = defaultObjectElasticity;
    this.friction = defaultObjectFriction;

    // init other object stuff
    this.spawnTime = time;
    this.velocity = vec2(
      (this.collideSolidObjects = this.renderOrder = this.angleVelocity = 0)
    );
    this.collideTiles = this.gravityScale = 1;
    this.children = [];

    // add to list of objects
    engineObjects.push(this);
  }

  update() {
    if (this.parent) {
      // copy parent pos/angle
      this.pos = this.localPos
        .multiply(vec2(this.getMirrorSign(), 1))
        .rotate(-this.parent.angle)
        .add(this.parent.pos);
      this.angle = this.getMirrorSign() * this.localAngle + this.parent.angle;
      return;
    }

    // limit max speed to prevent missing collisions
    this.velocity.x = clamp(this.velocity.x, maxObjectSpeed, -maxObjectSpeed);
    this.velocity.y = clamp(this.velocity.y, maxObjectSpeed, -maxObjectSpeed);

    // apply physics
    const oldPos = this.pos.copy();
    this.pos.x += this.velocity.x = this.damping * this.velocity.x;
    this.pos.y += this.velocity.y =
      this.damping * this.velocity.y + gravity * this.gravityScale;
    this.angle += this.angleVelocity *= this.angleDamping;

    if (!this.mass)
      // do not update collision for fixed objects
      return;

    const wasMovingDown = this.velocity.y < 0;
    if (this.groundObject) {
      // apply friction in local space of ground object
      const groundSpeed = this.groundObject.velocity
        ? this.groundObject.velocity.x
        : 0;
      this.velocity.x =
        groundSpeed + (this.velocity.x - groundSpeed) * this.friction;
      this.groundObject = 0;
    }

    if (this.collideSolidObjects) {
      // check collisions against solid objects
      const epsilon = 1e-3; // necessary to push slightly outside of the collision
      for (const o of engineCollideObjects) {
        // non solid objects don't collide with eachother
        if (!this.isSolid & !o.isSolid || o.destroyed || o.parent) continue;

        // check collision
        if (!isOverlapping(this.pos, this.size, o.pos, o.size) || o == this)
          continue;

        // pass collision to objects
        if (!this.collideWithObject(o) | !o.collideWithObject(this)) continue;

        if (isOverlapping(oldPos, this.size, o.pos, o.size)) {
          // if already was touching, try to push away
          const deltaPos = oldPos.subtract(o.pos);
          const length = deltaPos.length();
          const pushAwayAccel = 0.001; // push away if alread overlapping
          const velocity =
            length < 0.01
              ? randVector(pushAwayAccel)
              : deltaPos.scale(pushAwayAccel / length);
          this.velocity = this.velocity.add(velocity);
          if (o.mass)
            // push away if not fixed
            o.velocity = o.velocity.subtract(velocity);

          continue;
        }

        // check for collision
        const sx = this.size.x + o.size.x;
        const sy = this.size.y + o.size.y;
        const smallStepUp = (oldPos.y - o.pos.y) * 2 > sy + gravity; // prefer to push up if small delta
        const isBlockedX = abs(oldPos.y - o.pos.y) * 2 < sy;
        const isBlockedY = abs(oldPos.x - o.pos.x) * 2 < sx;

        if (smallStepUp || isBlockedY || !isBlockedX) {
          // resolve y collision
          // push outside object collision
          this.pos.y =
            o.pos.y + (sy * 0.5 + epsilon) * sign(oldPos.y - o.pos.y);
          if ((o.groundObject && wasMovingDown) || !o.mass) {
            // set ground object if landed on something
            if (wasMovingDown) this.groundObject = o;

            // bounce if other object is fixed or grounded
            this.velocity.y *= -this.elasticity;
          } else if (o.mass) {
            // set center of mass velocity
            this.velocity.y = o.velocity.y =
              (this.mass * this.velocity.y + o.mass * o.velocity.y) /
              (this.mass + o.mass);
          }
        }
        if (!smallStepUp && (isBlockedX || !isBlockedY)) {
          // resolve x collision
          // push outside collision
          this.pos.x =
            o.pos.x + (sx * 0.5 + epsilon) * sign(oldPos.x - o.pos.x);
          if (o.mass) {
            // set center of mass velocity
            this.velocity.x = o.velocity.x =
              (this.mass * this.velocity.x + o.mass * o.velocity.x) /
              (this.mass + o.mass);
          } // bounce if other object is fixed
          else this.velocity.x *= -this.elasticity;
        }
      }
    }
    if (this.collideTiles) {
      // check collision against tiles
      if (tileCollisionTest(this.pos, this.size, this)) {
        // if already was stuck in collision, don't do anything
        // this should not happen unless something starts in collision
        if (!tileCollisionTest(oldPos, this.size, this)) {
          // test which side we bounced off (or both if a corner)
          const isBlockedY = tileCollisionTest(
            new Vector2(oldPos.x, this.pos.y),
            this.size,
            this
          );
          const isBlockedX = tileCollisionTest(
            new Vector2(this.pos.x, oldPos.y),
            this.size,
            this
          );
          if (isBlockedY || !isBlockedX) {
            // set if landed on ground
            this.groundObject = wasMovingDown;

            // push out of collision and bounce
            this.pos.y = oldPos.y;
            this.velocity.y *= -this.elasticity;
          }
          if (isBlockedX || !isBlockedY) {
            // push out of collision and bounce
            this.pos.x = oldPos.x;
            this.velocity.x *= -this.elasticity;
          }
        }
      }
    }
  }

  render() {
    // default object render
    drawTile(
      this.pos,
      this.size,
      this.tileIndex,
      this.tileSize,
      this.color,
      this.angle,
      this.mirror,
      this.additiveColor
    );
  }

  destroy() {
    if (this.destroyed) return;

    // disconnect from parent and destroy chidren
    this.destroyed = 1;
    this.parent && this.parent.removeChild(this);
    for (const child of this.children) child.destroy((child.parent = 0));
  }
  collideWithTile(data, pos) {
    return data > 0;
  }
  collideWithTileRaycast(data, pos) {
    return data > 0;
  }
  collideWithObject(o) {
    return 1;
  }
  getAliveTime() {
    return time - this.spawnTime;
  }
  applyAcceleration(a) {
    this.velocity = this.velocity.add(a);
  }
  applyForce(force) {
    this.applyAcceleration(force.scale(1 / this.mass));
  }
  getMirrorSign(s = 1) {
    return this.mirror ? -s : s;
  }

  addChild(child, localPos = vec2(), localAngle = 0) {
    this.children.push(child);
    child.parent = this;
    child.localPos = localPos.copy();
    child.localAngle = localAngle;
  }
  removeChild(child) {
    this.children.splice(this.children.indexOf(child), 1);
    child.parent = 0;
  }

  setCollision(collideSolidObjects = 1, isSolid, collideTiles = 1) {
    // track collidable objects in separate list
    if (collideSolidObjects && !this.collideSolidObjects) {
      engineCollideObjects.push(this);
    } else if (!collideSolidObjects && this.collideSolidObjects) {
      engineCollideObjects.splice(engineCollideObjects.indexOf(this), 1);
    }

    this.collideSolidObjects = collideSolidObjects;
    this.isSolid = isSolid;
    this.collideTiles = collideTiles;
  }
}

/* LittleJS Tile Layer System */
// Tile Collision

let tileCollision = [];
let tileCollisionSize = vec2();
const tileLayerCanvasCache = [];
const defaultTileLayerRenderOrder = -1e9;

function initTileCollision(size) {
  // reset collision to be clear
  tileCollisionSize = size;
  tileCollision = [];
  for (let i = (tileCollision.length = tileCollisionSize.area()); i--; )
    tileCollision[i] = 0;
}

const setTileCollisionData = (pos, data = 0) =>
  pos.arrayCheck(tileCollisionSize) &&
  (tileCollision[((pos.y | 0) * tileCollisionSize.x + pos.x) | 0] = data);
const getTileCollisionData = (pos) =>
  pos.arrayCheck(tileCollisionSize)
    ? tileCollision[((pos.y | 0) * tileCollisionSize.x + pos.x) | 0]
    : 0;

function tileCollisionTest(pos, size = vec2(), object) {
  // check if there is collision in a given area
  const minX = (pos.x - size.x * 0.5) | 0;
  const minY = (pos.y - size.y * 0.5) | 0;
  const maxX = (pos.x + size.x * 0.5) | 0;
  const maxY = (pos.y + size.y * 0.5) | 0;
  for (let y = minY; y <= maxY; ++y)
    for (let x = minX; x <= maxX; ++x) {
      const tileData = tileCollision[y * tileCollisionSize.x + x];
      if (
        tileData &&
        (!object || object.collideWithTile(tileData, new Vector2(x, y)))
      )
        return 1;
    }
}

// return the center of tile if any that is hit (this does not return the exact hit point)
// todo: return the exact hit point, it must still be inside the hit tile
function tileCollisionRaycast(posStart, posEnd, object) {
  // test if a ray collides with tiles from start to end
  posStart = posStart.int();
  posEnd = posEnd.int();
  const posDelta = posEnd.subtract(posStart);
  const dx = abs(posDelta.x),
    dy = -abs(posDelta.y);
  const sx = sign(posDelta.x),
    sy = sign(posDelta.y);
  let e = dx + dy;

  for (let x = posStart.x, y = posStart.y; ; ) {
    const tileData = getTileCollisionData(vec2(x, y));
    if (
      tileData &&
      (object
        ? object.collideWithTileRaycast(tileData, new Vector2(x, y))
        : tileData > 0)
    ) {
      return new Vector2(x + 0.5, y + 0.5);
    }

    // update Bresenham line drawing algorithm
    if ((x == posEnd.x) & (y == posEnd.y)) break;
    const e2 = 2 * e;
    if (e2 >= dy) (e += dy), (x += sx);
    if (e2 <= dx) (e += dx), (y += sy);
  }
}

// Tile Layer Rendering System

class TileLayerData {
  constructor(tile = -1, direction = 0, mirror = 0, color = new Color()) {
    this.tile = tile;
    this.direction = direction;
    this.mirror = mirror;
    this.color = color;
  }
  clear() {
    this.tile = this.direction = this.mirror = 0;
    color = new Color();
  }
}

class TileLayer extends EngineObject {
  constructor(pos, size, scale = vec2(1), layer = 0) {
    super(pos, size);

    // create new canvas if necessary
    this.canvas = tileLayerCanvasCache.length
      ? tileLayerCanvasCache.pop()
      : document.createElement("canvas");
    this.context = this.canvas.getContext("2d");

    this.scale = scale;
    this.tileSize = defaultTileSize.copy();
    this.layer = layer;
    this.renderOrder = defaultTileLayerRenderOrder + layer;
    this.flushGLBeforeRender = 1;

    // init tile data
    this.data = [];
    for (let j = this.size.area(); j--; ) this.data.push(new TileLayerData());
  }

  destroy() {
    // add canvas back to the cache
    tileLayerCanvasCache.push(this.canvas);
    super.destroy();
  }

  setData(layerPos, data, redraw) {
    if (layerPos.arrayCheck(this.size)) {
      this.data[((layerPos.y | 0) * this.size.x + layerPos.x) | 0] = data;
      redraw && this.drawTileData(layerPos);
    }
  }

  getData(layerPos) {
    return (
      layerPos.arrayCheck(this.size) &&
      this.data[((layerPos.y | 0) * this.size.x + layerPos.x) | 0]
    );
  }

  update() {} // tile layers are not updated
  render() {
    // flush and copy gl canvas because tile canvas does not use gl
    this.flushGLBeforeRender && glCopyToContext(mainContext);

    // draw the entire cached level onto the main canvas
    const pos = worldToScreen(
      this.pos.add(vec2(0, this.size.y * this.scale.y))
    );
    mainContext.drawImage(
      this.canvas,
      pos.x,
      pos.y,
      cameraScale * this.size.x * this.scale.x,
      cameraScale * this.size.y * this.scale.y
    );
  }

  redraw() {
    // draw all the tile data to an offscreen canvas using webgl if possible
    this.redrawStart();
    this.drawAllTileData();
    this.redrawEnd();
  }

  redrawStart(clear = 1) {
    // clear and set size
    const width = this.size.x * this.tileSize.x;
    const height = this.size.y * this.tileSize.y;

    if (clear) {
      this.canvas.width = width;
      this.canvas.height = height;
    }

    // save current render settings
    this.savedRenderSettings = [
      mainCanvasSize,
      mainCanvas,
      mainContext,
      cameraScale,
      cameraPos,
    ];

    // set camera transform for renering
    cameraScale = this.tileSize.x;
    cameraPos = this.size.scale(0.5);
    mainCanvas = this.canvas;
    mainContext = this.context;
    mainContext.imageSmoothingEnabled = false; // disable smoothing for pixel art
    mainCanvasSize = vec2(width, height);
    glPreRender(width, height);
  }

  redrawEnd() {
    glCopyToContext(mainContext, true);

    // set stuff back to normal
    [mainCanvasSize, mainCanvas, mainContext, cameraScale, cameraPos] =
      this.savedRenderSettings;
  }

  drawTileData(layerPos) {
    // first clear out where the tile was
    const pos = layerPos.int().add(this.pos).add(vec2(0.5));
    this.drawCanvas2D(pos, vec2(1), 0, 0, (context) =>
      context.clearRect(-0.5, -0.5, 1, 1)
    );

    // draw the tile
    const d = this.getData(layerPos);
    d.tile < 0 ||
      drawTile(
        pos,
        vec2(1),
        d.tile || -1,
        this.tileSize,
        d.color,
        (d.direction * PI) / 2,
        d.mirror
      );
  }

  drawAllTileData() {
    for (let x = this.size.x; x--; )
      for (let y = this.size.y; y--; ) this.drawTileData(vec2(x, y));
  }

  // draw directly to the 2d canvas in world space (bipass webgl)
  drawCanvas2D(pos, size, angle, mirror, drawFunction) {
    const context = this.context;
    context.save();
    pos = pos.subtract(this.pos).multiply(this.tileSize);
    size = size.multiply(this.tileSize);
    context.translate(pos.x, this.canvas.height - pos.y);
    context.rotate(angle);
    context.scale(mirror ? -size.x : size.x, size.y);
    drawFunction(context);
    context.restore();
  }

  drawTile(
    pos,
    size = vec2(1),
    tileIndex = 0,
    tileSize = defaultTileSize,
    color = new Color(),
    angle = 0,
    mirror
  ) {
    // draw a tile directly onto the layer canvas
    this.drawCanvas2D(pos, size, angle, mirror, (context) => {
      if (tileIndex < 0) {
        // untextured
        context.fillStyle = color.rgba();
        context.fillRect(-0.5, -0.5, 1, 1);
      } else {
        const cols = tileImage.width / tileSize.x;
        context.globalAlpha = color.a; // full color not supported in this mode
        context.drawImage(
          tileImage,
          (tileIndex % cols) * tileSize.x,
          ((tileIndex / cols) | 0) * tileSize.x,
          tileSize.x,
          tileSize.y,
          -0.5,
          -0.5,
          1,
          1
        );
      }
    });
  }

  drawRect(pos, size, color, angle) {
    this.drawTile(pos, size, -1, 0, color, angle, 0);
  }
}
/* LittleJS Input System */

// input

const enableGamepads = 1;
const enableTouchInput = 0;
const copyGamepadDirectionToStick = 1;
const copyWASDToDpad = 1;

// input for all devices including keyboard, mouse, and gamepad. (d=down, p=pressed, r=released)
const inputData = [[]];
const keyIsDown = (key, device = 0) =>
  inputData[device][key] && inputData[device][key].d ? 1 : 0;
const keyWasPressed = (key, device = 0) =>
  inputData[device][key] && inputData[device][key].p ? 1 : 0;
const keyWasReleased = (key, device = 0) =>
  inputData[device][key] && inputData[device][key].r ? 1 : 0;
const clearInput = () => (inputData[0].length = 0);

// mouse input is stored with keyboard
let hadInput = 0;
let mouseWheel = 0;
let mousePosScreen = vec2();
let mousePosWorld = vec2();
const mouseIsDown = keyIsDown;
const mouseWasPressed = keyWasPressed;
const mouseWasReleased = keyWasReleased;

// handle input events
onkeydown = (e) => {
  e.repeat ||
    (inputData[(isUsingGamepad = 0)][remapKeyCode(e.keyCode)] = {
      d: (hadInput = 1),
      p: 1,
    });
};
onkeyup = (e) => {
  const c = remapKeyCode(e.keyCode);
  inputData[0][c] && ((inputData[0][c].d = 0), (inputData[0][c].r = 1));
};

// oncontextmenu = (e) => !1; // prevent right click menu
const remapKeyCode = (c) =>
  copyWASDToDpad
    ? c == 87
      ? 38
      : c == 83
      ? 40
      : c == 65
      ? 37
      : c == 68
      ? 39
      : c
    : c;

// gamepad

let isUsingGamepad = 0;
let gamepadCount = 0;
const gamepadStick = (stick, gamepad = 0) =>
  gamepad < gamepadCount ? inputData[gamepad + 1].stickData[stick] : vec2();
const gamepadIsDown = (button, gamepad = 0) =>
  gamepad < gamepadCount ? keyIsDown(button, gamepad + 1) : 0;
const gamepadWasPressed = (button, gamepad = 0) =>
  gamepad < gamepadCount ? keyWasPressed(button, gamepad + 1) : 0;
const gamepadWasReleased = (button, gamepad = 0) =>
  gamepad < gamepadCount ? keyWasReleased(button, gamepad + 1) : 0;

function updateGamepads() {
  if (!navigator.getGamepads || !enableGamepads) return;

  const gamepads = navigator.getGamepads();
  gamepadCount = 0;
  for (let i = 0; i < navigator.getGamepads().length; ++i) {
    // get or create gamepad data
    const gamepad = gamepads[i];
    let data = inputData[i + 1];
    if (!data) {
      data = inputData[i + 1] = [];
      data.stickData = [vec2(), vec2()];
    }

    if (gamepad && gamepad.axes.length >= 2) {
      gamepadCount = i + 1;

      // read analog sticks and clamp dead zone
      const deadZone = 0.3,
        deadZoneMax = 0.8;
      const applyDeadZone = (v) =>
        v > deadZone
          ? percent(v, deadZoneMax, deadZone)
          : v < -deadZone
          ? -percent(-v, deadZoneMax, deadZone)
          : 0;
      data.stickData[0] = vec2(
        applyDeadZone(gamepad.axes[0]),
        applyDeadZone(-gamepad.axes[1])
      );

      if (copyGamepadDirectionToStick) {
        // copy dpad to left analog stick when pressed
        if (
          gamepadIsDown(12, i) |
          gamepadIsDown(13, i) |
          gamepadIsDown(14, i) |
          gamepadIsDown(15, i)
        )
          data.stickData[0] = vec2(
            gamepadIsDown(15, i) - gamepadIsDown(14, i),
            gamepadIsDown(12, i) - gamepadIsDown(13, i)
          );
      }

      // clamp stick input to unit vector
      data.stickData[0] = data.stickData[0].clampLength();

      // read buttons
      gamepad.buttons.map((button, j) => {
        inputData[i + 1][j] = button.pressed
          ? { d: 1, p: !gamepadIsDown(j, i) }
          : (inputData[i + 1][j] = { r: gamepadIsDown(j, i) });
        isUsingGamepad |= button.pressed && !i;
      });
    }
  }
}

/* LittleJS Particle System */

class ParticleEmitter extends EngineObject {
  constructor(
    pos, // world space position of emitter
    emitSize = 0, // size of emitter (float for circle diameter, vec2 for rect)
    emitTime = 0, // how long to stay alive (0 is forever)
    emitRate = 100, // how many particles per second to spawn
    emitConeAngle = PI, // local angle to apply velocity to particles from emitter
    tileIndex = -1, // index into tile sheet, if <0 no texture is applied
    tileSize = defaultTileSize, // tile size for particles
    colorStartA = new Color(), // color at start of life
    colorStartB = new Color(), // randomized between start colors
    colorEndA = new Color(1, 1, 1, 0), // color at end of life
    colorEndB = new Color(1, 1, 1, 0), // randomized between end colors
    particleTime = 0.5, // how long particles live
    sizeStart = 0.1, // how big are particles at start
    sizeEnd = 1, // how big are particles at end
    speed = 0.1, // how fast are particles when spawned
    angleSpeed = 0.05, // how fast are particles rotating
    damping = 1, // how much to dampen particle speed
    angleDamping = 1, // how much to dampen particle angular speed
    gravityScale = 0, // how much does gravity effect particles
    particleConeAngle = PI, // cone for start particle angle
    fadeRate = 0.1, // how quick to fade in particles at start/end in percent of life
    randomness = 0.2, // apply extra randomness percent
    collideTiles, // do particles collide against tiles
    additive, // should particles use addtive blend
    randomColorLinear = 1, // should color be randomized linearly or across each component
    renderOrder = additive ? 1e9 : 0 // render order for particles (additive is above other stuff by default)
  ) {
    super(pos, new Vector2(), tileIndex, tileSize);

    // emitter settings
    this.emitSize = emitSize;
    this.emitTime = emitTime;
    this.emitRate = emitRate;
    this.emitConeAngle = emitConeAngle;

    // color settings
    this.colorStartA = colorStartA;
    this.colorStartB = colorStartB;
    this.colorEndA = colorEndA;
    this.colorEndB = colorEndB;
    this.randomColorLinear = randomColorLinear;

    // particle settings
    this.particleTime = particleTime;
    this.sizeStart = sizeStart;
    this.sizeEnd = sizeEnd;
    this.speed = speed;
    this.angleSpeed = angleSpeed;
    this.damping = damping;
    this.angleDamping = angleDamping;
    this.gravityScale = gravityScale;
    this.particleConeAngle = particleConeAngle;
    this.fadeRate = fadeRate;
    this.randomness = randomness;
    this.collideTiles = collideTiles;
    this.additive = additive;
    this.renderOrder = renderOrder;
    this.trailScale = this.emitTimeBuffer = 0;
  }

  update() {
    // only do default update to apply parent transforms
    this.parent && super.update();

    // update emitter
    if (!this.emitTime || this.getAliveTime() <= this.emitTime) {
      // emit particles
      if (this.emitRate) {
        const rate = 1 / this.emitRate;
        for (
          this.emitTimeBuffer += TIME_DELTA;
          this.emitTimeBuffer > 0;
          this.emitTimeBuffer -= rate
        )
          this.emitParticle();
      }
    } else this.destroy();
  }

  emitParticle() {
    // spawn a particle
    const pos =
      this.emitSize.x != undefined // check if vec2 was used for size
        ? new Vector2(rand(-0.5, 0.5), rand(-0.5, 0.5))
            .multiply(this.emitSize)
            .rotate(this.angle) // box emitter
        : randInCircle(this.emitSize * 0.5); // circle emitter
    const particle = new Particle(
      this.pos.add(pos),
      this.tileIndex,
      this.tileSize,
      this.angle + rand(this.particleConeAngle, -this.particleConeAngle)
    );

    // randomness scales each paremeter by a percentage
    const randomness = this.randomness;
    const randomizeScale = (v) => v + v * rand(randomness, -randomness);

    // randomize particle settings
    const particleTime = randomizeScale(this.particleTime);
    const sizeStart = randomizeScale(this.sizeStart);
    const sizeEnd = randomizeScale(this.sizeEnd);
    const speed = randomizeScale(this.speed);
    const angleSpeed = randomizeScale(this.angleSpeed) * randSign();
    const coneAngle = rand(this.emitConeAngle, -this.emitConeAngle);
    const colorStart = randColor(
      this.colorStartA,
      this.colorStartB,
      this.randomColorLinear
    );
    const colorEnd = randColor(
      this.colorEndA,
      this.colorEndB,
      this.randomColorLinear
    );

    // build particle settings
    particle.colorStart = colorStart;
    particle.colorEndDelta = colorEnd.subtract(colorStart);
    particle.velocity = new Vector2().setAngle(this.angle + coneAngle, speed);
    particle.angleVelocity = angleSpeed;
    particle.lifeTime = particleTime;
    particle.sizeStart = sizeStart;
    particle.sizeEndDelta = sizeEnd - sizeStart;
    //particle.mirror          = rand(2)|0; // random mirroring
    particle.fadeRate = this.fadeRate;
    particle.damping = this.damping;
    particle.angleDamping = this.angleDamping;
    particle.elasticity = this.elasticity;
    particle.friction = this.friction;
    particle.gravityScale = this.gravityScale;
    particle.collideTiles = this.collideTiles;
    particle.additive = this.additive;
    particle.renderOrder = this.renderOrder;
    particle.trailScale = this.trailScale;

    // setup callbacks for particles
    particle.destroyCallback = this.particleDestroyCallback;
    this.particleCreateCallback && this.particleCreateCallback(particle);

    // return the newly created particle
    return particle;
  }

  render() {} // emitters are not rendered
}

// particle object

class Particle extends EngineObject {
  constructor(pos, tileIndex, tileSize, angle) {
    super(pos, new Vector2(), tileIndex, tileSize, angle);
  }

  render() {
    // modulate size and color
    const p = min((time - this.spawnTime) / this.lifeTime, 1);
    const radius = this.sizeStart + p * this.sizeEndDelta;
    const size = new Vector2(radius, radius);

    const fadeRate = this.fadeRate * 0.5;
    const color = new Color(
      this.colorStart.r + p * this.colorEndDelta.r,
      this.colorStart.g + p * this.colorEndDelta.g,
      this.colorStart.b + p * this.colorEndDelta.b,
      (this.colorStart.a + p * this.colorEndDelta.a) *
        (p < fadeRate
          ? p / fadeRate
          : p > 1 - fadeRate
          ? (1 - p) / fadeRate
          : 1)
    ); // fade alpha

    // draw the particle
    this.additive && glSetBlendMode(1);
    if (this.trailScale) {
      // trail style particles
      const speed = this.velocity.length();
      const direction = this.velocity.scale(1 / speed);
      const trailLength = speed * this.trailScale;
      size.y = max(size.x, trailLength);
      this.angle = direction.angle();
      drawTile(
        this.pos.add(direction.multiply(vec2(0, -trailLength * 0.5))),
        size,
        this.tileIndex,
        this.tileSize,
        color,
        this.angle,
        this.mirror
      );
    } else
      drawTile(
        this.pos,
        size,
        this.tileIndex,
        this.tileSize,
        color,
        this.angle,
        this.mirror
      );
    this.additive && glSetBlendMode(0);

    if (p == 1) {
      this.color = color;
      this.size = size;
      this.destroyCallback && this.destroyCallback(this);
      this.destroyed = 1;
      return;
    }
  }
}

/* LittleJS WebGL Interface */

let glCanvas,
  glContext,
  glTileTexture,
  glShader,
  glPositionData,
  glColorData,
  glBatchCount,
  glDirty,
  glAdditive,
  glShrinkTilesX,
  glShrinkTilesY;

function glInit() {
  // create the canvas and tile texture
  glCanvas = document.createElement("canvas");
  glContext = glCanvas.getContext("webgl", { antialias: false });
  glTileTexture = glCreateTexture(tileImage);
  glShrinkTilesX = tileBleedShrinkFix / tileImageSize.x;
  glShrinkTilesY = tileBleedShrinkFix / tileImageSize.y;

  // firefox is much faster without copying the gl buffer so we just overlay it with some tradeoffs
  document.body.appendChild(glCanvas);
  glCanvas.style = mainCanvas.style.cssText;

  // setup vertex and fragment shaders
  glShader = glCreateProgram(
    "precision lowp float;" + // use lowp for better performance
      "uniform mat4 m;" + // transform matrix
      "attribute float a;" + // angle
      "attribute vec2 p,s,t;" + // position, size, uv
      "attribute vec4 c,b;" + // color, additiveColor
      "varying vec2 v;" + // return uv
      "varying vec4 d,e;" + // return color, additiveColor
      "void main(){" + // shader entry point
      "gl_Position=m*vec4((s*cos(-a)+vec2(-s.y,s.x)*sin(-a))*.5+p,1,1);" + // transform position
      "v=t;d=c;e=b;" + // pass stuff to fragment shader
      "}", // end of shader
    "precision lowp float;" + // use lowp for better performance
      "varying vec2 v;" + // uv
      "varying vec4 d,e;" + // color, additiveColor
      "uniform sampler2D j;" + // texture
      "void main(){" + // shader entry point
      "gl_FragColor=texture2D(j,v)*d+e;" + // modulate texture by color plus additive
      "}" // end of shader
  );

  // init buffers
  const glVertexData = new ArrayBuffer(
    MAX_BATCH * VERTICES_PER_QUAD * VERTEX_STRIDE
  );
  glCreateBuffer(gl_ARRAY_BUFFER, glVertexData.byteLength, gl_DYNAMIC_DRAW);
  glPositionData = new Float32Array(glVertexData);
  glColorData = new Uint32Array(glVertexData);

  // setup the vertex data array
  const initVertexAttribArray = (name, type, typeSize, size, normalize = 0) => {
    const location = glContext.getAttribLocation(glShader, name);
    glContext.enableVertexAttribArray(location);
    glContext.vertexAttribPointer(
      location,
      size,
      type,
      normalize,
      VERTEX_STRIDE,
      offset
    );
    offset += size * typeSize;
  };
  let offset = (glDirty = glBatchCount = 0);
  initVertexAttribArray("a", gl_FLOAT, 4, 1); // angle
  initVertexAttribArray("p", gl_FLOAT, 4, 2); // position
  initVertexAttribArray("s", gl_FLOAT, 4, 2); // size
  initVertexAttribArray("t", gl_FLOAT, 4, 2); // texture coords
  initVertexAttribArray("c", gl_UNSIGNED_BYTE, 1, 4, 1); // color
  initVertexAttribArray("b", gl_UNSIGNED_BYTE, 1, 4, 1); // additiveColor

  // use point filtering for pixelated rendering
  glContext.texParameteri(gl_TEXTURE_2D, gl_TEXTURE_MIN_FILTER, gl_NEAREST);
  glContext.texParameteri(gl_TEXTURE_2D, gl_TEXTURE_MAG_FILTER, gl_NEAREST);
}

function glSetBlendMode(additive) {
  if (additive != glAdditive) glFlush();

  // setup blending
  glAdditive = additive;

  glContext.blendFunc(gl_SRC_ALPHA, additive ? gl_ONE : gl_ONE_MINUS_SRC_ALPHA);
  /*glContext.blendFuncSeparate(
        gl_SRC_ALPHA, additive ? gl_ONE : gl_ONE_MINUS_SRC_ALPHA, 
        gl_ONE,       additive ? gl_ONE : gl_ONE_MINUS_SRC_ALPHA);*/
  glContext.enable(gl_BLEND);
}

function glCompileShader(source, type) {
  // build the shader
  const shader = glContext.createShader(type);
  glContext.shaderSource(shader, source);
  glContext.compileShader(shader);

  // check for errors
  return shader;
}

function glCreateProgram(vsSource, fsSource) {
  // build the program
  const program = glContext.createProgram();
  glContext.attachShader(program, glCompileShader(vsSource, gl_VERTEX_SHADER));
  glContext.attachShader(
    program,
    glCompileShader(fsSource, gl_FRAGMENT_SHADER)
  );
  glContext.linkProgram(program);

  // check for errors
  return program;
}

function glCreateBuffer(bufferType, size, usage) {
  // build the buffer
  const buffer = glContext.createBuffer();
  glContext.bindBuffer(bufferType, buffer);
  glContext.bufferData(bufferType, size, usage);
  return buffer;
}

function glCreateTexture(image) {
  // build the texture
  const texture = glContext.createTexture();
  glContext.bindTexture(gl_TEXTURE_2D, texture);
  glContext.texImage2D(
    gl_TEXTURE_2D,
    0,
    gl_RGBA,
    gl_RGBA,
    gl_UNSIGNED_BYTE,
    image
  );
  return texture;
}

function glPreRender(width, height) {
  // clear and set to same size as main canvas
  glCanvas.width = width;
  glCanvas.height = height;
  glContext.viewport(0, 0, width, height);

  // set up the shader
  glContext.useProgram(glShader);
  glSetBlendMode();

  // build the transform matrix
  const sx = (2 * cameraScale) / width;
  const sy = (2 * cameraScale) / height;
  glContext.uniformMatrix4fv(
    glContext.getUniformLocation(glShader, "m"),
    0,
    new Float32Array([
      sx,
      0,
      0,
      0,
      0,
      sy,
      0,
      0,
      1,
      1,
      -1,
      1,
      -1 - sx * cameraPos.x,
      -1 - sy * cameraPos.y,
      0,
      0,
    ])
  );
}

function glFlush() {
  if (!glBatchCount) return;

  // draw all the sprites in the batch and reset the buffer
  glContext.bufferSubData(
    gl_ARRAY_BUFFER,
    0,
    glPositionData.subarray(0, glBatchCount * VERTICES_PER_QUAD * VERTEX_STRIDE)
  );
  glContext.drawArrays(gl_TRIANGLES, 0, glBatchCount * VERTICES_PER_QUAD);
  glBatchCount = 0;
}

function glCopyToContext(context, forceDraw) {
  if (!glDirty) return;

  // draw any sprites still in the buffer, copy to main canvas and clear
  glFlush();

  if (forceDraw) {
    // do not draw/clear in overlay mode because the canvas is visible
    context.drawImage(glCanvas, 0, (glAdditive = glDirty = 0));
    glContext.clear(gl_COLOR_BUFFER_BIT);
  }
}

function glDraw(
  x,
  y,
  sizeX,
  sizeY,
  angle,
  mirror,
  uv0X,
  uv0Y,
  uv1X,
  uv1Y,
  abgr,
  abgrAdditive
) {
  // flush if there is no room for more verts
  if (glBatchCount >= MAX_BATCH) glFlush();

  if (tileBleedShrinkFix) {
    // shrink tiles to prevent bleeding
    uv0X += glShrinkTilesX;
    uv0Y += glShrinkTilesY;
    uv1X -= glShrinkTilesX;
    uv1Y -= glShrinkTilesY;
  }

  // setup 2 triangles to form a quad
  let offset = glBatchCount++ * VERTICES_PER_QUAD * INDICIES_PER_VERT - 1;
  sizeX = mirror ? -sizeX : sizeX;
  glDirty = 1;

  // vertex 0
  glPositionData[++offset] = angle;
  glPositionData[++offset] = x;
  glPositionData[++offset] = y;
  glPositionData[++offset] = -sizeX;
  glPositionData[++offset] = -sizeY;
  glPositionData[++offset] = uv0X;
  glPositionData[++offset] = uv1Y;
  glColorData[++offset] = abgr;
  glColorData[++offset] = abgrAdditive;

  // vertex 1
  glPositionData[++offset] = angle;
  glPositionData[++offset] = x;
  glPositionData[++offset] = y;
  glPositionData[++offset] = sizeX;
  glPositionData[++offset] = sizeY;
  glPositionData[++offset] = uv1X;
  glPositionData[++offset] = uv0Y;
  glColorData[++offset] = abgr;
  glColorData[++offset] = abgrAdditive;

  // vertex 2
  glPositionData[++offset] = angle;
  glPositionData[++offset] = x;
  glPositionData[++offset] = y;
  glPositionData[++offset] = -sizeX;
  glPositionData[++offset] = sizeY;
  glPositionData[++offset] = uv0X;
  glPositionData[++offset] = uv0Y;
  glColorData[++offset] = abgr;
  glColorData[++offset] = abgrAdditive;

  // vertex 0
  glPositionData[++offset] = angle;
  glPositionData[++offset] = x;
  glPositionData[++offset] = y;
  glPositionData[++offset] = -sizeX;
  glPositionData[++offset] = -sizeY;
  glPositionData[++offset] = uv0X;
  glPositionData[++offset] = uv1Y;
  glColorData[++offset] = abgr;
  glColorData[++offset] = abgrAdditive;

  // vertex 3
  glPositionData[++offset] = angle;
  glPositionData[++offset] = x;
  glPositionData[++offset] = y;
  glPositionData[++offset] = sizeX;
  glPositionData[++offset] = -sizeY;
  glPositionData[++offset] = uv1X;
  glPositionData[++offset] = uv1Y;
  glColorData[++offset] = abgr;
  glColorData[++offset] = abgrAdditive;

  // vertex 1
  glPositionData[++offset] = angle;
  glPositionData[++offset] = x;
  glPositionData[++offset] = y;
  glPositionData[++offset] = sizeX;
  glPositionData[++offset] = sizeY;
  glPositionData[++offset] = uv1X;
  glPositionData[++offset] = uv0Y;
  glColorData[++offset] = abgr;
  glColorData[++offset] = abgrAdditive;
}

// store gl constants as integers so their name doesn't use space in minifed
const gl_ONE = 1,
  gl_TRIANGLES = 4,
  gl_SRC_ALPHA = 770,
  gl_ONE_MINUS_SRC_ALPHA = 771,
  gl_BLEND = 3042,
  gl_TEXTURE_2D = 3553,
  gl_UNSIGNED_BYTE = 5121,
  gl_FLOAT = 5126,
  gl_RGBA = 6408,
  gl_NEAREST = 9728,
  gl_LINEAR = 9729,
  gl_TEXTURE_MAG_FILTER = 10240,
  gl_TEXTURE_MIN_FILTER = 10241,
  gl_COLOR_BUFFER_BIT = 16384,
  gl_ARRAY_BUFFER = 34962,
  gl_DYNAMIC_DRAW = 35048,
  gl_FRAGMENT_SHADER = 35632,
  gl_VERTEX_SHADER = 35633,
  gl_COMPILE_STATUS = 35713,
  gl_LINK_STATUS = 35714,
  // constants for batch rendering
  VERTICES_PER_QUAD = 6,
  INDICIES_PER_VERT = 9,
  MAX_BATCH = 1 << 16,
  VERTEX_STRIDE = 4 + 4 * 2 * 3 + 4 * 2; // float + vec2 * 3 + (char * 4) * 2
/*
    LittleJS Drawing System

    - Super fast tile sheet rendering
    - Utility functions for webgl
    - Adapted from Tiny-Canvas https://github.com/bitnenfer/tiny-canvas
*/

///////////////////////////////////////////////////////////////////////////////\

const screenToWorld = (screenPos) =>
  screenPos
    .add(vec2(0.5))
    .subtract(mainCanvasSize.scale(0.5))
    .multiply(vec2(1 / cameraScale, -1 / cameraScale))
    .add(cameraPos);
const worldToScreen = (worldPos) =>
  worldPos
    .subtract(cameraPos)
    .multiply(vec2(cameraScale, -cameraScale))
    .add(mainCanvasSize.scale(0.5))
    .subtract(vec2(0.5));

// draw textured tile centered on pos
function drawTile(
  pos,
  size = vec2(1),
  tileIndex = -1,
  tileSize = defaultTileSize,
  color = new Color(),
  angle = 0,
  mirror,
  additiveColor = new Color(0, 0, 0, 0)
) {
  if (!size.x | !size.y) return;

  if (tileIndex < 0) {
    // if negative tile index, force untextured
    glDraw(
      pos.x,
      pos.y,
      size.x,
      size.y,
      angle,
      0,
      0,
      0,
      0,
      0,
      0,
      color.rgbaInt()
    );
  } else {
    // calculate uvs and render
    const cols = (tileImage.width / tileSize.x) | 0;
    const uvSizeX = tileSize.x * tileImageSizeInverse.x;
    const uvSizeY = tileSize.y * tileImageSizeInverse.y;
    const uvX = (tileIndex % cols) * uvSizeX,
      uvY = ((tileIndex / cols) | 0) * uvSizeY;
    glDraw(
      pos.x,
      pos.y,
      size.x,
      size.y,
      angle,
      mirror,
      uvX,
      uvY,
      uvX + uvSizeX,
      uvY + uvSizeY,
      color.rgbaInt(),
      additiveColor.rgbaInt()
    );
  }
}

// draw a colored untextured rect centered on pos
function drawRect(pos, size, color, angle) {
  drawTile(pos, size, -1, defaultTileSize, color, angle);
}

// draw textured tile centered on pos in screen space
function drawTileScreenSpace(
  pos,
  size = vec2(1),
  tileIndex,
  tileSize,
  color,
  angle,
  mirror,
  additiveColor
) {
  drawTile(
    screenToWorld(pos),
    size.scale(1 / cameraScale),
    tileIndex,
    tileSize,
    color,
    angle,
    mirror,
    additiveColor
  );
}

// END ENGINE

class GameObject extends EngineObject {
  constructor(pos, size, tileIndex, tileSize, angle) {
    super(pos, size, tileIndex, tileSize, angle);
    this.isGameObject = 1;
    this.health = this.healthMax = 0;
    this.burnDelay = 0.1;
    this.burnTime = 3;
    this.damageTimer = new Timer();
    this.burnDelayTimer = new Timer();
    this.burnTimer = new Timer();
    this.extinguishTimer = new Timer();
    this.color = new Color();
    this.additiveColor = new Color(0, 0, 0, 0);
  }

  inUpdateWindow() {
    return (
      levelWarmup ||
      isOverlapping(this.pos, this.size, cameraPos, updateWindowSize)
    );
  }

  update() {
    if (
      this.parent ||
      this.persistent ||
      !this.groundObject ||
      this.inUpdateWindow()
    )
      // pause physics if outside update window
      super.update();

    if (!this.isLavaRock) {
      if (!this.isDead() && this.damageTimer.isSet()) {
        // flash white when damaged
        const a = 0.5 * percent(this.damageTimer.get(), 0, 0.15);
        this.additiveColor = new Color(a, a, a, 0);
      } else this.additiveColor = new Color(0, 0, 0, 0);
    }

    if (!this.parent && this.pos.y < -1) {
      // kill and destroy if fall below level
      this.kill();
      this.persistent || this.destroy();
    } else if (this.burnTime) {
      if (this.burnTimer.isSet()) {
        // burning
        if (this.burnTimer.elapsed()) {
          this.kill();
          if (this.fireEmitter) this.fireEmitter.emitRate = 0;
        } else if (rand() < 0.01) {
          // random chance to spread fire
          const spreadRadius = 2;
          forEachObject(
            this.pos,
            spreadRadius,
            (o) => o.isGameObject && o.burn()
          );
        }
      } else if (this.burnDelayTimer.elapsed()) {
        // finished waiting to burn
        this.burn(1);
      }
    }
  }

  render() {
    drawTile(
      this.pos,
      this.size,
      this.tileIndex,
      this.tileSize,
      this.color.scale(this.burnColorPercent(), 1),
      this.angle,
      this.mirror,
      this.additiveColor
    );
  }

  burnColorPercent() {
    return lerp(this.burnTimer.getPercent(), 0.2, 1);
  }

  burn(instant) {
    if (
      !this.canBurn ||
      this.burnTimer.isSet() ||
      this.extinguishTimer.active()
    )
      return;

    if (GOD_MODE && this.isPlayer) return;

    if (this.team == team_player) {
      // safety window after spawn
      if (GOD_MODE || this.getAliveTime() < 2) return;
    }

    if (instant) {
      this.burnTimer.set(this.burnTime * rand(1.5, 1));
      this.fireEmitter = makeFire();
      this.addChild(this.fireEmitter);
    } else
      this.burnDelayTimer.isSet() ||
        this.burnDelayTimer.set(this.burnDelay * rand(1.5, 1));
  }

  extinguish() {
    if (this.fireEmitter && this.fireEmitter.emitRate == 0) return;

    // stop burning
    this.extinguishTimer.set(0.1);
    this.burnTimer.unset();
    this.burnDelayTimer.unset();
    if (this.fireEmitter) this.fireEmitter.destroy();
    this.fireEmitter = 0;
  }

  heal(health) {
    if (this.isDead()) return 0;

    // apply healing and return amount healed
    return (
      this.health - (this.health = min(this.health + health, this.healthMax))
    );
  }

  damage(damage, damagingObject) {
    if (this.isDead()) return 0;

    // set damage timer;
    this.damageTimer.set();
    for (const child of this.children)
      child.damageTimer && child.damageTimer.set();

    // apply damage and kill if necessary
    const newHealth = max(this.health - damage, 0);
    if (!newHealth) this.kill(damagingObject);

    // set new health and return amount damaged
    return this.health - (this.health = newHealth);
  }

  isDead() {
    return !this.health;
  }
  kill(damagingObject) {
    this.destroy();
  }

  collideWithObject(o) {
    if (o.isLavaRock && this.canBurn) {
      if (levelWarmup) {
        this.destroy();
        return 1;
      }
      this.burn();
    }
    return 1;
  }
}

const propType_crate_wood = 0;
const propType_crate_explosive = 1;
const propType_crate_metal = 2;
const propType_barrel_explosive = 3;
const propType_barrel_water = 4;
const propType_barrel_metal = 5;
const propType_barrel_highExplosive = 6;
const propType_rock = 7;
const propType_rock_lava = 8;
const propType_count = 9;

class Prop extends GameObject {
  constructor(pos, typeOverride) {
    super(pos);

    const type = (this.type =
      typeOverride != undefined
        ? typeOverride
        : (rand() ** 2 * propType_count) | 0);
    let health = 5;
    this.tileIndex = 16;
    this.explosionSize = 0;
    if (type == propType_crate_wood) {
      this.color = new Color(1, 0.5, 0);
      this.canBurn = 1;
    } else if (type == propType_crate_metal) {
      this.color = new Color(0.9, 0.9, 1);
      health = 10;
    } else if (type == propType_crate_explosive) {
      this.color = new Color(0.2, 0.8, 0.2);
      this.canBurn = 1;
      this.explosionSize = 2;
      health = 1e3;
    } else if (type == propType_barrel_metal) {
      this.tileIndex = 17;
      this.color = new Color(0.9, 0.9, 1);
      health = 10;
    } else if (type == propType_barrel_explosive) {
      this.tileIndex = 17;
      this.color = new Color(0.2, 0.8, 0.2);
      this.canBurn = 1;
      this.explosionSize = 2;
      health = 1e3;
    } else if (type == propType_barrel_highExplosive) {
      this.tileIndex = 17;
      this.color = new Color(1, 0.1, 0.1);
      this.canBurn = 1;
      this.explosionSize = 3;
      this.burnTimeDelay = 0;
      this.burnTime = rand(0.5, 0.1);
      health = 1e3;
    } else if (type == propType_barrel_water) {
      this.tileIndex = 17;
      this.color = new Color(0, 0.6, 1);
      health = 0.01;
    } else if (type == propType_rock || type == propType_rock_lava) {
      this.tileIndex = 18;
      this.color = new Color(0.8, 0.8, 0.8).mutate(0.2);
      health = 30;
      this.mass *= 4;
      if (rand() < 0.2) {
        health = 99;
        this.mass *= 4;
        this.size = this.size.scale(2);
        this.pos.y += 0.5;
      }
      this.isCrushing = 1;

      if (type == propType_rock_lava) {
        this.color = new Color(1, 0.9, 0);
        this.additiveColor = new Color(1, 0, 0);
        this.isLavaRock = 1;
      }
    }

    // randomly angle and flip axis (90 degree rotation)
    this.angle = ((rand(4) | 0) * PI) / 2;
    if (rand() < 0.5) this.size = this.size.flip();

    this.mirror = rand() < 0.5;
    this.health = this.healthMax = health;
    this.setCollision(1, 1);
  }

  update() {
    const oldVelocity = this.velocity.copy();
    super.update();

    // apply collision damage
    const deltaSpeedSquared = this.velocity
      .subtract(oldVelocity)
      .lengthSquared();
    deltaSpeedSquared > 0.05 && this.damage(2 * deltaSpeedSquared);
  }

  damage(damage, damagingObject) {
    (this.explosionSize ||
      (this.type == propType_crate_wood && rand() < 0.1)) &&
      this.burn();
    super.damage(damage, damagingObject);
  }

  kill() {
    if (this.destroyed) return;

    if (this.type == propType_barrel_water) makeWater(this.pos);

    this.destroy();
    makeDebris(this.pos, this.color.scale(this.burnColorPercent(), 1));

    this.explosionSize
      ? explosion(this.pos, this.explosionSize)
      : playSound(fxDestroy, this.pos);
  }
}

let checkpointPos,
  activeCheckpoint,
  checkpointTimer = new Timer();

class Checkpoint extends GameObject {
  constructor(pos) {
    super(pos.int().add(vec2(0.5)));
    this.renderOrder = tileRenderOrder - 1;
    this.isCheckpoint = 1;
    for (let x = 3; x--; )
      for (let y = 6; y--; )
        setTileCollisionData(
          pos.subtract(vec2(x - 1, 1 - y)),
          y ? tileType_empty : tileType_solid
        );
  }

  update() {
    if (!this.inUpdateWindow()) return; // ignore offscreen objects

    // check if player is near
    for (const player of players)
      player &&
        !player.isDead() &&
        this.pos.distanceSquared(player.pos) < 1 &&
        this.setActive();
  }

  setActive() {
    if (activeCheckpoint != this && !levelWarmup)
      playSound(fxCheckPoint, this.pos);

    checkpointPos = this.pos;
    activeCheckpoint = this;
    checkpointTimer.set(0.1);
  }

  render() {
    // draw flag
    const height = 4;
    const color = activeCheckpoint == this ? new Color(1, 0, 0) : new Color();
    const a = Math.sin(time * 4 + this.pos.x);
    drawTile(
      this.pos.add(vec2(0.5, height - 0.3 - 0.5 - 0.03 * a)),
      vec2(1, 0.6),
      14,
      undefined,
      color,
      a * 0.06
    );
    drawRect(
      this.pos.add(vec2(0, height / 2 - 0.5)),
      vec2(0.1, height),
      new Color(0.9, 0.9, 0.9)
    );
  }
}

class Grenade extends GameObject {
  constructor(pos) {
    super(pos, vec2(0.2), 5, vec2(8));

    this.health = this.healthMax = 1e3;
    this.beepTimer = new Timer(1);
    this.elasticity = 0.3;
    this.friction = 0.9;
    this.angleDamping = 0.96;
    this.renderOrder = 1e8;
    this.setCollision();
  }

  update() {
    super.update();

    if (this.getAliveTime() > 3) {
      explosion(this.pos, 3);
      this.destroy();
      return;
    }

    if (this.beepTimer.elapsed()) {
      playSound(fxGrenade, this.pos);
      this.beepTimer.set(1);
    }

    alertEnemies(this.pos, this.pos);
  }

  render() {
    drawTile(
      this.pos,
      vec2(0.5),
      this.tileIndex,
      this.tileSize,
      this.color,
      this.angle
    );

    const a = this.getAliveTime();
    glSetBlendMode(1);
    drawTile(
      this.pos,
      vec2(2),
      0,
      vec2(16),
      new Color(1, 0, 0, 0.2 - 0.2 * Math.cos(a * 2 * PI))
    );
    drawTile(
      this.pos,
      vec2(1),
      0,
      vec2(16),
      new Color(1, 0, 0, 0.2 - 0.2 * Math.cos(a * 2 * PI))
    );
    drawTile(
      this.pos,
      vec2(0.5),
      0,
      vec2(16),
      new Color(1, 1, 1, 0.2 - 0.2 * Math.cos(a * 2 * PI))
    );
    glSetBlendMode(0);
  }
}

class Weapon extends EngineObject {
  constructor(pos, parent) {
    super(pos, vec2(0.6), 4, vec2(8));

    // weapon settings
    this.isWeapon = 1;
    this.fireTimeBuffer = this.localAngle = 0;
    this.recoilTimer = new Timer();

    this.addChild(
      (this.shellEmitter = new ParticleEmitter(
        vec2(),
        0,
        0,
        0,
        0.1, // pos, emitSize, emitTime, emitRate, emiteCone
        undefined,
        undefined, // tileIndex, tileSize
        new Color(1, 0.8, 0.5),
        new Color(0.9, 0.7, 0.5), // colorStartA, colorStartB
        new Color(1, 0.8, 0.5),
        new Color(0.9, 0.7, 0.5), // colorEndA, colorEndB
        3,
        0.1,
        0.1,
        0.15,
        0.1, // particleTime, sizeStart, sizeEnd, particleSpeed, particleAngleSpeed
        1,
        0.95,
        1,
        0,
        0, // damping, angleDamping, gravityScale, particleCone, fadeRate,
        0.1,
        1 // randomness, collide, additive, randomColorLinear, renderOrder
      ))
    );
    this.shellEmitter.elasticity = 0.5;
    this.shellEmitter.particleDestroyCallback =
      persistentParticleDestroyCallback;
    this.renderOrder = parent.renderOrder + 1;

    parent.weapon = this;
    parent.addChild(this, (this.localOffset = vec2(0.55, 0)));
  }

  update() {
    super.update();

    const fireRate = 8;
    const bulletSpeed = 0.5;
    const spread = 0.1;

    this.mirror = this.parent.mirror;
    this.fireTimeBuffer += TIME_DELTA;

    if (this.recoilTimer.active())
      this.localAngle = lerp(this.recoilTimer.getPercent(), 0, this.localAngle);

    if (this.triggerIsDown) {
      // slow down enemy bullets
      const speed = bulletSpeed * (this.parent.isPlayer ? 1 : 0.5);
      const rate = 1 / fireRate;
      for (; this.fireTimeBuffer > 0; this.fireTimeBuffer -= rate) {
        this.localAngle = -rand(0.2, 0.15);
        this.recoilTimer.set(rand(0.4, 0.3));
        const bullet = new Bullet(this.pos, this.parent);
        const direction = vec2(this.getMirrorSign(speed), 0);
        bullet.velocity = direction.rotate(rand(spread, -spread));

        this.shellEmitter.localAngle = -0.8 * this.getMirrorSign();
        this.shellEmitter.emitParticle();
        playSound(fxShoot, this.pos);

        // alert enemies
        this.parent.isPlayer && alertEnemies(this.pos, this.pos);
      }
    } else this.fireTimeBuffer = min(this.fireTimeBuffer, 0);
  }
}

class Bullet extends EngineObject {
  constructor(pos, attacker) {
    super(pos, vec2(0));
    this.color = new Color(1, 1, 0, 1);
    this.lastVelocity = this.velocity;
    this.setCollision();

    this.damage = this.damping = 1;
    this.gravityScale = 0;
    this.attacker = attacker;
    this.team = attacker.team;
    this.renderOrder = 1e9;
    this.range = 8;
  }

  update() {
    this.lastVelocity = this.velocity;
    super.update();

    this.range -= this.velocity.length();
    if (this.range < 0) {
      const emitter = new ParticleEmitter(
        this.pos,
        0.2,
        0.1,
        100,
        PI, // pos, emitSize, emitTime, emitRate, emiteCone
        0,
        undefined, // tileIndex, tileSize
        new Color(1, 1, 0, 0.5),
        new Color(1, 1, 1, 0.5), // colorStartA, colorStartB
        new Color(1, 1, 0, 0),
        new Color(1, 1, 1, 0), // colorEndA, colorEndB
        0.1,
        0.5,
        0.1,
        0.1,
        0.1, // particleTime, sizeStart, sizeEnd, particleSpeed, particleAngleSpeed
        1,
        1,
        0.5,
        PI,
        0.1, // damping, angleDamping, gravityScale, particleCone, fadeRate,
        0.5,
        0,
        1 // randomness, collide, additive, randomColorLinear, renderOrder
      );

      this.destroy();
      return;
    }

    // check if hit someone
    forEachObject(this.pos, this.size, (o) => {
      if (o.isGameObject && !o.parent && o.team != this.team)
        if (!o.dodgeTimer || !o.dodgeTimer.active()) this.collideWithObject(o);
    });
  }

  collideWithObject(o) {
    if (o.isGameObject) {
      o.damage(this.damage, this);
      o.applyForce(this.velocity.scale(0.1));
      if (o.isCharacter) {
        playSound(fxWalk, this.pos);
        this.destroy();
      } else this.kill();
    }

    return 1;
  }

  collideWithTile(data, pos) {
    if (data <= 0) return 0;

    const destroyTileChance =
      data == tileType_glass ? 1 : data == tileType_dirt ? 0.2 : 0.05;
    rand() < destroyTileChance && destroyTile(pos);
    this.kill();

    return 1;
  }

  kill() {
    if (this.destroyed) return;

    const emitter = new ParticleEmitter(
      this.pos,
      0,
      0.1,
      100,
      0.5, // pos, emitSize, emitTime, emitRate, emiteCone
      undefined,
      undefined, // tileIndex, tileSize
      new Color(1, 1, 0),
      new Color(1, 0, 0), // colorStartA, colorStartB
      new Color(1, 1, 0),
      new Color(1, 0, 0), // colorEndA, colorEndB
      0.2,
      0.2,
      0,
      0.1,
      0.1, // particleTime, sizeStart, sizeEnd, particleSpeed, particleAngleSpeed
      1,
      1,
      0.5,
      PI,
      0.1, // damping, angleDamping, gravityScale, particleCone, fadeRate,
      0.5,
      1,
      1 // randomness, collide, additive, randomColorLinear, renderOrder
    );
    emitter.trailScale = 1;
    emitter.angle = this.lastVelocity.angle() + PI;
    emitter.elasticity = 0.3;

    this.destroy();
  }

  render() {
    drawRect(
      this.pos,
      vec2(0.4, 0.5),
      new Color(1, 1, 1, 0.5),
      this.velocity.angle()
    );
    drawRect(this.pos, vec2(0.2, 0.5), this.color, this.velocity.angle());
  }
}

const aiEnable = 1;
const maxCharacterSpeed = 0.2;

class Character extends GameObject {
  constructor(pos, sizeScale = 1) {
    super(pos, vec2(0.6, 0.95).scale(sizeScale), 32);

    this.health = this.healthMax = this.canBurn = this.isCharacter = 1;
    this.sizeScale = sizeScale;
    this.groundTimer = new Timer();
    this.jumpTimer = new Timer();
    this.pressedJumpTimer = new Timer();
    this.preventJumpTimer = new Timer();
    this.dodgeTimer = new Timer();
    this.dodgeRechargeTimer = new Timer();
    this.deadTimer = new Timer();
    this.blinkTimer = new Timer();
    this.moveInput = vec2();
    this.extraAdditiveColor = new Color(0, 0, 0, 0);
    this.color = new Color();
    this.eyeColor = new Color();
    this.bodyTile = 3;
    this.headTile = 2;
    this.renderOrder = 10;
    this.overkill = this.grenadeCount = this.walkCyclePercent = 0;
    this.grendeThrowTimer = new Timer();
    this.setCollision();
  }

  update() {
    this.lastPos = this.pos.copy();
    this.gravityScale = 1; // reset default gravity (incase climbing ladder)

    if (this.isDead() || (!this.inUpdateWindow() && !this.persistent)) {
      super.update();
      return; // ignore offscreen objects
    }

    let moveInput = this.moveInput.copy();

    // allow grabbing ladder at head or feet
    let touchingLadder = 0;
    for (let y = 2; y--; ) {
      const testPos = this.pos.add(
        vec2(0, y + 0.1 * this.moveInput.y - this.size.y * 0.5)
      );
      const collisionData = getTileCollisionData(testPos);
      touchingLadder |= collisionData == tileType_ladder;
    }
    if (!touchingLadder) this.climbingLadder = 0;
    else if (this.moveInput.y) this.climbingLadder = 1;

    if (this.dodgeTimer.active()) {
      // update roll
      this.angle = this.getMirrorSign(2 * PI * this.dodgeTimer.getPercent());

      if (this.groundObject) this.velocity.x += this.getMirrorSign(0.1);

      // apply damage to enemies when rolling
      forEachObject(this.pos, this.size, (o) => {
        if (o.isCharacter && o.team != this.team && !o.isDead())
          o.damage(1, this);
      });
    } else this.angle = 0;

    if (this.climbingLadder) {
      this.gravityScale = this.climbingWall = this.groundObject = 0;
      this.jumpTimer.unset();
      this.groundTimer.unset();
      this.velocity = this.velocity
        .multiply(vec2(0.85))
        .add(vec2(0, 0.02 * moveInput.y));

      const delta = (this.pos.x | 0) + 0.5 - this.pos.x;
      this.velocity.x += 0.02 * delta * abs(moveInput.x ? 0 : moveInput.y);
      moveInput.x *= 0.2;

      // exit ladder if ground is below
      this.climbingLadder =
        moveInput.y >= 0 ||
        getTileCollisionData(this.pos.subtract(vec2(0, 1))) <= 0;
    } else {
      // update jumping and ground check
      if (this.groundObject || this.climbingWall) this.groundTimer.set(0.1);

      if (this.groundTimer.active() && !this.dodgeTimer.active()) {
        // is on ground
        if (
          this.pressedJumpTimer.active() &&
          !this.jumpTimer.active() &&
          !this.preventJumpTimer.active()
        ) {
          // start jump
          if (this.climbingWall) {
            this.velocity.y = 0.25;
          } else {
            this.velocity.y = 0.15;
            this.jumpTimer.set(0.2);
          }
          this.preventJumpTimer.set(0.5);
          playSound(fxJump, this.pos);
        }
      }

      if (this.jumpTimer.active() && !this.climbingWall) {
        // update variable height jump
        this.groundTimer.unset();
        if (this.holdingJump && this.velocity.y > 0 && this.jumpTimer.active())
          this.velocity.y += 0.017;
      }

      if (!this.groundObject) {
        // air control
        if (sign(moveInput.x) == sign(this.velocity.x))
          moveInput.x *= 0.1; // moving with velocity
        else moveInput.x *= 0.2; // moving against velocity (stopping)

        // slight extra gravity when moving down
        if (this.velocity.y < 0) this.velocity.y += gravity * 0.2;
      }
    }

    if (
      this.pressedDodge &&
      !this.dodgeTimer.active() &&
      !this.dodgeRechargeTimer.active()
    ) {
      // start dodge
      this.dodgeTimer.set(0.4);
      this.dodgeRechargeTimer.set(2);
      this.jumpTimer.unset();
      this.extinguish();
      playSound(fxDodge, this.pos);

      if (!this.groundObject && this.getAliveTime() > 0.2)
        this.velocity.y += 0.2;
    }

    // apply movement acceleration and clamp
    this.velocity.x = clamp(
      this.velocity.x + moveInput.x * 0.042,
      maxCharacterSpeed,
      -maxCharacterSpeed
    );

    // call parent, update physics
    const oldVelocity = this.velocity.copy();
    super.update();
    if (!this.isPlayer && !this.dodgeTimer.active()) {
      // apply collision damage
      const deltaSpeedSquared = this.velocity
        .subtract(oldVelocity)
        .lengthSquared();
      deltaSpeedSquared > 0.1 && this.damage(10 * deltaSpeedSquared);
    }

    if (
      this.climbingLadder ||
      (this.groundTimer.active() && !this.dodgeTimer.active())
    ) {
      const speed = this.velocity.length();
      this.walkCyclePercent += speed * 0.5;
      this.walkCyclePercent = speed > 0.01 ? mod(this.walkCyclePercent, 1) : 0;
    } else this.walkCyclePercent = 0;

    this.weapon.triggerIsDown = this.holdingShoot && !this.dodgeTimer.active();
    if (!this.dodgeTimer.active()) {
      if (
        this.grenadeCount > 0 &&
        this.pressingThrow &&
        !this.wasPressingThrow &&
        !this.grendeThrowTimer.active()
      ) {
        // throw greande
        --this.grenadeCount;
        const grenade = new Grenade(this.pos);
        grenade.velocity = this.velocity.add(
          vec2(this.getMirrorSign(), rand(0.8, 0.7)).normalize(
            0.25 + rand(0.02)
          )
        );
        grenade.angleVelocity = this.getMirrorSign() * rand(0.8, 0.5);
        playSound(fxJump, this.pos);
        if (!GOD_MODE) this.grendeThrowTimer.set(1);
      }
      this.wasPressingThrow = this.pressingThrow;
    }

    // update mirror
    if (this.moveInput.x && !this.dodgeTimer.active())
      this.mirror = this.moveInput.x < 0;

    // clamp x pos
    this.pos.x = clamp(this.pos.x, levelSize.x - 2, 2);

    // randomly blink
    rand() < 0.005 && this.blinkTimer.set(rand(0.2, 0.1));
  }

  render() {
    if (!isOverlapping(this.pos, this.size, cameraPos, renderWindowSize))
      return;

    // set tile to use
    this.tileIndex = this.isDead()
      ? this.bodyTile
      : this.climbingLadder || this.groundTimer.active()
      ? (this.bodyTile + 2 * this.walkCyclePercent) | 0
      : this.bodyTile + 1;

    let additive = this.additiveColor.add(this.extraAdditiveColor);
    if (
      this.isPlayer &&
      !this.isDead() &&
      this.dodgeRechargeTimer.elapsed() &&
      this.dodgeRechargeTimer.get() < 0.2
    ) {
      const v = 0.6 - this.dodgeRechargeTimer.get() * 3;
      additive = additive.add(new Color(0, v, v, 0)).clamp();
    }

    const sizeScale = this.sizeScale;
    const color = this.color.scale(this.burnColorPercent(), 1);
    const eyeColor = this.eyeColor.scale(this.burnColorPercent(), 1);

    const bodyPos = this.pos.add(
      vec2(0, -0.1 + 0.06 * Math.sin(this.walkCyclePercent * PI)).scale(
        sizeScale
      )
    );
    drawTile(
      bodyPos,
      vec2(sizeScale),
      this.tileIndex,
      this.tileSize,
      color,
      this.angle,
      this.mirror,
      additive
    );
    drawTile(
      this.pos.add(
        vec2(this.getMirrorSign(0.05), 0.46)
          .scale(sizeScale)
          .rotate(-this.angle)
      ),
      vec2(sizeScale / 2),
      this.headTile,
      vec2(8),
      color,
      this.angle,
      this.mirror,
      additive
    );

    //for(let i = this.grenadeCount; i--;)
    //    drawTile(bodyPos, vec2(.5), 5, vec2(8), new Color, this.angle, this.mirror, additive);

    const blinkScale = this.canBlink
      ? this.isDead()
        ? 0.3
        : 0.5 + 0.5 * Math.cos(this.blinkTimer.getPercent() * PI * 2)
      : 1;
    drawTile(
      this.pos.add(
        vec2(this.getMirrorSign(0.05), 0.46)
          .scale(sizeScale)
          .rotate(-this.angle)
      ),
      vec2(sizeScale / 2, (blinkScale * sizeScale) / 2),
      this.headTile + 1,
      vec2(8),
      eyeColor,
      this.angle,
      this.mirror,
      this.additiveColor
    );
  }

  damage(damage, damagingObject) {
    if (this.destroyed) return;

    if (this.team == team_player) {
      // safety window after spawn
      if (GOD_MODE || this.getAliveTime() < 2) return;
    }

    if (this.isDead() && !this.persistent) {
      this.overkill += damage;
      if (this.overkill > 5) {
        makeBlood(this.pos, 300);
        this.destroy();
      }
    }

    this.blinkTimer.set(rand(0.5, 0.4));
    makeBlood(damagingObject ? damagingObject.pos : this.pos);
    super.damage(damage, damagingObject);
  }

  kill(damagingObject) {
    if (this.isDead()) return 0;

    if (levelWarmup) {
      this.destroy();
      return 1;
    }

    this.deadTimer.set();
    this.size = this.size.scale(0.5);

    makeBlood(this.pos, 300);
    playSound(fxDie, this.pos);

    this.team = team_none;
    this.health = 0;
    const fallDirection = damagingObject
      ? sign(damagingObject.velocity.x)
      : randSign();
    this.angleVelocity = fallDirection * rand(0.22, 0.14);
    this.angleDamping = 0.9;
    this.weapon && this.weapon.destroy();

    // move to back layer
    this.renderOrder = 1;
  }

  collideWithTile(data, pos) {
    if (!data) return;

    if (data == tileType_ladder) {
      if (pos.y + 1 > this.lastPos.y - this.size.y * 0.5) return;

      if (
        getTileCollisionData(pos.add(vec2(0, 1))) && // above
        !(
          getTileCollisionData(pos.add(vec2(1, 0))) && // left
          getTileCollisionData(pos.add(vec2(1, 0)))
        ) // right
      )
        return; // dont collide if something above it and nothing to left or right

      // allow standing on top of ladders
      return !this.climbingLadder;
    }

    // break blocks above
    const d = pos.y - this.pos.y;
    if (
      !this.climbingLadder &&
      this.velocity.y > 0.1 &&
      d > 0 &&
      d < this.size.y * 0.5
    ) {
      if (destroyTile(pos)) {
        this.velocity.y = 0;
        return;
      }
    }

    return 1;
  }

  collideWithObject(o) {
    if (this.isDead()) return super.collideWithObject(o);

    if (o.velocity.lengthSquared() > 0.04) {
      const v = o.velocity.subtract(this.velocity);
      const m = 25 * o.mass * v.lengthSquared();
      if (
        !o.groundObject &&
        o.isCrushing &&
        !this.persistent &&
        o.velocity.y < 0 &&
        this.pos.y < o.pos.y - o.size.y / 2 &&
        abs(o.pos.x - this.pos.x) < o.size.x * 0.5
      ) {
        // crushing
        this.damage(1e3, o);
        if (this.isDead()) {
          makeBlood(this.pos, 300);
          this.destroy();
        }
      } else if (m > 1) this.damage((4 * m) | 0, o);
    }

    return super.collideWithObject(o);
  }
}

const type_weak = 0;
const type_normal = 1;
const type_strong = 2;
const type_elite = 3;
const type_grenade = 4;
const type_count = 5;

function alertEnemies(pos, playerPos) {
  const radius = 4;
  forEachObject(pos, radius, (o) => {
    o.team == team_enemy && o.alert && o.alert(playerPos);
  });
}

class Enemy extends Character {
  constructor(pos) {
    super(pos);

    this.team = team_enemy;
    this.sawPlayerTimer = new Timer();
    this.reactionTimer = new Timer();
    this.facePlayerTimer = new Timer();
    this.holdJumpTimer = new Timer();
    this.shootTimer = new Timer();
    this.maxVisionRange = 12;

    this.type = (randSeeded() ** 3 * min(level + 1, type_count)) | 0;

    let health = 1 + this.type;
    this.eyeColor = new Color(1, 0.5, 0);
    if (this.type == type_weak) {
      this.color = new Color(0, 1, 0);
      this.size = this.size.scale((this.sizeScale = 0.9));
    } else if (this.type == type_normal) {
      this.color = new Color(0, 0.4, 1);
    } else if (this.type == type_strong) {
      this.color = new Color(1, 0, 0);
      this.eyeColor = new Color(1, 1, 0);
    } else if (this.type == type_elite) {
      this.color = new Color(1, 1, 1);
      this.eyeColor = new Color(1, 0, 0);
      this.maxVisionRange = 15;
    } else if (this.type == type_grenade) {
      this.color = new Color(0.7, 0, 1);
      this.eyeColor = new Color(0, 0, 0);
      this.grenadeCount = 3;
      this.canBurn = 0;
    }

    if ((this.isBig = randSeeded() < 0.05)) {
      // chance of large enemy with extra health
      this.size = this.size.scale((this.sizeScale = 1.3));
      health *= 2;
      this.grenadeCount *= 10;
      this.maxVisionRange = 15;
      --levelEnemyCount;
    }

    this.health = this.healthMax = health;
    this.color = this.color.mutate();
    this.mirror = rand() < 0.5;

    new Weapon(this.pos, this);
    --levelEnemyCount;

    this.sightCheckFrame = rand(9) | 0;
  }

  update() {
    if (!aiEnable || levelWarmup || this.isDead() || !this.inUpdateWindow()) {
      if (this.weapon) this.weapon.triggerIsDown = 0;
      super.update();
      return; // ignore offscreen objects
    }

    if (this.weapon)
      this.weapon.localPos = this.weapon.localOffset.scale(this.sizeScale);

    // update check if players are visible
    const sightCheckFrames = 9;
    if (frame % sightCheckFrames == this.sightCheckFrame) {
      const sawRecently =
        this.sawPlayerTimer.isSet() && this.sawPlayerTimer.get() < 5;
      const visionRangeSquared =
        (sawRecently ? this.maxVisionRange * 1.2 : this.maxVisionRange) ** 2;
      for (const player of players) {
        // check range
        if (player && !player.isDead())
          if (
            sawRecently ||
            this.getMirrorSign() == sign(player.pos.x - this.pos.x)
          )
            if (
              sawRecently ||
              abs(player.pos.x - this.pos.x) > abs(player.pos.y - this.pos.y)
            )
              if (this.pos.distanceSquared(player.pos) < visionRangeSquared) {
                // 45 degree slope
                const raycastHit = tileCollisionRaycast(this.pos, player.pos);
                if (!raycastHit) {
                  this.alert(player.pos, 1);
                  break;
                }
              }
      }

      if (sawRecently) {
        // alert nearby enemies
        alertEnemies(this.pos, this.sawPlayerPos);
      }
    }

    this.pressedDodge = this.climbingWall = this.pressingThrow = 0;

    if (this.burnTimer.isSet()) {
      // burning, run around
      this.facePlayerTimer.unset();

      // random jump
      if (rand() < 0.005) {
        this.pressedJumpTimer.set(0.05);
        this.holdJumpTimer.set(rand(0.05));
      }

      // random movement
      if (rand() < 0.05) this.moveInput.x = randSign() * rand(0.6, 0.3);
      this.moveInput.y = 0;

      // random dodge
      if (this.type == type_elite) this.pressedDodge = 1;
      else if (this.groundObject) this.pressedDodge = rand() < 0.005;
    } else if (this.sawPlayerTimer.isSet() && this.sawPlayerTimer.get() < 10) {
      // wall climb
      if (
        this.type >= type_strong &&
        this.moveInput.x &&
        !this.velocity.x &&
        this.velocity.y < 0
      ) {
        this.velocity.y *= 0.8;
        this.climbingWall = 1;
        this.pressedJumpTimer.set(0.1);
        this.holdJumpTimer.set(rand(0.2));
      }

      const timeSinceSawPlayer = this.sawPlayerTimer.get();
      this.weapon.localAngle *= 0.8;
      if (this.reactionTimer.active()) {
        // just saw player for first time, act surprised
        this.moveInput.x = 0;
      } else if (timeSinceSawPlayer < 5) {
        if (!this.dodgeTimer.active()) {
          const playerDirection = sign(this.sawPlayerPos.x - this.pos.x);
          if (
            this.type == type_grenade &&
            rand() < 0.002 &&
            this.getMirrorSign() == playerDirection
          )
            this.pressingThrow = 1;

          // actively fighting player
          if (rand() < 0.05) this.facePlayerTimer.set(rand(2, 0.5));

          // random jump
          if (rand() < (this.type < type_strong ? 0.0005 : 0.005)) {
            this.pressedJumpTimer.set(0.1);
            this.holdJumpTimer.set(rand(0.2));
          }

          // random movement
          if (rand() < (this.isBig ? 0.05 : 0.02)) this.moveInput.x = 0;
          else if (rand() < 0.01)
            this.moveInput.x =
              rand() < 0.6
                ? playerDirection * rand(0.5, 0.2)
                : -playerDirection * rand(0.4, 0.2);
          if (rand() < 0.03)
            this.moveInput.y = rand() < 0.5 ? 0 : randSign() * rand(0.4, 0.2);

          // random shoot
          if (abs(this.sawPlayerPos.y - this.pos.y) < 4)
            if (!this.shootTimer.isSet() || this.shootTimer.get() > 1)
              rand() < (this.type > type_weak ? 0.02 : 0.01) &&
                this.shootTimer.set(this.isBig ? rand(2, 1) : 0.05);
        }

        // random dodge
        if (this.type == type_elite)
          this.pressedDodge = rand() < 0.01 && timeSinceSawPlayer < 0.5;
      } else {
        // was fighting but lost player

        if (rand() < 0.04) this.facePlayerTimer.set(rand(2, 0.5));

        // random movement
        if (rand() < 0.02) this.moveInput.x = 0;
        else if (rand() < 0.01) this.moveInput.x = randSign() * rand(0.4, 0.2);

        // random jump
        if (rand() < (this.sawPlayerPos.y > this.pos.y ? 0.002 : 0.001)) {
          this.pressedJumpTimer.set(0.1);
          this.holdJumpTimer.set(rand(0.2));
        }

        // random shoot
        if (!this.shootTimer.isSet() || this.shootTimer.get() > 5)
          rand() < 0.001 && this.shootTimer.set(rand(0.2, 0.1));

        // move up/down in dirction last player was seen
        this.moveInput.y = clamp(this.sawPlayerPos.y - this.pos.y, 0.5, -0.5);
      }
    } else {
      // try to act normal
      if (rand() < 0.03) this.moveInput.x = 0;
      else if (rand() < 0.005) this.moveInput.x = randSign() * rand(0.2, 0.1);
      else if (rand() < 0.001) this.moveInput.x = randSign() * 1e-9; // hack: look in a direction

      this.weapon.localAngle = lerp(0.1, 0.7, this.weapon.localAngle);
      this.reactionTimer.unset();
    }

    if (this.isBig && this.type != type_elite) {
      // big enemies cant jump
      this.pressedJumpTimer.unset();
      this.holdJumpTimer.unset();
    }
    this.holdingShoot = this.shootTimer.active();
    this.holdingJump = this.holdJumpTimer.active();

    super.update();

    // override default mirror
    if (
      this.facePlayerTimer.active() &&
      !this.dodgeTimer.active() &&
      !this.reactionTimer.active()
    )
      this.mirror = this.sawPlayerPos.x < this.pos.x;
  }

  alert(playerPos, resetSawPlayer) {
    if (resetSawPlayer || !this.sawPlayerTimer.isSet()) {
      if (!this.reactionTimer.isSet()) {
        this.reactionTimer.set(rand(1, 0.5) * (this.type == type_weak ? 2 : 1));
        this.facePlayerTimer.set(rand(2, 1));
        if (this.groundObject && rand() < 0.2) this.velocity.y += 0.1; // random jump
      }

      this.sawPlayerTimer.set();
      this.sawPlayerPos = playerPos;
    }
  }

  damage(damage, damagingObject) {
    super.damage(damage, damagingObject);
    if (!this.isDead()) {
      this.alert(
        damagingObject
          ? damagingObject.pos.subtract(damagingObject.velocity.normalize())
          : this.pos,
        1
      );
      this.reactionTimer.set(rand(1, 0.5));
      this.shootTimer.unset();
    }
  }

  kill(damagingObject) {
    if (this.isDead()) return 0;

    super.kill(damagingObject);
    levelWarmup || ++totalKills;
  }
}

class Player extends Character {
  constructor(pos, playerIndex = 0) {
    super(pos);

    this.grenadeCount = GOD_MODE ? 99999 : 3;
    this.burnTime = 2;

    this.eyeColor = new Color().setHSLA(-playerIndex * 0.6, 1, 0.5);
    if (playerIndex) {
      this.color = new Color().setHSLA(playerIndex * 0.3 - 0.3, 0.5, 0.5);
      this.extraAdditiveColor = new Color().setHSLA(
        playerIndex * 0.3 - 0.3,
        1,
        0.1,
        0
      );
    }

    this.bodyTile = 5;
    this.headTile = 18;
    this.playerIndex = playerIndex;
    this.renderOrder = 20 + 10 * playerIndex;
    this.walkSoundTime = 0;
    this.persistent = this.wasHoldingJump = this.canBlink = this.isPlayer = 1;
    this.team = team_player;

    new Weapon(this.pos, this);
    players[playerIndex] = this;

    // small jump on spawn
    this.velocity.y = 0.2;
    this.mirror = playerIndex % 2;
    --playerLives;
  }

  update() {
    if (this.isDead()) {
      if (this.persistent && playerLives) {
        if (players.length == 1) {
          if (this.deadTimer.get() > 2) {
            this.persistent = 0;
            new Player(checkpointPos, this.playerIndex);
            playSound(fxJump, cameraPos);
          }
        } else {
          // respawn only if all players dead, or checkpoint touched
          let hasLivingPlayers = 0;
          let minDeadTime = 1e3;
          for (const player of players) {
            if (player) {
              minDeadTime = min(
                minDeadTime,
                player.isDead() ? player.deadTimer.get() : 1e3
              );
              hasLivingPlayers |=
                !player.isDead() && player.getAliveTime() > 0.1;
            }
          }

          if (minDeadTime > 2) {
            if (!hasLivingPlayers) {
              // respawn all
              this.persistent = 0;
              new Player(
                checkpointPos.add(vec2(1 - this.playerIndex / 2, 0)),
                this.playerIndex
              );
              this.playerIndex || playSound(fxJump, cameraPos);
            } else if (checkpointTimer.active()) {
              // respawn if checkpoint active
              this.persistent = 0;
              const player = new Player(checkpointPos, this.playerIndex);
              playSound(fxJump, cameraPos);
            }
          }
        }
      }

      super.update();
      return;
    }

    // wall climb
    this.climbingWall = 0;
    if (this.moveInput.x && !this.velocity.x && this.velocity.y < 0) {
      this.velocity.y *= 0.8;
      this.climbingWall = 1;
    }

    // movement control
    this.moveInput.x =
      isUsingGamepad || this.playerIndex
        ? gamepadStick(0, this.playerIndex).x
        : keyIsDown(39) - keyIsDown(37);

    this.moveInput.y =
      isUsingGamepad || this.playerIndex
        ? gamepadStick(0, this.playerIndex).y
        : keyIsDown(38) - keyIsDown(40);

    // jump
    this.holdingJump =
      (!this.playerIndex && keyIsDown(38)) ||
      gamepadIsDown(0, this.playerIndex);
    if (!this.holdingJump) this.pressedJumpTimer.unset();
    else if (!this.wasHoldingJump || this.climbingWall)
      this.pressedJumpTimer.set(0.3);
    this.wasHoldingJump = this.holdingJump;

    // controls
    this.holdingShoot =
      (!this.playerIndex && (mouseIsDown(0) || keyIsDown(90))) ||
      gamepadIsDown(2, this.playerIndex);
    this.pressingThrow =
      (!this.playerIndex && (mouseIsDown(2) || keyIsDown(67))) ||
      gamepadIsDown(1, this.playerIndex);
    this.pressedDodge =
      (!this.playerIndex && (mouseIsDown(1) || keyIsDown(88))) ||
      gamepadIsDown(3, this.playerIndex);

    super.update();

    // update walk sound
    this.walkSoundTime += abs(this.velocity.x);
    if (
      abs(this.velocity.x) > 0.01 &&
      this.groundTimer.active() &&
      !this.dodgeTimer.active()
    ) {
      if (this.walkSoundTime > 1) {
        this.walkSoundTime = 0;
        playSound(fxWalk, this.pos);
      }
    } else this.walkSoundTime = 0.5;

    if (players.length > 1 && !this.isDead()) {
      // move to other player if offscreen and multiplayer
      if (!isOverlapping(this.pos, this.size, cameraPos, gameplayWindowSize)) {
        // move to location of another player if not falling off a cliff
        if (tileCollisionRaycast(this.pos, vec2(this.pos.x, 0))) {
          for (const player of players)
            if (player && player != this && !player.isDead()) {
              this.pos = player.pos.copy();
              this.velocity = vec2();
              playSound(fxJump, this.pos);
            }
        } else this.kill();
      }
    }
  }
}

const precipitationEnable = 1;

// sounds
// let fxRain = [0.02, , 1e3, 2, , 2, , , , , , , , 99];
// let fxWind = [0.01, 0.3, 2e3, 2, 1, 2, , , , , , , 1, 2, , , , , , 0.1];
let fxShoot = [, , 90, , 0.01, 0.03, 4, , , , , , , 9, 50, 0.2, , 0.2, 0.01];
let fxDie = [0.5, 0.4, 126, 0.05, , 0.2, 1, 2.09, , -4, , , 1, 1, 1, 0.4, 0.03];
let fxJump = [0.4, 0.2, 250, 0.04, , 0.04, , , 1, , , , , 3];
let fxDodge = [0.4, 0.2, 150, 0.05, , 0.05, , , -1, , , , , 4, , , , , 0.02];
let fxWalk = [0.3, 0.1, 70, , , 0.01, 4, , , , -9, 0.1, , , , , , 0.5];
let fxGrenade = [0.5, 0.01, 300, , , 0.02, 3, 0.22, , , -9, 0.2, , , , , , 0.5];
let fxCheckPoint =
  (0, [0.6, 0, 500, , 0.04, 0.3, 1, 2, , , 570, 0.02, 0.02, , , , 0.04]);
let fxDestroy =
  (0, [0.5, , 1e3, 0.02, , 0.2, 1, 3, 0.1, , , , , 1, -30, 0.5, , 0.5]);
let fxExplosion =
  (0, [2, 0.2, 72, 0.01, 0.01, 0.2, 4, , , , , , , 1, , 0.5, 0.1, 0.5, 0.02]);

// special effects

const persistentParticleDestroyCallback = (particle) => {
  // copy particle to tile layer on death
  if (particle.groundObject)
    tileLayer.drawTile(
      particle.pos,
      particle.size,
      particle.tileIndex,
      particle.tileSize,
      particle.color,
      particle.angle,
      particle.mirror
    );
};

function makeBlood(pos, amount = 50) {
  const emitter = new ParticleEmitter(
    pos,
    1,
    0.1,
    amount,
    PI, // pos, emitSize, emitTime, emitRate, emiteCone
    undefined,
    undefined, // tileIndex, tileSize
    new Color(1, 0, 0),
    new Color(0.5, 0, 0), // colorStartA, colorStartB
    new Color(1, 0, 0),
    new Color(0.5, 0, 0), // colorEndA, colorEndB
    3,
    0.1,
    0.1,
    0.1,
    0.1, // particleTime, sizeStart, sizeEnd, particleSpeed, particleAngleSpeed
    1,
    0.95,
    0.7,
    PI,
    0, // damping, angleDamping, gravityScale, particleCone, fadeRate,
    0.5,
    1 // randomness, collide, additive, randomColorLinear, renderOrder
  );
  emitter.particleDestroyCallback = persistentParticleDestroyCallback;
  return emitter;
}

function makeFire(pos = vec2()) {
  return new ParticleEmitter(
    pos,
    1,
    0,
    60,
    PI, // pos, emitSize, emitTime, emitRate, emiteCone
    0,
    undefined, // tileIndex, tileSize
    new Color(1, 1, 0),
    new Color(1, 0.5, 0.5), // colorStartA, colorStartB
    new Color(1, 0, 0),
    new Color(1, 0.5, 0.1), // colorEndA, colorEndB
    0.5,
    0.5,
    0.1,
    0.01,
    0.1, // particleTime, sizeStart, sizeEnd, particleSpeed, particleAngleSpeed
    0.95,
    0.1,
    -0.05,
    PI,
    0.5, // damping, angleDamping, gravityScale, particleCone, fadeRate,
    0.5,
    0,
    1
  ); // randomness, collide, additive, randomColorLinear, renderOrder
}

function makeDebris(pos, color = new Color(), amount = 100) {
  const color2 = color.lerp(new Color(), 0.5);
  const emitter = new ParticleEmitter(
    pos,
    1,
    0.1,
    amount,
    PI, // pos, emitSize, emitTime, emitRate, emiteCone
    undefined,
    undefined, // tileIndex, tileSize
    color,
    color2, // colorStartA, colorStartB
    color,
    color2, // colorEndA, colorEndB
    3,
    0.2,
    0.2,
    0.1,
    0.05, // particleTime, sizeStart, sizeEnd, particleSpeed, particleAngleSpeed
    1,
    0.95,
    0.4,
    PI,
    0, // damping, angleDamping, gravityScale, particleCone, fadeRate,
    0.5,
    1 // randomness, collide, additive, randomColorLinear, renderOrder
  );
  emitter.elasticity = 0.3;
  emitter.particleDestroyCallback = persistentParticleDestroyCallback;
  return emitter;
}

function makeWater(pos) {
  // overall spray
  new ParticleEmitter(
    pos,
    1,
    0.05,
    400,
    PI, // pos, emitSize, emitTime, emitRate, emiteCone
    0,
    undefined, // tileIndex, tileSize
    new Color(1, 1, 1, 0.5),
    new Color(0.5, 1, 1, 0.2), // colorStartA, colorStartB
    new Color(1, 1, 1, 0.5),
    new Color(0.5, 1, 1, 0.2), // colorEndA, colorEndB
    0.5,
    0.5,
    2,
    0.1,
    0.05, // particleTime, sizeStart, sizeEnd, particleSpeed, particleAngleSpeed
    0.9,
    1,
    0,
    PI,
    0.5, // damping, angleDamping, gravityScale, particleCone, fadeRate,
    0.5,
    0,
    0,
    0,
    1e9 // randomness, collide, additive, randomColorLinear, renderOrder
  );

  // droplets
  const emitter = new ParticleEmitter(
    pos,
    1,
    0.1,
    400,
    PI, // pos, emitSize, emitTime, emitRate, emiteCone
    0,
    undefined, // tileIndex, tileSize
    new Color(0.8, 1, 1, 0.6),
    new Color(0.5, 0.5, 1, 0.2), // colorStartA, colorStartB
    new Color(0.8, 1, 1, 0.6),
    new Color(0.5, 0.5, 1, 0.2), // colorEndA, colorEndB
    2,
    0.1,
    0.1,
    0.2,
    0, // particleTime, sizeStart, sizeEnd, particleSpeed, particleAngleSpeed
    0.99,
    1,
    0.5,
    PI,
    0.2, // damping, angleDamping, gravityScale, particleCone, fadeRate,
    0.5,
    1 // randomness, collide, additive, randomColorLinear, renderOrder
  );
  emitter.elasticity = 0.2;
  emitter.trailScale = 2;

  // put out fires
  const radius = 3;
  forEachObject(pos, 3, (o) => {
    if (o.isGameObject) {
      o.burnTimer.isSet() && o.extinguish();
      const d = o.pos.distance(pos);
      const p = percent(d, radius / 2, radius);
      const force = o.pos.subtract(pos).normalize(p * radius * 0.2);
      o.applyForce(force);
      if (o.isDead && o.isDead())
        o.angleVelocity += randSign() * rand(radius / 4, 0.3);
    }
  });

  return emitter;
}

function explosion(pos, radius = 2) {
  if (levelWarmup) return;

  const damage = radius * 2;

  // destroy level
  for (let x = -radius; x < radius; ++x) {
    const h = (radius ** 2 - x ** 2) ** 0.5;
    for (let y = -h; y <= h; ++y) destroyTile(pos.add(vec2(x, y)), 0, 0);
  }

  // cleanup neighbors
  const cleanupRadius = radius + 1;
  for (let x = -cleanupRadius; x < cleanupRadius; ++x) {
    const h = (cleanupRadius ** 2 - x ** 2) ** 0.5;
    for (let y = -h; y < h; ++y) decorateTile(pos.add(vec2(x, y)).int());
  }

  // kill/push objects
  const maxRangeSquared = (radius * 1.5) ** 2;
  forEachObject(pos, radius * 3, (o) => {
    const d = o.pos.distance(pos);
    if (o.isGameObject) {
      // do damage
      d < radius && o.damage(damage);

      // catch fire
      d < radius * 1.5 && o.burn();
    }

    // push
    const p = percent(d, radius, 2 * radius);
    const force = o.pos.subtract(pos).normalize(p * radius * 0.2);
    o.applyForce(force);
    if (o.isDead && o.isDead())
      o.angleVelocity += randSign() * rand((p * radius) / 4, 0.3);
  });

  playSound(fxExplosion, pos);

  // smoke
  new ParticleEmitter(
    pos,
    radius / 2,
    0.2,
    50 * radius,
    PI, // pos, emitSize, emitTime, emitRate, emiteCone
    0,
    undefined, // tileIndex, tileSize
    new Color(0, 0, 0),
    new Color(0, 0, 0), // colorStartA, colorStartB
    new Color(0, 0, 0, 0),
    new Color(0, 0, 0, 0), // colorEndA, colorEndB
    1,
    0.5,
    2,
    0.1,
    0.05, // particleTime, sizeStart, sizeEnd, particleSpeed, particleAngleSpeed
    0.9,
    1,
    -0.3,
    PI,
    0.1, // damping, angleDamping, gravityScale, particleCone, fadeRate,
    0.5,
    0,
    0,
    0,
    1e8 // randomness, collide, additive, randomColorLinear, renderOrder
  );

  // fire
  new ParticleEmitter(
    pos,
    radius / 2,
    0.1,
    100 * radius,
    PI, // pos, emitSize, emitTime, emitRate, emiteCone
    0,
    undefined, // tileIndex, tileSize
    new Color(1, 0.5, 0.1),
    new Color(1, 0.1, 0.1), // colorStartA, colorStartB
    new Color(1, 0.5, 0.1, 0),
    new Color(1, 0.1, 0.1, 0), // colorEndA, colorEndB
    0.5,
    0.5,
    2,
    0.1,
    0.05, // particleTime, sizeStart, sizeEnd, particleSpeed, particleAngleSpeed
    0.9,
    1,
    0,
    PI,
    0.05, // damping, angleDamping, gravityScale, particleCone, fadeRate,
    0.5,
    0,
    1,
    0,
    1e9 // randomness, collide, additive, randomColorLinear, renderOrder
  );
}

class TileCascadeDestroy extends EngineObject {
  constructor(pos, cascadeChance = 1, glass = 0) {
    super(pos, vec2());
    this.cascadeChance = cascadeChance;
    this.destroyTimer = new Timer(glass ? 0.05 : rand(0.3, 0.1));
  }

  update() {
    if (this.destroyTimer.elapsed()) {
      destroyTile(this.pos, 1, 1, this.cascadeChance);
      this.destroy();
    }
  }
}

function decorateBackgroundTile(pos) {
  const tileData = getTileBackgroundData(pos);
  if (tileData <= 0) return; // no need to clear if background cant change

  // round corners
  for (let i = 4; i--; ) {
    // check corner neighbors
    const neighborTileDataA = getTileBackgroundData(
      pos.add(vec2().setAngle((i * PI) / 2))
    );
    const neighborTileDataB = getTileBackgroundData(
      pos.add(vec2().setAngle((((i + 1) % 4) * PI) / 2))
    );

    if ((neighborTileDataA > 0) | (neighborTileDataB > 0)) continue;

    const directionVector = vec2()
      .setAngle((i * PI) / 2 + PI / 4, 10)
      .int();
    let drawPos = pos
      .add(vec2(0.5)) // center
      .scale(16)
      .add(directionVector)
      .int(); // direction offset

    // clear rect without any scaling to prevent blur from filtering
    const s = 2;
    tileBackgroundLayer.context.clearRect(
      (drawPos.x - s / 2) | 0,
      (tileBackgroundLayer.canvas.height - drawPos.y - s / 2) | 0,
      s | 0,
      s | 0
    );
  }
}

function decorateTile(pos) {
  const tileData = getTileCollisionData(pos);
  if (tileData <= 0) {
    tileData || tileLayer.setData(pos, new TileLayerData(), 1); // force it to clear if it is empty
    return;
  }

  if (
    (tileData != tileType_dirt) &
    (tileData != tileType_base) &
    (tileData != tileType_pipeV) &
    (tileData != tileType_pipeH) &
    (tileData != tileType_solid)
  )
    return;

  for (let i = 4; i--; ) {
    // outline towards neighbors of differing type
    const neighborTileData = getTileCollisionData(
      pos.add(vec2().setAngle((i * PI) / 2))
    );
    if (neighborTileData == tileData) continue;

    // hacky code to make pixel perfect outlines
    let size = tileData == tileType_dirt ? vec2(rand(16, 8), 2) : vec2(16, 1);
    i & 1 && (size = size.flip());

    const color =
      tileData == tileType_dirt
        ? levelGroundColor.mutate(0.1)
        : new Color(0.1, 0.1, 0.1);
    tileLayer.context.fillStyle = color.rgba();
    const drawPos = pos.scale(16);
    if (tileData == tileType_dirt)
      tileLayer.context.fillRect(
        (drawPos.x + ((i == 1 ? 14 : 0) + (i & 1 ? 0 : 8 - size.x / 2))) | 0,
        (tileLayer.canvas.height -
          drawPos.y +
          ((i == 0 ? -14 : 0) - (i & 1 ? 8 - size.y / 2 : 0))) |
          0,
        size.x | 0,
        -size.y | 0
      );
    else
      tileLayer.context.fillRect(
        (drawPos.x + (i == 1 ? 15 : 0)) | 0,
        (tileLayer.canvas.height - drawPos.y + (i == 0 ? -15 : 0)) | 0,
        size.x | 0,
        -size.y | 0
      );
  }
}

function destroyTile(
  pos,
  makeSound = 1,
  cleanNeighbors = 1,
  maxCascadeChance = 1
) {
  // pos must be an int
  pos = pos.int();

  // destroy tile
  const tileType = getTileCollisionData(pos);

  if (!tileType) return 1; // empty
  if (tileType == tileType_solid) return 0; // indestructable

  const centerPos = pos.add(vec2(0.5));
  const layerData = tileLayer.getData(pos);
  if (layerData) {
    makeDebris(centerPos, layerData.color.mutate());
    makeSound && playSound(fxDestroy, centerPos);

    setTileCollisionData(pos, tileType_empty);
    tileLayer.setData(pos, new TileLayerData(), 1); // set and clear tile

    // cleanup neighbors
    if (cleanNeighbors) {
      for (let i = -1; i <= 1; ++i)
        for (let j = -1; j <= 1; ++j) decorateTile(pos.add(vec2(i, j)));
    }

    // if weak earth, random chance of delayed destruction of tile directly above
    if (tileType == tileType_glass) {
      maxCascadeChance = 1;
      if (getTileCollisionData(pos.add(vec2(0, -1))) == tileType)
        new TileCascadeDestroy(pos.add(vec2(0, -1)), 1, 1);
    } else if (tileType != tileType_dirt) maxCascadeChance = 0;

    if (
      rand() < maxCascadeChance &&
      getTileCollisionData(pos.add(vec2(0, 1))) == tileType
    )
      new TileCascadeDestroy(
        pos.add(vec2(0, 1)),
        maxCascadeChance * 0.4,
        tileType == tileType_glass
      );
  }

  return 1;
}

function drawStars() {
  randSeed = levelSeed;
  for (let i = 400; i--; ) {
    let size = randSeeded(6, 1);
    let speed = randSeeded() < 0.9 ? randSeeded(5) : randSeeded(99, 9);
    let color = new Color().setHSLA(
      randSeeded(0.2, -0.3),
      randSeeded() ** 9,
      randSeeded(1, 0.5),
      randSeeded(0.9, 0.3)
    );
    if (i < 9) {
      // suns or moons
      size = randSeeded() ** 3 * 99 + 9;
      speed = randSeeded(5);
      color = new Color()
        .setHSLA(randSeeded(), randSeeded(), randSeeded(1, 0.5))
        .add(levelSkyColor.scale(0.5))
        .clamp();
    }

    const w = mainCanvas.width + 400,
      h = mainCanvas.height + 400;
    const screenPos = vec2(
      ((randSeeded(w) + time * speed) % w) - 200,
      ((randSeeded(h) + time * speed * randSeeded(1, 0.2)) % h) - 200
    );

    // if (lowGraphicsSettings) {
    // drawing stars with gl wont work in low graphics mode, just draw rects
    mainContext.fillStyle = color.rgba();
    if (size < 9) mainContext.fillRect(screenPos.x, screenPos.y, size, size);
    else
      mainContext.beginPath(
        mainContext.fill(mainContext.arc(screenPos.x, screenPos.y, size, 0, 9))
      );
    // }
    //  drawTileScreenSpace(screenPos, vec2(size), 0, vec2(16), color);
  }
}

let tileParallaxLayers = [];

function generateParallaxLayers() {
  tileParallaxLayers = [];
  for (let i = 0; i < 3; ++i) {
    const parallaxSize = vec2(600, 300),
      startGroundLevel = rand(99, 120) + i * 30;
    const tileParallaxLayer = (tileParallaxLayers[i] = new TileLayer(
      vec2(),
      parallaxSize
    ));
    let groundLevel = startGroundLevel,
      groundSlope = rand(1, -1);
    tileParallaxLayer.renderOrder = -3e3 + i;
    tileParallaxLayer.canvas.width = parallaxSize.x;

    const layerColor = levelColor
      .mutate(0.2)
      .lerp(levelSkyColor, 0.95 - i * 0.15);
    const gradient = (tileParallaxLayer.context.fillStyle =
      tileParallaxLayer.context.createLinearGradient(
        0,
        0,
        0,
        (tileParallaxLayer.canvas.height = parallaxSize.y)
      ));
    gradient.addColorStop(0, layerColor.rgba());
    gradient.addColorStop(
      1,
      layerColor.subtract(new Color(1, 1, 1, 0)).mutate(0.1).clamp().rgba()
    );

    for (let x = parallaxSize.x; x--; ) {
      // pull slope towards start ground level
      tileParallaxLayer.context.fillRect(
        x,
        (groundLevel += groundSlope =
          rand() < 0.05
            ? rand(1, -1)
            : groundSlope + (startGroundLevel - groundLevel) / 2e3),
        1,
        parallaxSize.y
      );
    }
  }
}

function updateParallaxLayers() {
  tileParallaxLayers.forEach((tileParallaxLayer, i) => {
    const distance = 4 + i;
    const parallax = vec2(150, 30).scale(i * i + 1);
    const cameraDeltaFromCenter = cameraPos
      .subtract(levelSize.scale(0.5))
      .divide(levelSize.scale(-0.5).divide(parallax));
    tileParallaxLayer.scale = vec2(distance / cameraScale);
    tileParallaxLayer.pos = cameraPos
      .subtract(
        tileParallaxLayer.size.multiply(tileParallaxLayer.scale).scale(0.5)
      )
      .add(cameraDeltaFromCenter.scale(1 / cameraScale))
      .subtract(vec2(0, 150 / cameraScale));
  });
}

const tileType_ladder = -1;
const tileType_empty = 0;
const tileType_solid = 1;
const tileType_dirt = 2;
const tileType_base = 3;
const tileType_pipeH = 4;
const tileType_pipeV = 5;
const tileType_glass = 6;
const tileType_baseBack = 7;
const tileType_window = 8;

const tileRenderOrder = -1e3;
const tileBackgroundRenderOrder = -2e3;

// level objects
let players = [],
  playerLives,
  tileLayer,
  tileBackgroundLayer,
  totalKills;

// level settings
let levelSize, level, levelSeed, levelEnemyCount, levelWarmup;
let levelColor,
  levelBackgroundColor,
  levelSkyColor,
  levelSkyHorizonColor,
  levelGroundColor;
let skyParticles,
  skyRain,
  skySoundTimer = new Timer();
let gameTimer = new Timer(),
  levelTimer = new Timer(),
  levelEndTimer = new Timer();

let tileBackground;
const setTileBackgroundData = (pos, data = 0) =>
  pos.arrayCheck(tileCollisionSize) &&
  (tileBackground[((pos.y | 0) * tileCollisionSize.x + pos.x) | 0] = data);
const getTileBackgroundData = (pos) =>
  pos.arrayCheck(tileCollisionSize)
    ? tileBackground[((pos.y | 0) * tileCollisionSize.x + pos.x) | 0]
    : 0;

// level generation

const resetGame = () => {
  levelEndTimer.unset();
  gameTimer.set((totalKills = level = 0));
  nextLevel((playerLives = 6));
};

function buildTerrain(size) {
  tileBackground = [];
  initTileCollision(size);
  let startGroundLevel = rand(40, 60);
  let groundLevel = startGroundLevel;
  let groundSlope = rand(0.5, -0.5);
  let canayonWidth = 0,
    backgroundDelta = 0,
    backgroundDeltaSlope = 0;
  for (let x = 0; x < size.x; x++) {
    // pull slope towards start ground level
    groundLevel += groundSlope =
      rand() < 0.05
        ? rand(0.5, -0.5)
        : groundSlope + (startGroundLevel - groundLevel) / 1e3;

    // small jump
    if (rand() < 0.04) groundLevel += rand(9, -9);

    if (rand() < 0.03) {
      // big jump
      const jumpDelta = rand(9, -9);
      startGroundLevel = clamp(startGroundLevel + jumpDelta, 80, 20);
      groundLevel += jumpDelta;
      groundSlope = rand(0.5, -0.5);
    }

    --canayonWidth;
    if (rand() < 0.005) canayonWidth = rand(7, 2);

    backgroundDelta += backgroundDeltaSlope;
    if (rand() < 0.1) backgroundDelta = rand(3, -1);
    if (rand() < 0.1) backgroundDelta = 0;
    if (rand() < 0.1) backgroundDeltaSlope = rand(1, -1);
    backgroundDelta = clamp(backgroundDelta, 3, -1);

    groundLevel = clamp(groundLevel, 99, 30);
    for (let y = 0; y < size.y; y++) {
      const pos = vec2(x, y);

      let frontTile = tileType_empty;
      if (y < groundLevel && canayonWidth <= 0) frontTile = tileType_dirt;

      let backTile = tileType_empty;
      if (y < groundLevel + backgroundDelta) backTile = tileType_dirt;

      setTileCollisionData(pos, frontTile);
      setTileBackgroundData(pos, backTile);
    }
  }

  // add random holes
  for (let i = levelSize.x; i--; ) {
    const pos = vec2(rand(levelSize.x), rand(levelSize.y - 19, 19));
    for (let x = rand(9, 1) | 0; --x; )
      for (let y = rand(9, 1) | 0; --y; )
        setTileCollisionData(pos.add(vec2(x, y)), tileType_empty);
  }
}

function spawnProps(pos) {
  if (abs(checkpointPos.x - pos.x) > 5) {
    new Prop(pos);
    const propPlaceSize = 0.51;
    if (randSeeded() < 0.2) {
      // 3 triangle prop stack
      new Prop(pos.add(vec2(propPlaceSize * 2, 0)));
      if (randSeeded() < 0.2)
        new Prop(pos.add(vec2(propPlaceSize, propPlaceSize * 2)));
    } else if (randSeeded() < 0.2) {
      // 3 column prop stack
      new Prop(pos.add(vec2(0, propPlaceSize * 2)));
      if (randSeeded() < 0.2) new Prop(pos.add(vec2(0, propPlaceSize * 4)));
    }
  }
}

function buildBase() {
  let raycastHit;
  for (let tries = 99; !raycastHit; ) {
    if (!tries--) return 1; // count not find pos

    const pos = vec2(randSeeded(levelSize.x - 40, 40), levelSize.y);

    // must not be near player start
    if (abs(checkpointPos.x - pos.x) > 30)
      raycastHit = tileCollisionRaycast(pos, vec2(pos.x, 0));
  }

  const cave = rand() < 0.5;
  const baseBottomCenterPos = raycastHit.int();
  const baseSize = randSeeded(20, 9) | 0;
  const baseFloors = cave ? 1 : randSeeded(6, 1) | 0;
  const basementFloors = randSeeded(cave ? 7 : 4, 0) | 0;
  let floorBottomCenterPos = baseBottomCenterPos.subtract(
    vec2(0, basementFloors * 6)
  );
  floorBottomCenterPos.y = max(floorBottomCenterPos.y, 9); // prevent going through bottom

  let floorWidth = baseSize;
  let previousFloorHeight = 0;
  for (let floor = -basementFloors; floor <= baseFloors; ++floor) {
    const topFloor = floor == baseFloors;
    const groundFloor = !floor;
    const isCaveFloor = cave
      ? (rand() < 0.8) | (floor == 0 && rand() < 0.6)
      : 0;
    let floorHeight = isCaveFloor
      ? randSeeded(9, 2) | 0
      : topFloor
      ? 0
      : groundFloor
      ? randSeeded(9, 4) | 0
      : randSeeded(7, 2) | 0;
    const floorSpace = topFloor ? 4 : max(floorHeight - 1, 0);

    let backWindow = rand() < 0.5;
    const windowTop = rand(4, 2);

    for (let x = -floorWidth; x <= floorWidth; ++x) {
      const isWindow = !isCaveFloor && randSeeded() < 0.3;
      const hasSide = !isCaveFloor && randSeeded() < 0.9;

      if (cave) backWindow = 0;
      else if (rand() < 0.1) backWindow = !backWindow;

      if (cave && rand() < 0.2)
        floorHeight = clamp((floorHeight + rand(3, -3)) | 0, 9, 2);

      for (let y = -1; y < floorHeight; ++y) {
        const pos = floorBottomCenterPos.add(vec2(x, y));
        let foregroundTile = tileType_empty;
        if (isCaveFloor) {
          // add ceiling and floor
          if ((y < 0) | (y == floorHeight - 1)) foregroundTile = tileType_dirt;

          setTileBackgroundData(pos, tileType_dirt);
          setTileCollisionData(pos, foregroundTile);
        } else {
          // add ceiling and floor
          const isHorizontal = (y < 0) | (y == floorHeight - 1);
          if (isHorizontal) foregroundTile = tileType_pipeH;

          // add walls and windows
          if (abs(x) == floorWidth)
            foregroundTile = isHorizontal
              ? tileType_base
              : isWindow
              ? tileType_glass
              : tileType_pipeV;

          let backgroundTile =
            foregroundTile > 0 || floorHeight < 3
              ? tileType_baseBack
              : tileType_base;
          if (
            backWindow &&
            y > 0 &&
            y < floorHeight - windowTop &&
            abs(x) < floorWidth - 2
          )
            backgroundTile = tileType_window;

          setTileBackgroundData(pos, backgroundTile);
          setTileCollisionData(pos, foregroundTile);
        }
      }
    }

    // add ladders to floor below
    if (!cave || !topFloor)
      for (let ladderCount = (randSeeded(2) + 1) | 0; ladderCount--; ) {
        const x = randSeeded(floorWidth - 1, -floorWidth + 1) | 0;
        const pos = floorBottomCenterPos.add(vec2(x, -2));

        let y = 0;
        let hitBottom = 0;
        for (; y < levelSize.y; ++y) {
          const pos = floorBottomCenterPos.add(vec2(x, -y - 1));
          if (pos.y < 2) {
            // hit bottom, no ladder
            break;
          }
          if (
            y &&
            getTileCollisionData(pos) > 0 &&
            getTileCollisionData(pos.add(vec2(0, 1))) <= 0
          ) {
            for (; y--; ) {
              const pos = floorBottomCenterPos.add(vec2(x, -y - 1));
              setTileCollisionData(pos, tileType_ladder);
            }
            break;
          }
        }
      }

    // spawn crates
    const propCount = randSeeded(floorWidth / 2) | 0;
    for (let i = propCount; i--; )
      spawnProps(
        floorBottomCenterPos.add(
          vec2(randSeeded(floorWidth - 2, -floorWidth + 2), 0.5)
        )
      );

    if (topFloor || floorSpace > 1) {
      // spawn enemies
      for (let i = propCount; i--; ) {
        const pos = floorBottomCenterPos.add(
          vec2(randSeeded(floorWidth - 1, -floorWidth + 1), 0.7)
        );
        new Enemy(pos);
      }
    }

    const oldFloorWidth = floorWidth;
    floorWidth = max(floorWidth + randSeeded(8, -8), 9) | 0;
    floorBottomCenterPos.y += floorHeight;
    floorBottomCenterPos.x += randSeeded(oldFloorWidth - floorWidth + 1) | 0;
    previousFloorHeight = floorHeight;
  }

  //checkpointPos = floorBottomCenterPos.copy(); // start player on base for testing

  // spawn random enemies and props
  for (let i = 20; levelEnemyCount > 0 && i--; ) {
    const pos = vec2(floorBottomCenterPos.x + randSeeded(99, -99), levelSize.y);
    raycastHit = tileCollisionRaycast(pos, vec2(pos.x, 0));
    // must not be near player start
    if (raycastHit && abs(checkpointPos.x - pos.x) > 20) {
      const pos = raycastHit.add(vec2(0, 2));
      randSeeded() < 0.7 ? new Enemy(pos) : spawnProps(pos);
    }
  }
}

function generateLevel() {
  levelEndTimer.unset();

  // remove all objects that are not persistnt or are descendants of something persitant
  for (const o of engineObjects) o.destroy();
  engineObjects = [];
  engineCollideObjects = [];

  // randomize ground level hills
  buildTerrain(levelSize);

  // find starting poing for player
  let raycastHit;
  for (let tries = 99; !raycastHit; ) {
    if (!tries--) return 1; // count not find pos

    // start on either side of level
    checkpointPos = vec2(
      (levelSize.x / 2 +
        (levelSize.x / 2 - 10 - randSeeded(9)) *
          (randSeeded() < 0.5 ? -1 : 1)) |
        0,
      levelSize.y
    );
    raycastHit = tileCollisionRaycast(checkpointPos, vec2(checkpointPos.x, 0));
  }
  checkpointPos = raycastHit.add(vec2(0, 1));

  // random bases until there enough enemies
  for (let tries = 99; levelEnemyCount > 0; ) {
    if (!tries--) return 1; // count not spawn enemies

    if (buildBase()) return 1;
  }

  // build checkpoints
  for (let x = 0; x < levelSize.x - 9; ) {
    x += rand(100, 70);
    const pos = vec2(x, levelSize.y);
    raycastHit = tileCollisionRaycast(pos, vec2(pos.x, 0));
    // must not be near player start
    if (raycastHit && abs(checkpointPos.x - pos.x) > 50) {
      // todo prevent overhangs
      const pos = raycastHit.add(vec2(0, 1));
      new Checkpoint(pos);
    }
  }
}

const groundTileStart = 8;

function makeTileLayers() {
  // create foreground layer
  tileLayer = new TileLayer(vec2(), levelSize);
  tileLayer.renderOrder = tileRenderOrder;

  // create background layer
  tileBackgroundLayer = new TileLayer(vec2(), levelSize);
  tileBackgroundLayer.renderOrder = tileBackgroundRenderOrder;

  for (let x = levelSize.x; x--; )
    for (let y = levelSize.y; y--; ) {
      const pos = vec2(x, y);
      let tileType = getTileCollisionData(pos);
      if (tileType) {
        // todo pick tile, direction etc based on neighbors tile type
        let direction = rand(4) | 0;
        let mirror = rand(2) | 0;
        let color;

        let tileIndex = groundTileStart;
        if (tileType == tileType_dirt) {
          tileIndex = (groundTileStart + 2 + rand() ** 3 * 2) | 0;
          color = levelColor.mutate(0.03);
        } else if (tileType == tileType_pipeH) {
          tileIndex = groundTileStart + 5;
          direction = 1;
        } else if (tileType == tileType_pipeV) {
          tileIndex = groundTileStart + 5;
          direction = 0;
        } else if (tileType == tileType_glass) {
          tileIndex = groundTileStart + 5;
          direction = 0;
          color = new Color(0, 1, 1, 0.5);
        } else if (tileType == tileType_base) tileIndex = groundTileStart + 4;
        else if (tileType == tileType_ladder) {
          tileIndex = groundTileStart + 7;
          direction = mirror = 0;
        }
        tileLayer.setData(
          pos,
          new TileLayerData(tileIndex, direction, mirror, color)
        );
      }

      tileType = getTileBackgroundData(pos);
      if (tileType) {
        // todo pick tile, direction etc based on neighbors tile type
        const direction = rand(4) | 0;
        const mirror = rand(2) | 0;
        let color = new Color();

        let tileIndex = groundTileStart;
        if (tileType == tileType_dirt) {
          tileIndex = (groundTileStart + 2 + rand() ** 3 * 2) | 0;
          color = levelColor.mutate();
        } else if (tileType == tileType_base) {
          tileIndex = groundTileStart + 6;
          color = color.scale(rand(1, 0.7), 1);
        } else if (tileType == tileType_baseBack) {
          tileIndex = groundTileStart + 6;
          color = color.scale(rand(0.5, 0.3), 1).mutate();
        } else if (tileType == tileType_window) {
          tileIndex = 0;
          color = new Color(0, 1, 1, 0.5);
        }
        tileBackgroundLayer.setData(
          pos,
          new TileLayerData(tileIndex, direction, mirror, color.scale(0.4, 1))
        );
      }
    }
  tileLayer.redraw();
  tileBackgroundLayer.redraw();
}

function applyArtToLevel() {
  makeTileLayers();

  // apply decoration to level tiles
  for (let x = levelSize.x; x--; )
    for (let y = levelSize.y; --y; ) {
      decorateBackgroundTile(vec2(x, y));
      decorateTile(vec2(x, y));
    }

  generateParallaxLayers();
}

function nextLevel() {
  playerLives += 4; // three for beating a level plus 1 for respawning
  levelEnemyCount = 15 + min(level * 30, 300);
  ++level;
  levelSeed = randSeed = rand(1e9) | 0;
  levelSize = vec2(min(level * 99, 400), 200);
  levelColor = randColor(new Color(0.2, 0.2, 0.2), new Color(0.8, 0.8, 0.8));
  levelSkyColor = randColor(new Color(0.5, 0.5, 0.5), new Color(0.9, 0.9, 0.9));
  levelSkyHorizonColor = levelSkyColor
    .subtract(new Color(0.05, 0.05, 0.05))
    .mutate(0.3)
    .clamp();
  levelGroundColor = levelColor.mutate().add(new Color(0.3, 0.3, 0.3)).clamp();

  // keep trying until a valid level is generated
  for (; generateLevel(); );

  // warm up level
  levelWarmup = 1;

  // objects that effect the level must be added here
  const firstCheckpoint = new Checkpoint(checkpointPos).setActive();

  applyArtToLevel();

  const warmUpTime = 2;
  for (let i = warmUpTime * FPS; i--; ) {
    engineUpdateObjects();
  }
  levelWarmup = 0;

  // destroy any objects that are stuck in collision
  forEachObject(0, 0, (o) => {
    if (o.isGameObject && o != firstCheckpoint) {
      const checkBackground = o.isCheckpoint;
      (checkBackground
        ? getTileBackgroundData(o.pos) > 0
        : tileCollisionTest(o.pos, o.size)) && o.destroy();
    }
  });

  // hack, subtract off warm up time from main game timer
  //gameTimer.time += warmUpTime;
  levelTimer.set();

  // spawn player
  players = [];
  new Player(checkpointPos);
  //new Enemy(checkpointPos.add(vec2(3))); // test enemy
}

const maxPlayers = 4;

const team_none = 0;
const team_player = 1;
const team_enemy = 2;

let updateWindowSize, renderWindowSize, gameplayWindowSize;

engineInit(
  () /* appInit */ => {
    resetGame();
    cameraScale = CAMERA_SCALE;
  },
  () /* appUpdate */ => {
    const cameraSize = vec2(mainCanvas.width, mainCanvas.height).scale(
      1 / cameraScale
    );
    renderWindowSize = cameraSize.add(vec2(5));

    gameplayWindowSize = vec2(mainCanvas.width, mainCanvas.height).scale(
      1 / CAMERA_SCALE
    );
    updateWindowSize = gameplayWindowSize.add(vec2(30));

    // restart if no lives left
    let minDeadTime = 1e3;
    for (const player of players)
      minDeadTime = min(
        minDeadTime,
        player && player.isDead() ? player.deadTimer.get() : 0
      );

    if (
      (minDeadTime > 3 &&
        (keyWasPressed(90) || keyWasPressed(32) || gamepadWasPressed(0))) ||
      keyWasPressed(82)
    )
      resetGame();

    if (levelEndTimer.get() > 3) nextLevel();
  },
  () /* appUpdatePost */ => {
    if (players.length == 1) {
      const player = players[0];
      if (!player.isDead())
        cameraPos = cameraPos.lerp(
          player.pos,
          clamp(player.getAliveTime() / 2)
        );
    } else {
      // camera follows average pos of living players
      let posTotal = vec2();
      let playerCount = 0;
      let cameraOffset = 1;
      for (const player of players) {
        if (player && !player.isDead()) {
          ++playerCount;
          posTotal = posTotal.add(player.pos.add(vec2(0, cameraOffset)));
        }
      }

      if (playerCount)
        cameraPos = cameraPos.lerp(posTotal.scale(1 / playerCount), 0.2);
    }

    // spawn players if they don't exist
    for (let i = maxPlayers; i--; ) {
      if (!players[i] && (gamepadWasPressed(0, i) || gamepadWasPressed(1, i))) {
        ++playerLives;
        new Player(checkpointPos, i);
      }
    }

    updateParallaxLayers();
  },
  () /* appRender */ => {
    const gradient = mainContext.createLinearGradient(
      0,
      0,
      0,
      mainCanvas.height
    );
    gradient.addColorStop(0, levelSkyColor.rgba());
    gradient.addColorStop(1, levelSkyHorizonColor.rgba());
    mainContext.fillStyle = gradient;
    //mainContext.fillStyle = levelSkyColor.rgba();
    mainContext.fillRect(0, 0, mainCanvas.width, mainCanvas.height);

    drawStars();
  },
  () /* appRenderPost */ => {
    mainContext.textAlign = "center";
    const p = percent(gameTimer.get(), 8, 10);
    mainContext.fillStyle = new Color(0, 0, 0, p).rgba();
    if (p > 0) {
      mainContext.font = mainCanvas.width / 12 + "px monospace";
      mainContext.fillText("SPACE HUGGERS", mainCanvas.width / 2, 140);
    }

    mainContext.font = mainCanvas.width / 30 + "px monospace";
    p > 0 &&
      mainContext.fillText(
        "A JS13K Game by Frank Force",
        mainCanvas.width / 2,
        210
      );

    // check if any enemies left
    let enemiesCount = 0;
    for (const o of engineCollideObjects) {
      if (o.isCharacter && o.team == team_enemy) {
        ++enemiesCount;
        const pos = vec2(
          mainCanvas.width / 2 + (o.pos.x - cameraPos.x) * 30,
          mainCanvas.height - 20
        );
        drawTileScreenSpace(
          pos,
          o.size.scale(20),
          -1,
          defaultTileSize,
          o.color.scale(1, 0.6)
        );
      }
    }

    if (!enemiesCount && !levelEndTimer.isSet()) levelEndTimer.set();

    mainContext.fillStyle = new Color(0, 0, 0).rgba();
    mainContext.fillText(
      "Level " +
        level +
        "    Lives " +
        playerLives +
        "    Enemies " +
        enemiesCount,
      mainCanvas.width / 2,
      mainCanvas.height - 40
    );

    // fade in level transition
    const fade = levelEndTimer.isSet()
      ? percent(levelEndTimer.get(), 3, 1)
      : percent(levelTimer.get(), 0.5, 2);
    drawRect(cameraPos, vec2(1e3), new Color(0, 0, 0, fade));
  }
);
