import React from 'react';

// Home route - will be code-split as a separate chunk
// This is discovered automatically by scanning app/routes/

export default function IndexRoute() {
  return (
    <div>
      <h1>Welcome to My Framework</h1>
      <p>This is the home page built with a custom meta-framework.</p>
      <p>Each route is bundled as a separate entry point with code splitting.</p>
    </div>
  );
}
