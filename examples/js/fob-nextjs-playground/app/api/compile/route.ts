import { NextRequest, NextResponse } from 'next/server';
import { bundle, recordBuild } from '@/lib/bundler';

export async function POST(request: NextRequest) {
  const startTime = performance.now();

  try {
    const body = await request.json();
    const { code, filename = 'main.tsx' } = body;

    if (!code || typeof code !== 'string') {
      return NextResponse.json({ error: 'Missing or invalid "code" field' }, { status: 400 });
    }

    const result = await bundle(code, filename);

    const buildInfo = {
      timestamp: Date.now(),
      duration: Math.round(performance.now() - startTime),
      modules: result.stats.totalModules,
      chunks: result.stats.totalChunks,
      size: result.stats.totalSize,
      cacheHitRate: result.stats.cacheHitRate,
      success: true,
    };

    recordBuild(buildInfo);

    return NextResponse.json({
      output: result.chunks[0]?.code || '',
      stats: {
        duration: buildInfo.duration,
        modules: buildInfo.modules,
        chunks: buildInfo.chunks,
        size: buildInfo.size,
        cacheHitRate: buildInfo.cacheHitRate,
      },
    });
  } catch (error) {
    const buildInfo = {
      timestamp: Date.now(),
      duration: Math.round(performance.now() - startTime),
      modules: 0,
      chunks: 0,
      size: 0,
      cacheHitRate: 0,
      success: false,
      error: error instanceof Error ? error.message : 'Unknown error',
    };

    recordBuild(buildInfo);

    return NextResponse.json(
      {
        error: error instanceof Error ? error.message : 'Compilation failed',
      },
      { status: 500 }
    );
  }
}
