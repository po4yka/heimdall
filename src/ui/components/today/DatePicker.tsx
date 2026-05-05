import { selectedDate, todayData } from '../../state/store';

function addDays(dateStr: string, delta: number): string {
  const d = new Date(`${dateStr}T00:00:00`);
  d.setDate(d.getDate() + delta);
  return d.toISOString().slice(0, 10);
}

function localToday(): string {
  const now = new Date();
  const y = now.getFullYear();
  const m = String(now.getMonth() + 1).padStart(2, '0');
  const d = String(now.getDate()).padStart(2, '0');
  return `${y}-${m}-${d}`;
}

interface DatePickerProps {
  onDateChange: (date: string | null) => void;
}

export function DatePicker({ onDateChange }: DatePickerProps) {
  const today = localToday();
  const resolvedDate = selectedDate.value ?? todayData.value?.day ?? today;
  const isToday = resolvedDate === today || selectedDate.value === null;

  function previousDay() {
    const next = addDays(resolvedDate, -1);
    selectedDate.value = next;
    onDateChange(next);
  }

  function nextDay() {
    if (resolvedDate >= today) return; // do not navigate into future
    const next = addDays(resolvedDate, 1);
    selectedDate.value = next === today ? null : next;
    onDateChange(next === today ? null : next);
  }

  function goToday() {
    selectedDate.value = null;
    onDateChange(null);
  }

  function onPick(e: Event) {
    const val = (e.target as HTMLInputElement).value;
    if (!val) return;
    const next = val === today ? null : val;
    selectedDate.value = next;
    onDateChange(next);
  }

  return (
    <div class="date-picker">
      <button
        type="button"
        class="date-picker-btn"
        onClick={previousDay}
        aria-label="Previous day"
      >
        &#9664;
      </button>
      <input
        type="date"
        class="date-picker-input"
        value={resolvedDate}
        max={today}
        onChange={onPick}
        aria-label="Select date"
      />
      <button
        type="button"
        class="date-picker-btn"
        onClick={nextDay}
        disabled={resolvedDate >= today}
        aria-label="Next day"
      >
        &#9654;
      </button>
      <button
        type="button"
        class={`date-picker-btn date-picker-today-btn${isToday ? ' date-picker-btn--active' : ''}`}
        onClick={goToday}
        disabled={isToday}
        aria-label="Go to today"
      >
        Today
      </button>
    </div>
  );
}
