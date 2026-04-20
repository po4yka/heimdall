import { fmt, fmtCost, fmtCredits } from '../../lib/format';

export function renderNumberCell(value: number, formatter: (value: number) => string = fmt) {
  return <span class="num">{formatter(value)}</span>;
}

export function renderCreditsCell(value: number | null) {
  return <span class="num">{fmtCredits(value)}</span>;
}

export function renderCostCell(value: number, isBillable = true) {
  return isBillable ? (
    <span class="cost">{fmtCost(value)}</span>
  ) : (
    <span class="cost-na">n/a</span>
  );
}

export function renderTagCell(
  label: string,
  onSelect?: (() => void) | undefined,
  className = 'table-action-btn table-action-btn--tag'
) {
  if (!onSelect) return <span class="model-tag">{label}</span>;
  return (
    <button type="button" class={className} onClick={onSelect}>
      <span class="model-tag">{label}</span>
    </button>
  );
}

export function renderActionCell(
  label: string,
  title: string,
  onSelect?: (() => void) | undefined,
  className = 'table-action-btn'
) {
  if (!onSelect) return <span title={title}>{label}</span>;
  return (
    <button type="button" class={className} title={title} onClick={onSelect}>
      {label}
    </button>
  );
}
