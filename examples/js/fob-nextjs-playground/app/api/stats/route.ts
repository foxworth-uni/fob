import { NextResponse } from 'next/server';
import { getBuildStats } from '@/lib/bundler';

export async function GET() {
  const stats = getBuildStats();
  return NextResponse.json(stats);
}
