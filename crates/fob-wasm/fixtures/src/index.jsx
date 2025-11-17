// Test JavaScript entry point
import BlogPost from './blog-post.mdx';
import { Button } from './component.jsx';

console.log('Fob bundler test');

export function App() {
  return (
    <div>
      <h1>Welcome to Joy!</h1>
      <BlogPost />
      <Button text="Click me" />
    </div>
  );
}

export default App;
