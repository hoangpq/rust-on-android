const fs = require('fs');
const path = require('path');
const http = require('http');
const { TextDecoder } = require('util');
const { Transform } = require('stream');

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

// const $toastQueue = new ToastQueue();
// $toastQueue.start();

const $list = Java.type('java/util/ArrayList');

for (let i = 0; i < 10; i++) {
  $list.add(Math.random() * 1000);
}

$log(`List: ${$list}`);

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

    res.writeHead(206, {
      'Content-Range': `bytes ${start}-${end}/${fileSize}`,
      'Accept-Ranges': 'bytes',
      'Content-Length': chunksize,
      'Content-Type': 'video/mp4'
    });
    file.pipe(filterStream).pipe(res);
  } else {
    res.writeHead(200, {
      'Content-Length': fileSize,
      'Content-Type': 'video/mp4'
    });
    fs.createReadStream(assetPath)
      .pipe(filterStream)
      .pipe(res);
  }
}

function handleNodeVersionRequest(req, res) {
  res.statusCode = 200;
  res.writeHead(200, {
    'Content-Type': 'application/json'
  });
  $toast(` \nUser-Agent: ${req.headers['user-agent']}\n`);
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
