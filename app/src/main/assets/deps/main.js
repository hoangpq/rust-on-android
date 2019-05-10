const fs = require('fs');
const path = require('path');
const http = require('http');
const { TextDecoder } = require('util');
const { Transform } = require('stream');

const server = require('./server');

const INTERVAL = 3000;
function ToastQueue() {
  this._queue = [];
  this._timer = null;
  this._initialize = false;
  process.on('exit', code => {
    this._timer && clearTimeout(this._timer);
    $log(`Toast queue stopped`);
  });
}

ToastQueue.prototype.push = function(func, value) {
  this._queue.push(func.bind(null, value || ''));
};

ToastQueue.prototype.start = function() {
  this._timer = setTimeout(
    () => {
      const func = this._queue.shift();
      if (func && typeof func === 'function') {
        $log(func.call(null));
        if (this._queue.length) {
          if (!this._initialize) {
            this._initialize = true;
          }
          this.start();
        } else {
          this._timer && clearTimeout(this._timer);
        }
      }
    },
    this._initialize ? INTERVAL : 0
  );
};

// wasm test
const env = {};
const waBuf = new Uint8Array(fs.readFileSync(path.join(__dirname, 'wa.wasm')));
// $log((new TextDecoder).decode(Uint8Array.from([0xe3, 0x81, 0x82])));

// run wa
(async function() {
  try {
    const result = await WebAssembly.instantiate(waBuf, env);
    const len = 3;
    const buf = new Uint8ClampedArray(
      result.instance.exports.memory.buffer,
      new Uint8Array(len),
      len
    );
    result.instance.exports.modify(buf);
    const word = new TextDecoder().decode(buf);
    $log(`WASM result: ${word}`);
  } catch (e) {
    $error(e.message);
  }
})();

try {
  server.serve(() => {
    if (typeof $load === 'function') $load();
    $log('Server is running...');
  });
} catch (e) {
  $log(e.message);
}
