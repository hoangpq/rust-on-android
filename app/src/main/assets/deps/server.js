const express = require('express');

const app = express();
const PORT = process.env.PORT || 3000;

function serveRaw(callback) {
  app.get('/', function(req, res) {
    $toast(` \nUser-Agent: ${req.headers['user-agent']}\n`);
    res.json({ ...process.versions });
  });

  app.get('/stream', function(req, res) {});
  app.listen(PORT, callback);
}

module.exports = { serve: serveRaw };
