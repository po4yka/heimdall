import { useRef, useEffect } from 'preact/hooks';

declare const ApexCharts: any;

export function ApexChart({ options, id }: { options: any; id?: string }) {
  const ref = useRef<HTMLDivElement>(null);
  const chartRef = useRef<any>(null);

  // Recreate the chart on every options identity change so labels,
  // callbacks, and closure-backed tooltip data stay in sync.
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
  }, [id, options]);

  return <div ref={ref} id={id} style={{ width: '100%', height: '100%' }} />;
}
