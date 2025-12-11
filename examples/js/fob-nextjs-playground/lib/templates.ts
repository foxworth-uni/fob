export const templates = {
  'react-app': `/**
 * React App Template
 */
import React, { useState, useEffect } from 'react';

interface User {
  id: number;
  name: string;
  email: string;
}

function UserCard({ user }: { user: User }) {
  return (
    <div className="user-card">
      <h3>{user.name}</h3>
      <p>{user.email}</p>
    </div>
  );
}

export function App() {
  const [users, setUsers] = useState<User[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    setTimeout(() => {
      setUsers([
        { id: 1, name: 'Alice', email: 'alice@example.com' },
        { id: 2, name: 'Bob', email: 'bob@example.com' },
      ]);
      setLoading(false);
    }, 500);
  }, []);

  if (loading) return <div>Loading...</div>;

  return (
    <div className="app">
      <h1>User Directory</h1>
      {users.map((user) => (
        <UserCard key={user.id} user={user} />
      ))}
    </div>
  );
}

export default App;`,

  counter: `/**
 * Counter Template
 */
import React, { useState } from 'react';

export function Counter() {
  const [count, setCount] = useState(0);

  return (
    <div className="counter">
      <h1>Count: {count}</h1>
      <button onClick={() => setCount(count - 1)}>-</button>
      <button onClick={() => setCount(0)}>Reset</button>
      <button onClick={() => setCount(count + 1)}>+</button>
    </div>
  );
}

export default Counter;`,

  hello: `/**
 * Hello World Template
 */
export function greet(name: string): string {
  return \`Hello, \${name}!\`;
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

export default App;`,
};

export type TemplateName = keyof typeof templates;
