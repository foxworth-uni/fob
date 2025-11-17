// Simple file-based router implementation
// In a real meta-framework, this would handle dynamic routes,
// nested layouts, and more sophisticated matching

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
