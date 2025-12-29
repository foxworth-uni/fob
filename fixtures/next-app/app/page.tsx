import { loadMdxModule } from '@fob/next';
import { MDXProvider } from '@fob/next';
import path from 'node:path';
import { Callout } from '../components/Callout';

export default async function Home() {
  // Load MDX module (cached per request)
  const mod = await loadMdxModule({
    filePath: path.join(process.cwd(), 'content/post.mdx'),
  });

  const Content = mod.default;

  return (
    <div className="flex min-h-screen items-center justify-center bg-zinc-50 font-sans dark:bg-black">
      <main className="flex min-h-screen w-full max-w-3xl flex-col items-center justify-between py-32 px-16 bg-white dark:bg-black sm:items-start">
        <MDXProvider
          components={{
            h1: (props) => (
              <h1
                className="text-4xl font-bold mt-8 mb-4 text-black dark:text-zinc-50"
                {...props}
              />
            ),
            h2: (props) => (
              <h2
                className="text-3xl font-semibold mt-6 mb-3 text-black dark:text-zinc-50"
                {...props}
              />
            ),
            p: (props) => (
              <p className="text-lg leading-8 my-4 text-zinc-600 dark:text-zinc-400" {...props} />
            ),
            code: (props) => (
              <code
                className="bg-zinc-100 dark:bg-zinc-800 px-1 py-0.5 rounded text-sm"
                {...props}
              />
            ),
            pre: (props) => (
              <pre
                className="bg-zinc-100 dark:bg-zinc-800 p-4 rounded-lg overflow-x-auto my-4"
                {...props}
              />
            ),
            Callout,
          }}
        >
          <Content />
        </MDXProvider>
      </main>
    </div>
  );
}
