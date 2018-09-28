var http = require('http');

var server = http.createServer( (request, response) => {
  $toast(`\nUser-Agent: ${request.headers['user-agent']}\n`);
  response.end(JSON.stringify({...process.versions }));
});

server.listen(3000);
