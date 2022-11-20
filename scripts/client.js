const http2 = require('node:http2');
const crypto = require('node:crypto');

const client = http2.connect('http://localhost:3000');
client.on('error', (err) => console.error(err));

async function main() {
  const orderBook = await get('/');
  console.dir(orderBook);

  while (true) {
    const side = randomSide();
    const price = crypto.randomInt(1800, 2400) / 100;
    const quantity = crypto.randomInt(200, 600);
    const order = await post('/orders', { side, price, quantity });
    console.dir(order);
  }
}

function randomSide() {
  const value = Boolean(crypto.randomInt(0, 2));
  return value ? 'Buy' : 'Sell';
}

function get(path) {
  return new Promise((resolve) => {
    const req = client.request({ ':path': path });

    req.setEncoding('utf8');
    let data = '';
    req.on('data', (chunk) => { data += chunk; });
    req.on('end', () => {
      resolve(JSON.parse(data))
    });
    req.end();
  });
}

function post(path, payload) {
  return new Promise((resolve) => {
    const req = client.request({ ':method': 'POST', ':path': path });

    req.setEncoding('utf8');
    let data = '';
    req.on('data', (chunk) => { data += chunk; });
    req.on('end', () => {
      resolve(JSON.parse(data))
    });
    req.write(JSON.stringify(payload));
    req.end();
  });
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
}).finally(() => {
  client.close();
})
