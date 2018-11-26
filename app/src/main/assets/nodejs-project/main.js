const http = require('http');
const { TextDecoder } = require('util');
const { createObject } = process.binding('java');

// wasm test
const fs = require('fs');
const path = require('path');

const env = {};
const waBuf = new Uint8Array(fs.readFileSync(path.join(__dirname, 'wa.wasm')));
// $log((new TextDecoder).decode(Uint8Array.from([0xe3, 0x81, 0x82])));

WebAssembly.instantiate(waBuf, env)
    .then(result => {
        const len = 3;
        var buf = new Uint8ClampedArray(result.instance.exports.memory.buffer, new Uint8Array(len), len);
        result.instance.exports.modify(buf);
        const word = new TextDecoder().decode(buf);
        // $log(`WASM result: ${word}`);
        $toast(`WASM result: ${word}`);
    }).catch(e => {
        // error caught
        $log(e.message);
    });

// console.log(typeof createObject);

var obj = createObject(10);
// $log(obj.plusOne());
// $log(obj.plusOne());
// $log(obj.plusOne());

var server = http.createServer( (request, response) => {
  var msg = ` \nUser-Agent: ${request.headers['user-agent']}\n`;
  $toast(msg);
  $log(msg);
  response.end(JSON.stringify({...process.versions }));
});

server.listen(3000);
