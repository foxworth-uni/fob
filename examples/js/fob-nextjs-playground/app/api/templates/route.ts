import { NextRequest, NextResponse } from 'next/server';
import { templates, TemplateName } from '@/lib/templates';

export async function GET(request: NextRequest) {
  const { searchParams } = new URL(request.url);
  const name = searchParams.get('name') as TemplateName;

  if (!name) {
    return NextResponse.json({
      templates: Object.keys(templates),
    });
  }

  const content = templates[name];
  if (!content) {
    return NextResponse.json({ error: `Template "${name}" not found` }, { status: 404 });
  }

  return NextResponse.json({
    content,
    filename: `${name}.tsx`,
  });
}
