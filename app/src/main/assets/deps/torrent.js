const express = require('express');
const WebTorrent = require('webtorrent');
const client = new WebTorrent();

const mime = require('mime');
const pump = require('pump');
const path = require('path');
const rangeParser = require('range-parser');

const magnetURI = 'https://webtorrent.io/torrents/sintel.torrent';

const app = express();
const PORT = process.env.PORT || 3000;

function fetchTorrent() {
  return new Promise((resolve, reject) => {
    setTimeout(function() {
      reject();
    }, 1e4);
    client.add(magnetURI, { path: path.join(__dirname, 'torrent') }, function(
      torrent
    ) {
      const file = torrent.files.find(function(file) {
        return file.name.endsWith('.mp4');
      });
      resolve(file);
    });
  });
}

// From https://developer.mozilla.org/en/docs/Web/JavaScript/Reference/Global_Objects/encodeURIComponent
function encodeRFC5987(str) {
  return (
    encodeURIComponent(str)
      // Note that although RFC3986 reserves "!", RFC5987 does not,
      // so we do not need to escape it
      .replace(/['()]/g, escape) // i.e., %27 %28 %29
      .replace(/\*/g, '%2A')
      // The following are not required for percent-encoding per RFC5987,
      // so we can allow for a little better readability over the wire: |`^
      .replace(/%(?:7C|60|5E)/g, unescape)
  );
}

function serve(callback) {
  fetchTorrent()
    .then(function(file) {
      function onRequest(req, res) {
        // Prevent browser mime-type sniffing
        res.setHeader('X-Content-Type-Options', 'nosniff');

        serveFile(file);

        function serveFile(file) {
          res.statusCode = 200;
          res.setHeader('Content-Type', mime.getType(file.name));

          // Support range-requests
          res.setHeader('Accept-Ranges', 'bytes');

          // Set name of file (for "Save Page As..." dialog)
          res.setHeader(
            'Content-Disposition',
            `inline; filename*=UTF-8''${encodeRFC5987(file.name)}`
          );

          // Support DLNA streaming
          res.setHeader('transferMode.dlna.org', 'Streaming');
          res.setHeader(
            'contentFeatures.dlna.org',
            'DLNA.ORG_OP=01;DLNA.ORG_CI=0;DLNA.ORG_FLAGS=01700000000000000000000000000000'
          );

          // `rangeParser` returns an array of ranges, or an error code (number) if
          // there was an error parsing the range.
          let range = rangeParser(file.length, req.headers.range || '');

          if (Array.isArray(range)) {
            res.statusCode = 206; // indicates that range-request was understood

            // no support for multi-range request, just use the first range
            range = range[0];

            res.setHeader(
              'Content-Range',
              `bytes ${range.start}-${range.end}/${file.length}`
            );
            res.setHeader('Content-Length', range.end - range.start + 1);
          } else {
            range = null;
            res.setHeader('Content-Length', file.length);
          }

          if (req.method === 'HEAD') {
            return res.end();
          }

          pump(file.createReadStream(range), res);
        }
      }

      app.get('/', function(req, res) {
        $toast(` \nUser-Agent: ${req.headers['user-agent']}\n`);
        res.json({ ...process.versions });
      });

      app.get('/stream', onRequest);
      app.listen(PORT, callback);
    })
    .catch(function() {
      throw new Error(`Can't not streaming video`);
    });
}
