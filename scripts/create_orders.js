import crypto from 'node:crypto';
import { post } from './client.js';

async function main() {
  while (true) {
    const side = randomSide();
    const price = crypto.randomInt(40, 400) / 4;
    const quantity = crypto.randomInt(200, 600);
    const order = await post('/orders', { side, price, quantity });
  }
}

function randomSide() {
  const value = Boolean(crypto.randomInt(0, 2));
  return value ? 'Buy' : 'Sell';
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
}).finally(() => {
  client.close();
})
