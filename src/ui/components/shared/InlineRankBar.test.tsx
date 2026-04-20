import { describe, expect, it } from 'vitest';
import { InlineRankBar } from './InlineRankBar';

describe('InlineRankBar', () => {
  it('renders the width overlay and tooltip from the value ratio', () => {
    const vnode = InlineRankBar({ value: 25, max: 50, label: '25' }) as {
      props: { title: string; children: Array<{ props: Record<string, unknown> }> };
    };
    const background = vnode.props.children[0]!;
    const label = vnode.props.children[1]!;

    expect(vnode.props.title).toBe('25 (50.0% of max 50)');
    expect(background.props['style']).toMatchObject({ width: '50%', opacity: 0.12 });
    expect(label.props['children']).toBe('25');
  });
});
