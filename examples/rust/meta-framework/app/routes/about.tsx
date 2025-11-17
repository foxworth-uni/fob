import React from 'react';

// About route - will be code-split as a separate chunk
// Shared code (like React) will be extracted into common chunks

export default function AboutRoute() {
  return (
    <div>
      <h1>About</h1>
      <p>This demonstrates code splitting across routes.</p>
      <p>React is imported in both routes but will be extracted into a shared chunk.</p>
    </div>
  );
}
