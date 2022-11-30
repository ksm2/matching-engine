import http2 from 'node:http2';

const client = http2.connect('http://localhost:3000');
client.on('error', (err) => console.error(err));

export function get(path) {
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

export function post(path, payload) {
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
