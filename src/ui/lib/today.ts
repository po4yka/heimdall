import { todayData, todayLoading } from '../state/store';
import type { TodayResponse } from '../state/types';

export async function loadToday(
  date: string | null,
  tzOffsetMin: number
): Promise<TodayResponse | null> {
  todayLoading.value = true;
  try {
    let url = `/api/today?tz_offset_min=${tzOffsetMin}`;
    if (date) url += `&date=${encodeURIComponent(date)}`;
    const resp = await fetch(url);
    if (!resp.ok) return null;
    const data = (await resp.json()) as TodayResponse;
    todayData.value = data;
    return data;
  } catch {
    return null;
  } finally {
    todayLoading.value = false;
  }
}
