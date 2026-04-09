// ── CSV Export Utilities ───────────────────────────────────────────────

export function csvField(val: unknown): string {
  const s = String(val);
  // Prevent CSV injection (formula execution in spreadsheets)
  const needsPrefix = /^[=+\-@\t\r]/.test(s);
  const escaped = needsPrefix ? "'" + s : s;
  if (escaped.includes(',') || escaped.includes('"') || escaped.includes('\n')) {
    return '"' + escaped.replace(/"/g, '""') + '"';
  }
  return escaped;
}

export function csvTimestamp(): string {
  const d = new Date();
  return d.getFullYear() + '-' + String(d.getMonth() + 1).padStart(2, '0') + '-' + String(d.getDate()).padStart(2, '0')
    + '_' + String(d.getHours()).padStart(2, '0') + String(d.getMinutes()).padStart(2, '0');
}

export function downloadCSV(reportType: string, header: string[], rows: unknown[][]): void {
  const lines = [header.map(csvField).join(',')];
  for (const row of rows) lines.push(row.map(csvField).join(','));
  const blob = new Blob([lines.join('\n')], { type: 'text/csv;charset=utf-8;' });
  const a = document.createElement('a');
  a.href = URL.createObjectURL(blob);
  a.download = reportType + '_' + csvTimestamp() + '.csv';
  a.click();
  setTimeout(() => URL.revokeObjectURL(a.href), 1000);
}
