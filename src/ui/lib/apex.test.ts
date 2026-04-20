import { describe, expect, expectTypeOf, it } from 'vitest';
import * as apexModule from './apex';
import type { ApexChartConstructor, ApexChartInstance, ApexOptions, ApexSeries } from './apex';

describe('apex types', () => {
  it('accepts the chart option and constructor contracts used by the dashboard', () => {
    const series: ApexSeries = { name: 'Input', data: [1, 2, 3] };
    const options: ApexOptions = {
      chart: {
        type: 'bar',
        events: {
          dataPointSelection: () => undefined,
        },
      },
      series: [series],
      tooltip: {
        custom: ({ seriesIndex, dataPointIndex }) => `${seriesIndex}:${dataPointIndex}`,
      },
    };
    const Constructor = class implements ApexChartInstance {
      constructor(_element: HTMLElement, _options: ApexOptions) {}
      render(): void {}
      destroy(): void {}
    };

    expectTypeOf(series).toEqualTypeOf<ApexSeries>();
    expectTypeOf(options).toEqualTypeOf<ApexOptions>();
    expectTypeOf(Constructor).toMatchTypeOf<ApexChartConstructor>();
    expect(Object.keys(apexModule)).toHaveLength(0);
  });
});
