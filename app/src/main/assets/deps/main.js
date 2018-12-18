const fs = require('fs');
const path = require('path');
const http = require('http');
const { TextDecoder } = require('util');
// const Java = process.binding('java');
const { Transform } = require('stream');

const INTERVAL = 3000;
function ToastQueue() {
  this._queue = [];
  this._timer = null;
  this._initialize = false;
  process.on('exit', (code) => {
    this._timer && clearTimeout(this._timer);
    $log(`Toast queue stopped`);
  });
}

ToastQueue.prototype.push = function(func, value) {
  this._queue.push(func.bind(null, value || ''));
};

ToastQueue.prototype.start = function() {
  this._timer = setTimeout(() => {
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
  }, this._initialize ? INTERVAL : 0);
};

const $toastQueue = new ToastQueue();
$toastQueue.start();

if (typeof Java !== 'undefined') {
  const $type = Java.type('java/util/ArrayList');
  // list all key

  console.log(typeof $type.add);

  // $type.add(10);
  $toast(`JNI Version: ${$type.$jni_version}`);
  // $toast(`Android Version API: ${$type.$version()}`);
  // JNI Version
  // $toastQueue.push($toast, `JNI Version: ${$type.jniVersion}`);
  // Android version
  // $toastQueue.push($toast, `Android Version API: ${$type.$version()}`);
}

// wasm test
const env = {};
const waBuf = new Uint8Array(fs.readFileSync(path.join(__dirname, 'wa.wasm')));
// $log((new TextDecoder).decode(Uint8Array.from([0xe3, 0x81, 0x82])));

WebAssembly.instantiate(waBuf, env)
  .then(result => {
    const len = 3;
    var buf = new Uint8ClampedArray(
      result.instance.exports.memory.buffer,
      new Uint8Array(len),
      len
    );
    result.instance.exports.modify(buf);
    const word = new TextDecoder().decode(buf);
    // $toastQueue.push($toast, `WASM result: ${word}`);
  })
  .catch(e => {
    // error caught
    $error(e.message);
  });

function serveFile(req, res) {
  res.setHeader('X-Content-Type-Options', 'nosniff');
  const assetPath = path.join(__dirname, 'sample.mp4');
  const stat = fs.statSync(assetPath);
  const fileSize = stat.size;
  const range = req.headers.range;

  const filterStream = new Transform({
    transform(chunk, encoding, callback) {
      this.push(chunk);
      callback();
    }
  });

  if (range) {
    const parts = range.replace(/bytes=/, '').split('-');
    const start = parseInt(parts[0], 10);
    const end = parts[1] ? parseInt(parts[1], 10) : fileSize - 1;

    const chunksize = end - start + 1;
    const file = fs.createReadStream(assetPath, { start, end });
    const head = {
      'Content-Range': `bytes ${start}-${end}/${fileSize}`,
      'Accept-Ranges': 'bytes',
      'Content-Length': chunksize,
      'Content-Type': 'video/mp4'
    };

    res.writeHead(206, head);
    file.pipe(filterStream).pipe(res);
  } else {
    const head = {
      'Content-Length': fileSize,
      'Content-Type': 'video/mp4'
    };
    res.writeHead(200, head);
    fs.createReadStream(assetPath)
      .pipe(filterStream)
      .pipe(res);
  }
}

function handleNodeVersionRequest(req, res) {
  res.statusCode = 200;
  res.setHeader('Content-Type', 'application/json');
  const msg = ` \nUser-Agent: ${req.headers['user-agent']}\n`;
  $toast(msg);
  $log(msg);
  res.end(JSON.stringify({ ...process.versions }));
}

const server = http.createServer((req, res) => {
  switch (req.url) {
    case '/':
      return handleNodeVersionRequest(req, res);
    case '/stream':
      return serveFile(req, res);
  }
});

server.listen(3000, () => $log(`Server is running on port 3000`));
