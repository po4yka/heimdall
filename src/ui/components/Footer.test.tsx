import { describe, expect, it } from 'vitest';
import { Footer } from './Footer';

function collectText(node: unknown): string[] {
  if (typeof node === 'string' || typeof node === 'number') return [String(node)];
  if (Array.isArray(node)) return node.flatMap(collectText);
  if (!node || typeof node !== 'object') return [];
  const vnode = node as { props?: { children?: unknown } };
  return collectText(vnode.props?.children);
}

function collectLinks(node: unknown): Array<{ href: string | undefined; children: unknown }> {
  if (Array.isArray(node)) return node.flatMap(collectLinks);
  if (!node || typeof node !== 'object') return [];
  const vnode = node as { type?: unknown; props?: { href?: string; children?: unknown } };
  const own = vnode.type === 'a' ? [{ href: vnode.props?.href, children: vnode.props?.children }] : [];
  return [...own, ...collectLinks(vnode.props?.children)];
}

describe('Footer', () => {
  it('renders pricing sources and project links', () => {
    const vnode = Footer();
    const text = collectText(vnode).join(' ');
    const links = collectLinks(vnode);

    expect(text).toContain('Cost estimates based on Anthropic and OpenAI API pricing');
    expect(text).toContain('Local dashboard totals are estimates');
    expect(links.map(link => link.href)).toEqual([
      'https://docs.anthropic.com/en/docs/about-claude/pricing',
      'https://developers.openai.com/api/docs/pricing',
      'https://github.com/po4yka/claude-usage-tracker',
    ]);
  });
});
