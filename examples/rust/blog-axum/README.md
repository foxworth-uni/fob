# Bunny Blog with Axum

A complete example of building a blog with Rust, Axum, and Bunny MDX compiler.

## Features

- 🚀 **Fast MDX Compilation** - Uses Bunny's high-performance Rust compiler
- 🎨 **Beautiful UI** - Clean, responsive design
- 📝 **Frontmatter Support** - YAML metadata for blog posts
- 🔄 **Hot Reload** - Reload posts without restarting the server
- 🏷️ **Tags & Categories** - Organize your content
- 📅 **Date Sorting** - Posts sorted by publication date
- 🎯 **Type-Safe** - Leverages Rust's type system

## Getting Started

### Prerequisites

- Rust 1.77 or later
- Cargo

### Installation

1. Navigate to the example directory:

```bash
cd examples/blog-axum
```

2. Build the project:

```bash
cargo build
```

3. Run the server:

```bash
cargo run
```

4. Open your browser and visit:

```
http://localhost:3000
```

## Project Structure

```
blog-axum/
├── Cargo.toml           # Dependencies and configuration
├── src/
│   ├── main.rs          # Axum server and routes
│   └── mdx.rs           # MDX compiler wrapper
├── templates/           # Askama HTML templates
│   ├── index.html       # Home page listing posts
│   ├── post.html        # Individual post page
│   └── 404.html         # Not found page
├── posts/               # Your MDX blog posts
│   ├── welcome.mdx
│   ├── rust-web-frameworks.mdx
│   └── mdx-guide.mdx
├── static/              # Static assets
│   └── style.css        # Styles
└── README.md
```

## Creating Blog Posts

Create a new `.mdx` file in the `posts/` directory with frontmatter:

````mdx
---
title: My Amazing Post
description: A short description of my post
author: Your Name
date: 2025-11-24T00:00:00Z
tags:
  - rust
  - tutorial
---

# My Amazing Post

Your content here! You can use all Markdown features:

- Lists
- **Bold** and _italic_ text
- Code blocks
- And more!

## Code Example

```rust
fn main() {
    println!("Hello, World!");
}
```
````

````

### Frontmatter Fields

- **title** (required) - Post title
- **description** (optional) - Short description for previews
- **author** (optional) - Author name
- **date** (optional) - Publication date in ISO 8601 format
- **tags** (optional) - Array of tags

## API Endpoints

### `GET /`

Lists all blog posts sorted by date (newest first).

### `GET /post/:slug`

Displays a single blog post. The slug is the filename without the `.mdx` extension.

Example: `/post/welcome` loads `posts/welcome.mdx`

### `GET /api/reload`

Reloads all blog posts from disk without restarting the server. Useful during development.

```bash
curl http://localhost:3000/api/reload
````

## Development Workflow

1. **Create a new post**: Add a `.mdx` file to `posts/`
2. **Reload posts**: Visit `/api/reload` or restart the server
3. **View your post**: Navigate to `/post/your-slug`

## Architecture

### MDX Compilation Pipeline

```
MDX File → Bunny Compiler → JSX Code → Display
   ↓
Frontmatter → YAML Parser → Metadata
```

### Components

#### `AppState`

Manages application state with:

- `posts` - In-memory cache of compiled posts
- `compiler` - Bunny MDX compiler instance

#### `BlogPost`

Struct representing a blog post:

- `slug` - URL-friendly identifier
- `title`, `description`, `author` - Metadata
- `date` - Publication date
- `tags` - Categorization
- `content` - Original MDX source
- `html` - Compiled JSX output

#### `MdxCompiler`

Wrapper around Bunny's MDX compiler with:

- Frontmatter extraction
- JSX compilation
- Error handling

## Customization

### Styling

Edit `static/style.css` to customize the appearance. The design uses CSS variables for easy theming:

```css
:root {
  --primary: #ff6b6b;
  --bg: #ffffff;
  --text: #2d3436;
  /* ... more variables */
}
```

### Templates

Modify the Askama templates in `templates/`:

- `index.html` - Home page layout
- `post.html` - Post page layout
- `404.html` - Error page

### MDX Options

Configure the compiler in `src/mdx.rs`:

```rust
Options {
    development: false,
    jsx_runtime: JsxRuntime::Automatic,
    jsx_import_source: Some("react".to_string()),
    provider_import_source: None,
    plugins: PluginOptions::default(),
}
```

## Production Considerations

This example demonstrates MDX compilation but doesn't include a full rendering pipeline. For production, you might want to:

1. **Add a React/Preact Renderer**
   - Use a JavaScript runtime to convert JSX to HTML
   - Or implement server-side rendering

2. **Integrate with a Bundler**
   - Use Rolldown or similar to bundle dependencies
   - Handle JavaScript modules

3. **Add Caching**
   - Cache compiled posts in memory or Redis
   - Implement cache invalidation

4. **Implement Search**
   - Add full-text search with Tantivy or similar
   - Build search indexes

5. **Add RSS Feed**
   - Generate RSS/Atom feeds
   - Implement feed pagination

6. **Performance**
   - Enable compression (gzip/brotli)
   - Add CDN for static assets
   - Implement HTTP caching headers

7. **Security**
   - Add rate limiting
   - Sanitize user input
   - Implement CORS properly

## Dependencies

- **axum** - Web framework
- **tokio** - Async runtime
- **tower** - Middleware
- **tower-http** - HTTP middleware (static files)
- **bunny** - MDX bundler
- **bunny-mdx** - MDX compiler
- **askama** - Template engine
- **serde** - Serialization
- **chrono** - Date/time handling
- **tracing** - Logging

## License

This example is part of the Bunny project and is licensed under the MIT License.

## Learn More

- [Bunny Documentation](https://bunny.dev)
- [Axum Documentation](https://docs.rs/axum)
- [MDX Specification](https://mdxjs.com)
- [Rust Book](https://doc.rust-lang.org/book/)

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

---

Built with ❤️ using Rust, Axum, and Bunny
