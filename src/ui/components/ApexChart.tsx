import { useRef, useEffect, useMemo } from 'preact/hooks';

declare const ApexCharts: any;

export function ApexChart({ options, id }: { options: any; id?: string }) {
  const ref = useRef<HTMLDivElement>(null);
  const chartRef = useRef<any>(null);

  const optionsKey = useMemo(
    () => JSON.stringify(options, (_key, val) => typeof val === 'function' ? undefined : val),
    [options],
  );

  useEffect(() => {
    if (chartRef.current) chartRef.current.destroy();
    if (ref.current && options) {
      chartRef.current = new ApexCharts(ref.current, options);
      chartRef.current.render();
    }
    return () => { chartRef.current?.destroy(); chartRef.current = null; };
  }, [optionsKey]);

  return <div ref={ref} id={id} style={{ width: '100%', height: '100%' }} />;
}
