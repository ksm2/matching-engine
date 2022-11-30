import { get } from './client.js';

async function main() {
  while (true) {
    await get('/');
  }
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
}).finally(() => {
  client.close();
})
