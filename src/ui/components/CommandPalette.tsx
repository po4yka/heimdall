/**
 * CommandPalette — global Cmd-K modal.
 *
 * Mounts at the document level via a portal-style render call. Opens on
 * Cmd-K / Ctrl-K, closes on Escape, focuses the input on open. Filters
 * the command registry as you type and runs the highlighted command on
 * Enter.
 */
import { useEffect, useMemo, useRef, useState } from 'preact/hooks';
import { commandPaletteOpen } from '../state/store';
import {
  buildCommands,
  filterCommands,
  type Command,
  type CommandGroup,
} from '../lib/commands';

interface CommandPaletteProps {
  triggerRescan: () => void | Promise<void>;
  toggleTheme: () => void;
}

const GROUP_LABEL: Record<CommandGroup, string> = {
  navigate: 'Navigate',
  widget: 'Widgets',
  session: 'Sessions',
  project: 'Projects',
  model: 'Models',
  action: 'Actions',
};

const GROUP_ORDER: CommandGroup[] = [
  'navigate',
  'action',
  'widget',
  'session',
  'project',
  'model',
];

export function CommandPalette({ triggerRescan, toggleTheme }: CommandPaletteProps) {
  const open = commandPaletteOpen.value;
  const [query, setQuery] = useState('');
  const [highlight, setHighlight] = useState(0);
  const inputRef = useRef<HTMLInputElement | null>(null);
  const listRef = useRef<HTMLDivElement | null>(null);

  // Build commands fresh on each render — cheap, and ensures the active
  // tab + dataset are always reflected.
  const commands = useMemo(
    () => (open ? buildCommands({ triggerRescan, toggleTheme }) : []),
    [open, triggerRescan, toggleTheme]
  );

  const filtered = useMemo(
    () => filterCommands(commands, query),
    [commands, query]
  );

  // Reset query + highlight when the palette opens.
  useEffect(() => {
    if (!open) return;
    setQuery('');
    setHighlight(0);
    // Defer focus until after the modal renders.
    window.setTimeout(() => inputRef.current?.focus(), 0);
  }, [open]);

  // Clamp highlight when filtered shrinks/grows.
  useEffect(() => {
    if (highlight >= filtered.length) {
      setHighlight(Math.max(0, filtered.length - 1));
    }
  }, [filtered.length, highlight]);

  // Scroll the highlighted row into view.
  useEffect(() => {
    if (!open) return;
    const el = listRef.current?.querySelector<HTMLElement>(
      `[data-cmd-index="${highlight}"]`
    );
    el?.scrollIntoView({ block: 'nearest' });
  }, [highlight, open]);

  if (!open) return null;

  const close = () => { commandPaletteOpen.value = false; };

  const onKeyDown = (e: KeyboardEvent) => {
    if (e.key === 'Escape') {
      e.preventDefault();
      close();
      return;
    }
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      setHighlight(h => Math.min(filtered.length - 1, h + 1));
      return;
    }
    if (e.key === 'ArrowUp') {
      e.preventDefault();
      setHighlight(h => Math.max(0, h - 1));
      return;
    }
    if (e.key === 'Enter') {
      e.preventDefault();
      const cmd = filtered[highlight];
      if (cmd) {
        cmd.run();
        close();
      }
    }
  };

  // Group filtered results for rendering.
  const grouped = new Map<CommandGroup, Command[]>();
  for (const cmd of filtered) {
    const list = grouped.get(cmd.group) ?? [];
    list.push(cmd);
    grouped.set(cmd.group, list);
  }
  // Stable index across groups for highlight tracking.
  let runningIndex = -1;
  const flatIndex = new Map<string, number>();
  for (const cmd of filtered) {
    runningIndex++;
    flatIndex.set(cmd.id, runningIndex);
  }

  return (
    <div
      class="cmd-palette-overlay"
      role="dialog"
      aria-modal="true"
      aria-label="Command palette"
      onClick={(e) => { if (e.target === e.currentTarget) close(); }}
    >
      <div class="cmd-palette" onKeyDown={onKeyDown}>
        <div class="cmd-palette__input-row">
          <span class="cmd-palette__prompt">[&gt;</span>
          <input
            ref={inputRef}
            class="cmd-palette__input"
            type="text"
            placeholder="Search…"
            value={query}
            onInput={(e) => setQuery((e.currentTarget as HTMLInputElement).value)}
            autoComplete="off"
            spellcheck={false}
            enterKeyHint="go"
          />
          <span class="cmd-palette__hint">[esc]</span>
        </div>
        <div class="cmd-palette__list" ref={listRef}>
          {filtered.length === 0 && (
            <div class="cmd-palette__empty">No results</div>
          )}
          {GROUP_ORDER.map(group => {
            const items = grouped.get(group);
            if (!items || items.length === 0) return null;
            return (
              <div key={group} class="cmd-palette__group">
                <div class="cmd-palette__group-label">{GROUP_LABEL[group]}</div>
                {items.map(cmd => {
                  const idx = flatIndex.get(cmd.id) ?? 0;
                  const isActive = idx === highlight;
                  return (
                    <button
                      type="button"
                      key={cmd.id}
                      data-cmd-index={idx}
                      class={`cmd-palette__row${isActive ? ' is-active' : ''}`}
                      onMouseEnter={() => setHighlight(idx)}
                      onClick={() => { cmd.run(); close(); }}
                    >
                      <span class="cmd-palette__row-label">{cmd.label}</span>
                      {cmd.hint && (
                        <span class="cmd-palette__row-hint">{cmd.hint}</span>
                      )}
                    </button>
                  );
                })}
              </div>
            );
          })}
        </div>
        <div class="cmd-palette__footer">
          <span>↑↓ to move</span>
          <span>↵ to run</span>
          <span>esc to close</span>
        </div>
      </div>
    </div>
  );
}
