import { invoke, isTauri } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { dateKey, upcomingFrom, validateNote } from './calendar';
import type { DailyNote, NoteInput } from './calendar';

export const native = isTauri();
export interface Settings {
  preferredDisplay: 'display2' | 'primary' | 'last' | 'manual'; manualMonitor: string | null;
  autoReturn: boolean; rememberPosition: boolean; rememberSize: boolean;
  closeToTray: boolean; showOnStart: boolean; autostart: boolean;
  opacity: number; effect: 'mica' | 'acrylic' | 'blur' | 'none'; cornerRadius: number;
  weekStart: 0 | 1; upcomingCount: 5 | 8 | 10;
}
export interface Monitor { id: string; name: string; number: number | null; primary: boolean; x: number; y: number; width: number; height: number; scale: number }
export const defaults: Settings = { preferredDisplay: 'display2', manualMonitor: null, autoReturn: true, rememberPosition: true, rememberSize: true, closeToTray: true, showOnStart: true, autostart: false, opacity: .72, effect: 'none', cornerRadius: 18, weekStart: 1, upcomingCount: 8 };
const storageKey = 'cds-browser-preview-v1';
function browserNotes(): DailyNote[] { return (JSON.parse(localStorage.getItem(storageKey) ?? '[]') as DailyNote[]).map(n => ({ ...n, endDate: n.endDate ?? null })); }
function browserWrite(notes: DailyNote[]) { localStorage.setItem(storageKey, JSON.stringify(notes)); }
export const api = {
  async list(start: string, end: string): Promise<DailyNote[]> {
    if (native) return invoke('list_notes', { start, end });
    return browserNotes().filter(n => n.date <= end && (n.endDate ?? n.date) >= start).sort((a,b) => a.date.localeCompare(b.date) || (a.time ?? '23:59').localeCompare(b.time ?? '23:59') || b.priority-a.priority);
  },
  async upcoming(limit: number): Promise<DailyNote[]> { return native ? invoke('upcoming_notes', { limit }) : upcomingFrom(browserNotes(), new Date(), limit); },
  async save(input: NoteInput): Promise<string> {
    validateNote(input);
    if (native) return invoke('save_note', { input: { ...input, id: input.id ?? null } });
    const notes = browserNotes(), now = new Date().toISOString();
    const old = notes.find(n => n.id === input.id);
    if (input.id && !old) throw new Error('日签已不存在，请刷新');
    const id = input.id ?? crypto.randomUUID();
    const note: DailyNote = { ...input, id, title: input.title.trim(), completed: old?.completed ?? false, createdAt: old?.createdAt ?? now, updatedAt: now };
    browserWrite([...notes.filter(n => n.id !== id), note]); return id;
  },
  async complete(id: string, completed: boolean): Promise<void> {
    if (native) return invoke('complete_note', { id, completed });
    browserWrite(browserNotes().map(n => n.id === id ? { ...n, completed, updatedAt: new Date().toISOString() } : n));
  },
  async remove(id: string): Promise<void> { if (native) return invoke('delete_note', { id }); browserWrite(browserNotes().filter(n => n.id !== id)); },
  async settings(): Promise<Settings> {
    const settings: Settings = native ? await invoke('get_settings') : { ...defaults, ...JSON.parse(localStorage.getItem(`${storageKey}-settings`) ?? '{}') };
    return { ...settings, effect: 'none', opacity: Math.min(.85, Math.max(.35, settings.opacity)) };
  },
  async saveSettings(settings: Settings): Promise<void> { if (native) return invoke('save_settings', { settings }); localStorage.setItem(`${storageKey}-settings`, JSON.stringify(settings)); },
  async monitors(): Promise<Monitor[]> { return native ? invoke('get_monitors') : []; },
  async backup(): Promise<string> { if (!native) throw new Error('自动 SQLite 备份在桌面应用中可用'); return invoke('backup_data'); },
  async openFolder(): Promise<void> { if (!native) throw new Error('请在桌面应用中打开数据库目录'); return invoke('open_data_folder'); },
  async export(format: 'json' | 'csv'): Promise<string | null> {
    if (native) return invoke('export_data', { format });
    const notes = browserNotes();
    const field = (s: unknown) => { const text = String(s ?? ''); return `"${(/^[\s]*[=+@-]/.test(text) ? "'" : '') + text.replace(/"/g, '""')}"`; };
    const keys: (keyof DailyNote)[] = ['id','date','endDate','time','title','content','category','priority','completed','createdAt','updatedAt'];
    const data = format === 'json' ? JSON.stringify(notes, null, 2) : '\ufeff' + keys.join(',') + '\r\n' + notes.map(n => keys.map(k => field(n[k])).join(',')).join('\r\n');
    const url = URL.createObjectURL(new Blob([data], { type: format === 'json' ? 'application/json' : 'text/csv;charset=utf-8' }));
    const link = document.createElement('a'); link.href = url; link.download = `日签-${dateKey(new Date())}.${format}`; link.click(); setTimeout(() => URL.revokeObjectURL(url), 1000); return link.download;
  },
  async import(): Promise<number | null> { if (!native) throw new Error('请在桌面应用中导入 JSON 日签'); return invoke('import_data'); },
};
export async function windowAction(action: 'minimize' | 'maximize' | 'close') {
  if (!native) return;
  const win = getCurrentWindow();
  return action === 'maximize' ? win.toggleMaximize() : action === 'close' ? win.close() : win.minimize();
}
export async function windowReady() { if (native) await invoke('window_ready'); }
export async function onNativeEvent<T>(name: string, callback: (value: T) => void) {
  if (!native) return () => {};
  return listen<T>(name, event => callback(event.payload));
}
