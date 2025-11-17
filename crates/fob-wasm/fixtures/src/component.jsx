// Test JSX component
import { useState } from 'react';

export function Button({ text }) {
  const [count, setCount] = useState(0);

  return (
    <button onClick={() => setCount(count + 1)}>
      {text}: {count}
    </button>
  );
}

export function Card({ title, children }) {
  return (
    <div className="card">
      <h2>{title}</h2>
      <div className="card-body">{children}</div>
    </div>
  );
}

export default Button;
