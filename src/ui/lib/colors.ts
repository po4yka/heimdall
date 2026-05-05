/**
 * Resolve a CSS custom property to its computed value at render time.
 *
 * ApexCharts injects strings directly into SVG attributes; some attributes
 * (legend marker fills, tooltip backgrounds, annotation markers) do not
 * resolve `var(...)` expressions, so we feed them concrete hex values here.
 *
 * @param name     CSS custom property name, e.g. '--text-primary'
 * @param fallback Concrete colour to use in SSR / test environments where
 *                 `window` is unavailable, or when the property is unset.
 */
export function resolveCssVar(name: string, fallback: string): string {
  if (typeof window === 'undefined') return fallback;
  const value = getComputedStyle(document.documentElement)
    .getPropertyValue(name)
    .trim();
  return value || fallback;
}
