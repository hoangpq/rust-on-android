function serveRaw(callback) {
  app.get('/', function(req, res) {
    $toast(` \nUser-Agent: ${req.headers['user-agent']}\n`);
    res.json({ ...process.versions });
  });

  app.get('/stream', onRequest);
  app.listen(PORT, callback);
}

module.exports = { serve: serveRaw };
