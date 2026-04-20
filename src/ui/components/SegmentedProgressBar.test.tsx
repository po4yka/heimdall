import { describe, expect, it } from 'vitest';
import { SegmentedProgressBar } from './SegmentedProgressBar';

describe('SegmentedProgressBar', () => {
  it('renders bounded progress with the expected accessibility contract', () => {
    const vnode = SegmentedProgressBar({
      value: 45,
      max: 90,
      segments: 10,
      status: 'warning',
      'aria-label': 'Usage',
    }) as { props: Record<string, unknown> };
    const segments = vnode.props['children'] as Array<{ props: { style: { background: string } } }>;

    expect(vnode.props['role']).toBe('progressbar');
    expect(vnode.props['aria-label']).toBe('Usage');
    expect(vnode.props['aria-valuenow']).toBe(50);
    expect(segments).toHaveLength(10);
    expect(segments[0]?.props.style.background).toBe('var(--warning)');
    expect(segments[9]?.props.style.background).toBe('var(--border)');
  });

  it('caps overflow at 100 percent and switches to accent styling', () => {
    const vnode = SegmentedProgressBar({
      value: 15,
      max: 10,
      segments: 5,
      size: 'compact',
    }) as { props: Record<string, unknown> };
    const segments = vnode.props['children'] as Array<{ props: { style: { background: string } } }>;

    expect(vnode.props['class']).toContain('segmented-bar--compact');
    expect(vnode.props['aria-valuenow']).toBe(100);
    expect(segments.every(segment => segment.props.style.background === 'var(--accent)')).toBe(
      true
    );
  });
});
