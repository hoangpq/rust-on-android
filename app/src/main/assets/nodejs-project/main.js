var http = require('http');
var { createObject } = process.binding('java');

console.log(typeof createObject);

var obj = createObject(10);
$log(obj.plusOne());
// Prints: 11
$log(obj.plusOne());
// Prints: 12
$log(obj.plusOne());
// Prints: 13

var obj2 = createObject(20);
$log(obj2.plusOne());
// Prints: 21
$log(obj2.plusOne());
// Prints: 22
$log(obj2.plusOne());
// Prints: 23

var server = http.createServer( (request, response) => {
  var msg = ` \nUser-Agent: ${request.headers['user-agent']}\n`;
  $toast(msg);
  $log(msg);
  response.end(JSON.stringify({...process.versions }));
});

server.listen(3000);
