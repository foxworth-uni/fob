/**
 * Heavy module that will be code-split via dynamic import
 * This demonstrates how Fob handles code splitting
 */

export function processData(data) {
  console.log('⚡ Heavy module loaded!');
  
  const sum = data.reduce((acc, val) => acc + val, 0);
  const avg = sum / data.length;
  const max = Math.max(...data);
  const min = Math.min(...data);
  
  return {
    sum,
    average: avg,
    max,
    min,
    count: data.length,
  };
}

export function complexCalculation(iterations = 1000) {
  let result = 0;
  for (let i = 0; i < iterations; i++) {
    result += Math.sqrt(i) * Math.sin(i);
  }
  return result;
}

