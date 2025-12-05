// Basic JSX test fixture - React component without TypeScript
import React from 'react';

export function Greeting({ name }) {
  return (
    <div className="greeting">
      <h1>Hello, {name}!</h1>
      <p>Welcome to the JSX test.</p>
    </div>
  );
}

export default Greeting;
