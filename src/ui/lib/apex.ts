export type ApexChartType = 'bar' | 'donut' | 'line' | 'pie' | string;
export type ApexTooltipPosition =
  | 'topLeft'
  | 'topRight'
  | 'bottomLeft'
  | 'bottomRight'
  | string;

export interface ApexTooltipFormatterContext {
  dataPointIndex?: number;
}

export interface ApexDataPointSelectionConfig {
  dataPointIndex: number;
}

export interface ApexLabelStyle {
  colors?: string;
  fontFamily?: string;
  fontSize?: string;
  letterSpacing?: string;
  maxWidth?: number;
}

export interface ApexAxisLabels {
  style?: ApexLabelStyle;
  formatter?: (value: number, context?: ApexTooltipFormatterContext) => string;
  hideOverlappingLabels?: boolean;
  maxWidth?: number;
  rotate?: number;
  maxHeight?: number;
}

export interface ApexAxisConfig {
  type?: 'category' | 'datetime' | 'numeric';
  categories?: string[];
  min?: number;
  max?: number;
  tickAmount?: number;
  labels?: ApexAxisLabels;
  axisBorder?: { color?: string; show?: boolean };
  axisTicks?: { color?: string; show?: boolean };
  opposite?: boolean;
  show?: boolean;
}

export interface ApexChartOptions {
  type?: ApexChartType;
  height?: string | number;
  width?: string | number;
  background?: string;
  toolbar?: { show?: boolean };
  fontFamily?: string;
  animations?: { enabled?: boolean };
  stacked?: boolean;
  sparkline?: { enabled?: boolean };
  events?: {
    dataPointSelection?: (
      event: unknown,
      context: unknown,
      config: ApexDataPointSelectionConfig
    ) => void;
  };
}

export interface ApexLegendOptions {
  show?: boolean;
  position?: 'top' | 'bottom';
  fontFamily?: string;
  fontSize?: string;
  labels?: { colors?: string; fontFamily?: string };
  markers?: { width?: number; height?: number; radius?: number };
  itemMargin?: { horizontal?: number; vertical?: number };
}

export interface ApexSeries {
  name?: string;
  type?: string;
  /// ApexCharts accepts plain numbers for category-axis charts and
  /// `{x, y}` pairs for time-series. Both shapes are passed through to the
  /// underlying chart unchanged.
  data: number[] | Array<number | null> | Array<{ x: number | string; y: number | null }>;
  color?: string;
}

export interface ApexOptions {
  chart?: ApexChartOptions;
  theme?: { mode?: 'light' | 'dark' };
  legend?: ApexLegendOptions;
  grid?: {
    borderColor?: string;
    strokeDashArray?: number;
    xaxis?: { lines?: { show?: boolean } };
    yaxis?: { lines?: { show?: boolean } };
  };
  xaxis?: ApexAxisConfig;
  yaxis?: ApexAxisConfig;
  stroke?: { width?: number | number[]; curve?: 'straight' | 'smooth' | string; colors?: string[]; dashArray?: number | number[] };
  tooltip?: {
    theme?: 'light' | 'dark';
    enabled?: boolean;
    style?: { fontFamily?: string; fontSize?: string };
    fixed?: {
      enabled?: boolean;
      position?: ApexTooltipPosition;
      offsetX?: number;
      offsetY?: number;
    };
    custom?: (context: { seriesIndex: number; dataPointIndex: number }) => string;
    y?: {
      formatter?: (value: number, context?: ApexTooltipFormatterContext) => string;
    };
  };
  dataLabels?: { enabled?: boolean };
  fill?: { type?: 'solid' | string | Array<'solid' | string>; opacity?: number | number[] };
  series?: ApexSeries[] | number[];
  colors?: string[];
  plotOptions?: {
    bar?: {
      horizontal?: boolean;
      barHeight?: string;
      columnWidth?: string;
      borderRadius?: number;
    };
    pie?: {
      expandOnClick?: boolean;
      donut?: {
        size?: string;
        labels?: {
          show?: boolean;
          total?: {
            show?: boolean;
            label?: string;
            fontFamily?: string;
            fontSize?: string;
            color?: string;
            formatter?: () => string;
          };
          value?: {
            fontFamily?: string;
            fontSize?: string;
            color?: string;
            formatter?: (value: string) => string;
          };
          name?: {
            fontFamily?: string;
            fontSize?: string;
            color?: string;
          };
        };
      };
    };
  };
  labels?: string[];
  states?: {
    hover?: { filter?: { type?: string; value?: number } };
    active?: { filter?: { type?: string; value?: number } };
  };
  annotations?: {
    xaxis?: Array<{
      x: number | string;
      x2?: number | string;
      borderColor?: string;
      strokeDashArray?: number;
      label?: {
        text?: string;
        borderColor?: string;
        offsetY?: number;
        style?: {
          color?: string;
          background?: string;
          fontFamily?: string;
          fontSize?: string;
        };
      };
    }>;
    points?: Array<{
      x: number | string;
      y?: number | null;
      marker?: {
        size?: number;
        fillColor?: string;
        strokeColor?: string;
        radius?: number;
      };
      label?: {
        text?: string;
        style?: {
          color?: string;
          background?: string;
          fontFamily?: string;
          fontSize?: string;
        };
      };
    }>;
  };
  markers?: { size?: number | number[]; strokeWidth?: number; hover?: { size?: number } };
}

export interface ApexChartInstance {
  render(): Promise<void> | void;
  destroy(): void;
}

export interface ApexChartConstructor {
  new (element: HTMLElement, options: ApexOptions): ApexChartInstance;
}
