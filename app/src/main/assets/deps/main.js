const fs = require('fs');
const path = require('path');
const http = require('http');
const { TextDecoder } = require('util');
const Java = process.binding('java');
const { Transform } = require('stream');

if (typeof Java !== 'undefined') {
  const $type = Java.type();
  if (typeof $type.$toast === 'function') {
    $type.$toast();
  }
  if (typeof $type.jni_version !== 'undefined') {
    $log(`JNI Version: ${$type.jni_version}`);
  }
  // Android version
  if (typeof $type.androidVersion === 'function') {
    $log(`Android Version API: ${$type.androidVersion()}`);
  }
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
    // console.log(`WASM result: ${word}`);
    const $t = setTimeout(() => {
      $toast(`WASM result: ${word}`);
      clearTimeout($t);
    }, 3000);
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
