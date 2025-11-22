// Worker entry point
// Shares code with app.js - will be extracted into common chunk

import { add, multiply } from '@lib/math';
import { formatNumber } from '@lib/format';
import { APP_CONFIG } from './config.js';

console.log(`⚙️  Worker: ${APP_CONFIG.name}`);
console.log('');

// Worker-specific computation
function processData(data) {
  const sum = data.reduce((acc, val) => add(acc, val), 0);
  const product = data.reduce((acc, val) => multiply(acc, val), 1);

  return {
    sum: formatNumber(sum),
    product: formatNumber(product),
    count: data.length,
  };
}

const testData = [1, 2, 3, 4, 5];
const result = processData(testData);

console.log('📊 Worker Results:');
console.log(`   Data: [${testData.join(', ')}]`);
console.log(`   Sum: ${result.sum}`);
console.log(`   Product: ${result.product}`);
console.log(`   Count: ${result.count}`);

export { processData };
