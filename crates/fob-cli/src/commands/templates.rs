//! Project templates for the init command.

/// Template types available for project initialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Template {
    Library,
    App,
    ComponentLibrary,
    MetaFramework,
}

impl Template {
    /// Parse template name from string.
    ///
    /// Accepts multiple aliases for each template type to improve UX:
    /// - Library: "library", "lib"
    /// - App: "app", "application"
    /// - ComponentLibrary: "component-library", "components"
    /// - MetaFramework: "meta-framework", "framework"
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "library" | "lib" => Some(Template::Library),
            "app" | "application" => Some(Template::App),
            "component-library" | "components" => Some(Template::ComponentLibrary),
            "meta-framework" | "framework" => Some(Template::MetaFramework),
            _ => None,
        }
    }

    /// Get template name.
    pub fn name(&self) -> &'static str {
        match self {
            Template::Library => "library",
            Template::App => "app",
            Template::ComponentLibrary => "component-library",
            Template::MetaFramework => "meta-framework",
        }
    }
}

/// Generate package.json content for a template.
///
/// Creates a package.json tailored to each template type with appropriate
/// dependencies, scripts, and configuration. All generated packages use ESM
/// format (type: "module") for modern JavaScript development.
pub fn package_json(name: &str, template: Template) -> String {
    match template {
        Template::Library => format!(
            r#"{{
  "name": "{}",
  "version": "0.1.0",
  "type": "module",
  "main": "./dist/index.js",
  "module": "./dist/index.js",
  "types": "./dist/index.d.ts",
  "exports": {{
    ".": {{
      "types": "./dist/index.d.ts",
      "import": "./dist/index.js",
      "require": "./dist/index.cjs"
    }}
  }},
  "files": [
    "dist"
  ],
  "scripts": {{
    "build": "fob build src/index.ts --dts",
    "dev": "fob dev",
    "check": "fob check"
  }},
  "devDependencies": {{
    "@fob/bundler": "^0.1.0",
    "typescript": "^5.0.0"
  }}
}}
"#,
            name
        ),
        Template::App => format!(
            r#"{{
  "name": "{}",
  "version": "0.1.0",
  "type": "module",
  "scripts": {{
    "dev": "fob dev",
    "build": "fob build src/main.ts --minify",
    "check": "fob check"
  }},
  "devDependencies": {{
    "@fob/browser": "^0.1.0",
    "typescript": "^5.0.0"
  }}
}}
"#,
            name
        ),
        Template::ComponentLibrary => format!(
            r#"{{
  "name": "{}",
  "version": "0.1.0",
  "type": "module",
  "main": "./dist/index.js",
  "types": "./dist/index.d.ts",
  "exports": {{
    ".": "./dist/index.js",
    "./Button": "./dist/Button.js"
  }},
  "scripts": {{
    "build": "fob build",
    "dev": "fob dev"
  }},
  "peerDependencies": {{
    "react": "^18.0.0"
  }},
  "devDependencies": {{
    "@types/react": "^18.0.0",
    "react": "^18.0.0",
    "typescript": "^5.0.0"
  }}
}}
"#,
            name
        ),
        Template::MetaFramework => format!(
            r#"{{
  "name": "{}",
  "version": "0.1.0",
  "type": "module",
  "bin": {{
    "{}": "./dist/index.js"
  }},
  "exports": {{
    ".": "./dist/index.js",
    "./runtime": "./dist/runtime.js"
  }},
  "scripts": {{
    "build": "fob build",
    "dev": "fob dev"
  }},
  "dependencies": {{
    "@fob/bundler": "^0.1.0"
  }}
}}
"#,
            name, name
        ),
    }
}

/// Generate tsconfig.json content for a template.
///
/// Returns appropriate TypeScript configuration based on the template type.
/// Component library template includes JSX support for React.
pub fn tsconfig_json(template: Template) -> &'static str {
    match template {
        Template::ComponentLibrary => {
            r#"{
  "compilerOptions": {
    "target": "ES2020",
    "module": "ESNext",
    "lib": ["ES2020"],
    "jsx": "react",
    "moduleResolution": "bundler",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true,
    "resolveJsonModule": true,
    "declaration": true,
    "declarationMap": true,
    "sourceMap": true
  },
  "include": ["src/**/*"],
  "exclude": ["node_modules", "dist"]
}
"#
        }
        _ => {
            r#"{
  "compilerOptions": {
    "target": "ES2020",
    "module": "ESNext",
    "lib": ["ES2020"],
    "moduleResolution": "bundler",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true,
    "resolveJsonModule": true,
    "declaration": true,
    "declarationMap": true,
    "sourceMap": true
  },
  "include": ["src/**/*"],
  "exclude": ["node_modules", "dist"]
}
"#
        }
    }
}

/// Generate fob.config.json for a template.
///
/// Creates bundler configuration appropriate for each template type.
/// Component libraries externalize React and emit type declarations.
/// Meta frameworks target Node.js with multiple entry points.
pub fn joy_config_json(template: Template) -> &'static str {
    match template {
        Template::Library => {
            r#"{
  "bundle": {
    "entries": ["src/index.ts"],
    "format": "esm",
    "output_dir": "dist",
    "minify": false,
    "source_maps": "external",
    "platform": "node",
    "transform": {
      "target": "es2020"
    },
    "typescript_config": {
      "emit_declarations": true,
      "declaration_map": true
    }
  }
}
"#
        }
        Template::App => {
            r#"{
  "bundle": {
    "entries": ["src/main.ts"],
    "format": "esm",
    "output_dir": "dist",
    "minify": true,
    "source_maps": "external",
    "platform": "browser",
    "transform": {
      "target": "es2020"
    }
  }
}
"#
        }
        Template::ComponentLibrary => {
            r#"{
  "bundle": {
    "entries": ["src/index.ts", "src/Button.tsx"],
    "format": "esm",
    "external": ["react"],
    "typescript_config": {
      "emit_declarations": true
    }
  }
}
"#
        }
        Template::MetaFramework => {
            r#"{
  "bundle": {
    "entries": ["src/index.ts", "src/router.ts", "src/server.ts"],
    "format": "esm",
    "platform": "node"
  }
}
"#
        }
    }
}

/// Generate source file content for a template.
///
/// Note: Currently uses static strings for simplicity, but could be refactored
/// to use fob-gen's JsBuilder for type-safe code generation. TypeScript syntax
/// (type annotations) would require TS AST support in fob-gen.
pub fn source_file(template: Template) -> &'static str {
    match template {
        Template::Library => {
            r#"/**
 * Example library entry point.
 *
 * This is a simple library template. Replace with your own code.
 */

export function greet(name: string): string {
  return `Hello, ${name}!`;
}

export function add(a: number, b: number): number {
  return a + b;
}
"#
        }
        Template::App => {
            r#"/**
 * Application entry point.
 */

import './app.css';

function main() {
  const app = document.getElementById('app');
  if (app) {
    app.innerHTML = `
      <h1>Welcome to Joy!</h1>
      <p>Edit src/main.ts to get started.</p>
    `;
  }
}

main();
"#
        }
        Template::ComponentLibrary => {
            r#"export { Button } from './Button';
"#
        }
        Template::MetaFramework => {
            r#"export { createRouter } from './router';
export { createServer } from './server';
"#
        }
    }
}

/// Generate HTML file for app template.
pub fn index_html(name: &str) -> String {
    use fob_gen::{Allocator, HtmlBuilder};
    
    let allocator = Allocator::default();
    let html_builder = HtmlBuilder::new(&allocator);
    
    // Use HtmlBuilder to generate the HTML
    // Note: HtmlBuilder doesn't support custom titles yet, so we'll generate and modify
    html_builder.index_html(Some("/src/main.ts"))
        .unwrap_or_else(|_| {
            // Fallback if builder fails
            format!(
                r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{}</title>
</head>
<body>
  <div id="app"></div>
  <script type="module" src="/src/main.ts"></script>
</body>
</html>
"#,
                name
            )
        })
        .replace("Fob Dev Server", name) // Replace default title with project name
}

/// Generate CSS file for app template.
pub fn app_css() -> &'static str {
    r#":root {
  font-family: system-ui, -apple-system, sans-serif;
  line-height: 1.5;
  color: #213547;
  background-color: #ffffff;
}

#app {
  max-width: 1280px;
  margin: 0 auto;
  padding: 2rem;
  text-align: center;
}

h1 {
  font-size: 3.2em;
  line-height: 1.1;
}
"#
}

/// Generate .gitignore content.
pub fn gitignore() -> &'static str {
    r#"# Dependencies
node_modules/

# Build output
dist/

# Environment files
.env
.env.local

# IDE
.vscode/
.idea/
*.swp
*.swo

# OS
.DS_Store
Thumbs.db

# Logs
*.log
npm-debug.log*
"#
}

/// Generate README.md content for a template.
pub fn readme(name: &str, template: Template) -> String {
    match template {
        Template::Library => format!(
            r#"# {}

A TypeScript library built with Joy.

## Development

```bash
# Install dependencies
npm install

# Build library
npm run build

# Check configuration
npm run check
```

## Usage

```typescript
import {{ greet }} from '{}';

console.log(greet('World'));
```

## License

MIT
"#,
            name, name
        ),
        Template::App => format!(
            r#"# {}

A web application built with Joy.

## Development

```bash
# Install dependencies
npm install

# Start development server
npm run dev

# Build for production
npm run build
```

## License

MIT
"#,
            name
        ),
        Template::ComponentLibrary => format!(
            r#"# {}

A minimal React component library built with fob.

## Installation

```bash
npm install {}
```

## Usage

```tsx
import {{ Button }} from '{}';

function App() {{
  return <Button onClick={{() => console.log('clicked')}}>Click me</Button>;
}}
```

## Development

```bash
npm run dev   # Watch mode
npm run build # Production build
```
"#,
            name, name, name
        ),
        Template::MetaFramework => format!(
            r#"# {}

A minimal meta-framework example showing how to build frameworks with fob.

## Concepts Demonstrated

- **File-based routing** - Simple router implementation
- **Server runtime** - Basic HTTP server with fetch handler
- **Multi-entry bundling** - Separate bundles for different purposes

## Structure

```
src/
├── index.ts   # Main exports
├── router.ts  # Routing logic (~15 lines)
└── server.ts  # Server runtime (~20 lines)
```

## Extending

This is a minimal example. To build a production framework, add:
- SSR/SSG rendering
- Hot module replacement
- Build optimization
- Plugin system
"#,
            name
        ),
    }
}

/// Generate Button.tsx component for ComponentLibrary template.
///
/// Creates a minimal React component with TypeScript props interface.
/// Demonstrates proper typing for React components including children and event handlers.
pub fn button_component() -> &'static str {
    r#"import React from 'react';

export interface ButtonProps {
  children: React.ReactNode;
  onClick?: () => void;
}

export function Button({ children, onClick }: ButtonProps) {
  return <button onClick={onClick}>{children}</button>;
}
"#
}

/// Generate router.ts module for MetaFramework template.
///
/// Implements a minimal file-based router using a Map for path-to-handler lookup.
/// This demonstrates the core routing concept without external dependencies.
pub fn router_module() -> &'static str {
    r#"// Simple file-based router example
export function createRouter() {
  const routes = new Map<string, () => string>();

  return {
    add(path: string, handler: () => string) {
      routes.set(path, handler);
    },
    match(path: string) {
      return routes.get(path);
    }
  };
}
"#
}

/// Generate server.ts module for MetaFramework template.
///
/// Creates a basic HTTP server using the Web Fetch API standard.
/// Demonstrates how to integrate the router with a server runtime.
pub fn server_module() -> &'static str {
    r#"import { createRouter } from './router';

export function createServer() {
  const router = createRouter();

  router.add('/', () => '<h1>Welcome</h1>');

  return {
    fetch(request: Request) {
      const url = new URL(request.url);
      const handler = router.match(url.pathname);

      if (handler) {
        return new Response(handler(), {
          headers: { 'Content-Type': 'text/html' }
        });
      }

      return new Response('Not found', { status: 404 });
    }
  };
}
"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_from_str() {
        assert_eq!(Template::from_str("library"), Some(Template::Library));
        assert_eq!(Template::from_str("lib"), Some(Template::Library));
        assert_eq!(Template::from_str("app"), Some(Template::App));
        assert_eq!(Template::from_str("application"), Some(Template::App));
        assert_eq!(Template::from_str("component-library"), Some(Template::ComponentLibrary));
        assert_eq!(Template::from_str("components"), Some(Template::ComponentLibrary));
        assert_eq!(Template::from_str("meta-framework"), Some(Template::MetaFramework));
        assert_eq!(Template::from_str("framework"), Some(Template::MetaFramework));
        assert_eq!(Template::from_str("invalid"), None);
    }

    #[test]
    fn test_template_name() {
        assert_eq!(Template::Library.name(), "library");
        assert_eq!(Template::App.name(), "app");
        assert_eq!(Template::ComponentLibrary.name(), "component-library");
        assert_eq!(Template::MetaFramework.name(), "meta-framework");
    }

    #[test]
    fn test_package_json_library() {
        let json = package_json("my-lib", Template::Library);
        assert!(json.contains("\"name\": \"my-lib\""));
        assert!(json.contains("\"types\""));
        assert!(json.contains("\"exports\""));
        assert!(json.contains("\"@fob/bundler\""));
    }

    #[test]
    fn test_package_json_app() {
        let json = package_json("my-app", Template::App);
        assert!(json.contains("\"name\": \"my-app\""));
        assert!(json.contains("\"scripts\""));
        assert!(json.contains("\"@fob/browser\""));
    }

    #[test]
    fn test_package_json_component_library() {
        let json = package_json("my-components", Template::ComponentLibrary);
        assert!(json.contains("\"name\": \"my-components\""));
        assert!(json.contains("\"peerDependencies\""));
        assert!(json.contains("\"react\": \"^18.0.0\""));
        assert!(json.contains("\"./Button\""));
    }

    #[test]
    fn test_package_json_meta_framework() {
        let json = package_json("my-framework", Template::MetaFramework);
        assert!(json.contains("\"name\": \"my-framework\""));
        assert!(json.contains("\"bin\""));
        assert!(json.contains("\"my-framework\""));
        assert!(json.contains("\"@fob/bundler\""));
    }

    #[test]
    fn test_tsconfig_json() {
        let json = tsconfig_json(Template::Library);
        assert!(json.contains("\"compilerOptions\""));
        assert!(json.contains("\"strict\": true"));
    }

    #[test]
    fn test_tsconfig_json_component_library() {
        let json = tsconfig_json(Template::ComponentLibrary);
        assert!(json.contains("\"jsx\": \"react\""));
    }

    #[test]
    fn test_joy_config_json() {
        let lib_config = joy_config_json(Template::Library);
        assert!(lib_config.contains("\"bundle\""));
        assert!(lib_config.contains("\"entries\""));
        assert!(lib_config.contains("\"typescript_config\""));
        assert!(lib_config.contains("\"emit_declarations\": true"));

        let app_config = joy_config_json(Template::App);
        assert!(app_config.contains("\"bundle\""));
        assert!(app_config.contains("\"platform\": \"browser\""));

        let comp_config = joy_config_json(Template::ComponentLibrary);
        assert!(comp_config.contains("src/Button.tsx"));
        assert!(comp_config.contains("\"external\": [\"react\"]"));

        let meta_config = joy_config_json(Template::MetaFramework);
        assert!(meta_config.contains("src/router.ts"));
        assert!(meta_config.contains("src/server.ts"));
        assert!(meta_config.contains("\"platform\": \"node\""));
    }

    #[test]
    fn test_source_file() {
        let lib_src = source_file(Template::Library);
        assert!(lib_src.contains("export function greet"));

        let app_src = source_file(Template::App);
        assert!(app_src.contains("function main()"));

        let comp_src = source_file(Template::ComponentLibrary);
        assert!(comp_src.contains("export { Button }"));

        let meta_src = source_file(Template::MetaFramework);
        assert!(meta_src.contains("export { createRouter }"));
        assert!(meta_src.contains("export { createServer }"));
    }

    #[test]
    fn test_index_html() {
        let html = index_html("My App");
        assert!(html.contains("<title>My App</title>"));
        assert!(html.contains("<script type=\"module\""));
    }

    #[test]
    fn test_gitignore() {
        let ignore = gitignore();
        assert!(ignore.contains("node_modules/"));
        assert!(ignore.contains("dist/"));
    }

    #[test]
    fn test_readme() {
        let lib_readme = readme("my-lib", Template::Library);
        assert!(lib_readme.contains("# my-lib"));

        let app_readme = readme("my-app", Template::App);
        assert!(app_readme.contains("# my-app"));

        let comp_readme = readme("my-components", Template::ComponentLibrary);
        assert!(comp_readme.contains("# my-components"));
        assert!(comp_readme.contains("React component library"));

        let meta_readme = readme("my-framework", Template::MetaFramework);
        assert!(meta_readme.contains("# my-framework"));
        assert!(meta_readme.contains("meta-framework"));
    }

    #[test]
    fn test_button_component() {
        let button = button_component();
        assert!(button.contains("export interface ButtonProps"));
        assert!(button.contains("export function Button"));
        assert!(button.contains("React.ReactNode"));
    }

    #[test]
    fn test_router_module() {
        let router = router_module();
        assert!(router.contains("export function createRouter"));
        assert!(router.contains("routes.set"));
        assert!(router.contains("routes.get"));
    }

    #[test]
    fn test_server_module() {
        let server = server_module();
        assert!(server.contains("export function createServer"));
        assert!(server.contains("import { createRouter }"));
        assert!(server.contains("fetch(request: Request)"));
    }
}
