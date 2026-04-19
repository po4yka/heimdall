import { useRef, useEffect, useMemo } from 'preact/hooks';

declare const ApexCharts: any;

export function ApexChart({ options, id }: { options: any; id?: string }) {
  const ref = useRef<HTMLDivElement>(null);
  const chartRef = useRef<any>(null);

  const themeMode = options.theme?.mode ?? '';

  const optionsKey = useMemo(() => {
    const s = options.series;
    const type = options.chart?.type ?? '';
    // Include themeMode so toggling theme fully recreates the chart and
    // restores per-chart option functions (e.g., axis formatters) that the
    // lightweight updateOptions path was dropping.
    if (Array.isArray(s)) {
      const parts = s.map((ss: any) => {
        const d = ss.data;
        if (!d || !d.length) return '0';
        return `${d.length}:${d[0]}:${d[d.length - 1]}`;
      });
      return `${type}-${themeMode}-${parts.join(',')}`;
    }
    return `${type}-${themeMode}-${s?.length ?? 0}`;
  }, [options, themeMode]);

  // Full destroy/recreate when data or theme changes.
  useEffect(() => {
    if (chartRef.current) chartRef.current.destroy();
    if (!ref.current || !options) {
      return () => { chartRef.current?.destroy(); chartRef.current = null; };
    }
    // ApexCharts' `height: '100%'` is unreliable because it reads the
    // parent's clientHeight synchronously during `new ApexCharts(...)`; if
    // layout hasn't settled, the chart falls back to 150px. Wait for the
    // next animation frame so the chart-wrap's CSS height (240px / 300px)
    // has resolved, then pass a numeric height. Tall chart-wraps use the
    // `.tall` class modifier (300px) — the explicit fallback keeps the
    // chart usable even if clientHeight is still zero.
    let cancelled = false;
    const raf = requestAnimationFrame(() => {
      if (cancelled || !ref.current) return;
      const parent = ref.current.parentElement;
      let h = parent?.clientHeight ?? 0;
      if (h <= 0) h = parent?.classList.contains('tall') ? 300 : 240;
      const opts = { ...options, chart: { ...options.chart, height: h } };
      chartRef.current = new ApexCharts(ref.current, opts);
      chartRef.current.render();
    });
    return () => {
      cancelled = true;
      cancelAnimationFrame(raf);
      chartRef.current?.destroy();
      chartRef.current = null;
    };
  }, [optionsKey]);

  return <div ref={ref} id={id} style={{ width: '100%', height: '100%' }} />;
}
