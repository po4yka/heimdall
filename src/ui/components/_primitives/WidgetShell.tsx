import type { ComponentChildren } from 'preact';

export type WidgetDensity = 'normal' | 'compact';

interface WidgetShellProps {
  /** Section heading. Sentence-case per design system. */
  title?: ComponentChildren;
  /** Right-aligned chrome: filters, density toggles, segmented controls. */
  actions?: ComponentChildren;
  /** Density variant — `compact` halves vertical padding. */
  density?: WidgetDensity;
  /** Tag override; defaults to <section>. */
  as?: 'section' | 'div';
  /** Additional class names appended to the shell. */
  className?: string;
  children: ComponentChildren;
}

/**
 * Standard widget container. Replaces the inline
 * `<div class="card card-flat bento-full table-card">` open-class soup
 * scattered throughout `widgets/registry.ts` and `dashboard/view.tsx`.
 *
 * Always renders the same structure: header (title + actions) + body. The
 * body's overflow is `visible` so GridStack `sizeToContent` can measure
 * the natural content height.
 */
export function WidgetShell({
  title,
  actions,
  density = 'normal',
  as: Tag = 'section',
  className,
  children,
}: WidgetShellProps) {
  const cls = ['widget-shell', `widget-shell--${density}`, className].filter(Boolean).join(' ');
  const showHeader = title !== undefined || actions !== undefined;

  return (
    <Tag class={cls}>
      {showHeader && (
        <header class="widget-shell__header">
          {title !== undefined && <h2 class="widget-shell__title">{title}</h2>}
          {actions !== undefined && <div class="widget-shell__actions">{actions}</div>}
        </header>
      )}
      <div class="widget-shell__body">{children}</div>
    </Tag>
  );
}
