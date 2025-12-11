/**
 * Hello World Template
 *
 * The simplest possible example.
 */

export function greet(name: string): string {
  return `Hello, ${name}!`;
}

export function App() {
  return (
    <div>
      <h1>{greet('World')}</h1>
      <p>Welcome to Fob Playground!</p>
    </div>
  );
}

console.log(greet('Console'));

export default App;
