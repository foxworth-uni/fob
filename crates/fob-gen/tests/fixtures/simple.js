// Simple JavaScript file for round-trip testing
const x = 42;
let y = "hello";
var z = true;

function greet(name) {
    return `Hello, ${name}!`;
}

const double = (value) => value * 2;

const greeting = greet(y);
const doubled = double(x);

if (x > 0) {
    console.log("positive", greeting, doubled, z);
} else {
    console.log("negative", greeting, doubled, z);
}

