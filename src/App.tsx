import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import type { CSSProperties } from 'react';
import { addDays, endOfMonth, format, isSameMonth, parseISO, startOfMonth } from 'date-fns';
import { zhCN } from 'date-fns/locale';
import { ArrowUpRight, CalendarDays, Check, ChevronLeft, ChevronRight, Circle, Clock3, Maximize2, Minus, Pencil, Plus, Settings2, ShieldCheck, Sun, Trash2, X } from 'lucide-react';
import CalendarViews from './CalendarViews';
import { categoryClass, dateKey, groupNotes, navigate, visibleRange } from './calendar';
import type { DailyNote, NoteInput, View } from './calendar';
import { api, defaults, native, onNativeEvent, windowAction, windowReady } from './services';
import type { Settings } from './services';
import NoteEditor from './NoteEditor';
import type { EditorTarget } from './NoteEditor';
import SettingsPanel from './SettingsPanel';
import Dialog from './Dialog';
import './App.css';

const message = (e: unknown) => e instanceof Error ? e.message : String(e);
export default function App() {
  const [now, setNow] = useState(new Date()), [anchor, setAnchor] = useState(new Date());
  const [selected, setSelected] = useState(dateKey(new Date())), [view, setView] = useState<View>('month');
  const [settings, setSettings] = useState<Settings>(defaults), [ready, setReady] = useState(false);
  const [notes, setNotes] = useState<DailyNote[]>([]), [upcoming, setUpcoming] = useState<DailyNote[]>([]);
  const [editor, setEditor] = useState<EditorTarget | null>(null), [settingsOpen, setSettingsOpen] = useState(false);
  const [deleting, setDeleting] = useState<DailyNote | null>(null), [busy, setBusy] = useState(false), [error, setError] = useState(''), [loading, setLoading] = useState(true);
  const windowShown = useRef(false), refreshVersion = useRef(0), actions = useRef({ today: () => {}, add: () => {}, settings: () => {}, refresh: async () => {} });
  const today = dateKey(now);
  const [start, end] = visibleRange(anchor, view, settings.weekStart);
  const grouped = useMemo(() => groupNotes(notes, start, end), [notes, start, end]);
  const refresh = useCallback(async () => {
    const version = ++refreshVersion.current;
    setLoading(true);
    try {
      const [list, next] = await Promise.all([api.list(start, end), api.upcoming(settings.upcomingCount)]);
      if (version === refreshVersion.current) { setNotes(list); setUpcoming(next); }
    } finally { if (version === refreshVersion.current) setLoading(false); }
  }, [start, end, settings.upcomingCount]);
  const goToday = () => { const day = new Date(); setNow(day); setAnchor(day); setSelected(dateKey(day)); };
  const openEditor = (date = selected, quick = false, time?: string, note?: DailyNote) => { setEditor({ date, quick, time, note }); };
  actions.current = { today: goToday, add: () => openEditor(dateKey(new Date()), true), settings: () => setSettingsOpen(true), refresh };
  useEffect(() => {
    let active = true;
    api.settings().then(s => { if (active) setSettings(s); }).catch(e => { if (active) setError(message(e)); }).finally(() => { if (active) setReady(true); });
    return () => { active = false; };
  }, []);
  useEffect(() => { if (ready) void refresh().catch(e => setError(message(e))); }, [ready, refresh]);
  useEffect(() => { if (ready && !loading && !windowShown.current) { windowShown.current = true; void windowReady().catch(e => setError(message(e))); } }, [ready, loading]);
  useEffect(() => {
    const unlisteners: (() => void)[] = []; let active = true;
    const bind = async <T,>(name: string, callback: (value: T) => void) => {
      try { const fn = await onNativeEvent(name, callback); if (active) unlisteners.push(fn); else fn(); } catch (e) { if (active) setError(message(e)); }
    };
    void bind<string>('app-error', value => setError(value));
    void bind<string>('tray-action', value => { if (value === 'today') actions.current.today(); else if (value === 'add') actions.current.add(); else if (value === 'settings') actions.current.settings(); });
    const tick = () => { setNow(new Date()); void actions.current.refresh().catch(e => setError(message(e))); };
    const visibility = () => { if (document.visibilityState === 'visible') tick(); };
    const timer = setInterval(tick, 60_000); document.addEventListener('visibilitychange', visibility);
    return () => { active = false; unlisteners.forEach(fn => fn()); clearInterval(timer); document.removeEventListener('visibilitychange', visibility); };
  }, []);
  useEffect(() => {
    const key = (e: KeyboardEvent) => { if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'n') { e.preventDefault(); if (!editor && !settingsOpen && !deleting) openEditor(); } };
    document.addEventListener('keydown', key); return () => document.removeEventListener('keydown', key);
  }, [selected, editor, settingsOpen, deleting]);
  const select = (date: string, quick = false, time?: string) => { setSelected(date); if (view !== 'month' || !isSameMonth(parseISO(date), anchor)) setAnchor(parseISO(date)); if (view === 'year') setView('month'); if (quick) openEditor(date, true, time); };
  const changeView = (mode: View) => { setView(mode); if (mode === 'day') setAnchor(parseISO(selected)); };
  const move = (amount: number) => { const date = navigate(anchor, view, amount); if (dateKey(date) < '1900-01-01' || dateKey(date) > '9999-12-31') return; setAnchor(date); setSelected(dateKey(date)); };
  const save = async (input: NoteInput) => {
    await api.save(input); setEditor(null); setSelected(input.date); setAnchor(parseISO(input.date));
    await refresh().catch(e => setError(message(e)));
  };
  const mutate = async (fn: () => Promise<void>) => {
    if (busy) return; setBusy(true); setError('');
    try { await fn(); await refresh(); } catch (e) { setError(message(e)); } finally { setBusy(false); }
  };
  const dayNotes = grouped.get(selected) ?? [], completed = dayNotes.filter(n => n.completed).length;
  const monthNotes = notes.filter(n => n.date <= dateKey(endOfMonth(anchor)) && (n.endDate ?? n.date) >= dateKey(startOfMonth(anchor)));
  const title = view === 'year' ? `${anchor.getFullYear()} 年` : view === 'day' ? format(anchor, 'M 月 d 日') : format(anchor, 'LLLL', { locale: zhCN });
  const subtitle = view === 'year' ? 'A YEAR OF LITTLE THINGS' : view === 'day' ? format(anchor, 'EEEE · yyyy', { locale: zhCN }) : format(anchor, 'MMMM yyyy').toUpperCase();
  const relative = (date: string) => date === today ? '今天' : date === dateKey(addDays(now, 1)) ? '明天' : format(parseISO(date), 'M / d');
  return <main className={`app-shell ${native ? 'desktop' : 'preview'}`} style={{ '--panel-opacity': Math.min(.85, settings.opacity), '--corner': `${settings.cornerRadius}px` } as CSSProperties}>
    <header className="titlebar" data-tauri-drag-region><div className="brand" data-tauri-drag-region><Sun size={19} /><strong data-tauri-drag-region>日签</strong><span data-tauri-drag-region>把日子，轻轻安排好。</span></div><div className="window-controls">{!native && <span className="preview-badge">浏览器预览</span>}<button className="icon-button" aria-label="设置" onClick={() => setSettingsOpen(true)}><Settings2 size={16} /></button>{native && <><button aria-label="最小化" className="icon-button" onClick={() => windowAction('minimize').catch(e => setError(message(e)))}><Minus size={16} /></button><button aria-label="最大化或还原" className="icon-button" onClick={() => windowAction('maximize').catch(e => setError(message(e)))}><Maximize2 size={13} /></button><button aria-label="关闭窗口" className="icon-button close-window" onClick={() => windowAction('close').catch(e => setError(message(e)))}><X size={17} /></button></>}</div></header>
    {error && <div className="error-banner" role="alert"><span>{error}</span><button className="text-button" onClick={() => { setError(''); void refresh().catch(e => setError(message(e))); }}>重试</button><button className="icon-button" aria-label="关闭提示" onClick={() => setError('')}><X size={15} /></button></div>}
    <div className="workspace"><section className="calendar-main">
      <div className="calendar-heading"><div><p className="eyebrow">{subtitle}</p><h1>{title}<span className="title-dot">.</span></h1></div><div className="calendar-toolbar"><div className="period-navigation"><button className="icon-button" aria-label="上一页" onClick={() => move(-1)}><ChevronLeft size={17} /></button><button className="today-button" onClick={goToday}>今天</button><button className="icon-button" aria-label="下一页" onClick={() => move(1)}><ChevronRight size={17} /></button></div><div className="view-switch" aria-label="日历视图">{(['day','month','year'] as View[]).map((v,i) => <button key={v} aria-pressed={view === v} onClick={() => changeView(v)}>{['日','月','年'][i]}</button>)}</div><button className="add-button" title="新建日签 · Ctrl N" aria-label="新建日签" onClick={() => openEditor()}><Plus size={22} /></button></div></div>
      <div className="calendar-summary"><span><i className="status-dot" />{view === 'year' ? `${notes.length} 条年度日签` : view === 'day' ? `${dayNotes.length} 条当日日签` : `${monthNotes.length} 条本月日签`}</span><span>{format(now, 'M 月 d 日 EEEE', { locale: zhCN })}</span></div>
      <CalendarViews anchor={anchor} selected={selected} today={today} view={view} weekStart={settings.weekStart} grouped={grouped} select={select} month={date => { setAnchor(date); setSelected(dateKey(date)); setView('month'); }} edit={note => openEditor(note.date, false, undefined, note)} />
      <section className="daily-section" aria-label="当天日签"><div className="section-heading"><div><p className="eyebrow">YOUR DAY, AT A GLANCE</p><h2>{format(parseISO(selected), 'M 月 d 日')}<span>{format(parseISO(selected), 'EEEE', { locale: zhCN })}</span></h2></div><span className="note-progress">{dayNotes.length ? `${completed} / ${dayNotes.length} 已完成` : '留一点空间给自己'}</span></div>
        <div className="daily-list" aria-busy={loading}>{dayNotes.length ? dayNotes.map(note => <article className={`note-row ${note.completed ? 'completed' : ''}`} key={note.id}>
          <button className={`complete-button ${categoryClass[note.category]}`} disabled={busy} aria-label={`${note.completed ? '取消完成' : '完成'} ${note.title}`} onClick={() => mutate(() => api.complete(note.id, !note.completed))}>{note.completed ? <Check size={14} /> : <Circle size={19} />}</button>
          <span className="note-time">{note.date < selected ? '进行中' : note.time ?? '全天'}</span><button className="note-body" onClick={() => openEditor(note.date,false,undefined,note)}><strong>{note.title}</strong>{note.endDate && note.endDate !== note.date && <span className="note-range">{note.date} → {note.endDate}</span>}{note.content && <span>{note.content}</span>}</button><span className={`category-tag ${categoryClass[note.category]}`}>{note.category}</span>{note.priority === 3 && <span className="priority-flag" title="高优先级">!</span>}<div className="note-actions"><button aria-label={`编辑 ${note.title}`} className="icon-button" onClick={() => openEditor(note.date,false,undefined,note)}><Pencil size={14} /></button><button aria-label={`删除 ${note.title}`} className="icon-button" disabled={busy} onClick={() => setDeleting(note)}><Trash2 size={14} /></button></div>
        </article>) : <div className="daily-empty"><CalendarDays size={27} strokeWidth={1.2} /><div><strong>{loading ? '正在读取日签…' : '这一天，还没有安排'}</strong><span>一件待办，一个想法，都可以从这里开始。</span></div></div>}</div>
        <button className="add-note-link" onClick={() => openEditor(selected)}><Plus size={16} />添加日签<span>Ctrl N</span></button>
      </section>
    </section><aside className="upcoming-panel" aria-label="Upcoming"><div className="upcoming-heading"><div><p className="eyebrow">WHAT’S NEXT</p><h2>Upcoming<span>{upcoming.length.toString().padStart(2,'0')}</span></h2></div><ArrowUpRight size={21} strokeWidth={1.4} /></div><p className="upcoming-caption">接下来，慢慢来。</p>
      <div className="upcoming-list">{upcoming.length ? upcoming.map((note,i) => <button key={note.id} className="upcoming-item" onClick={() => select(note.date < today ? today : note.date)}><span className="upcoming-number">{(i+1).toString().padStart(2,'0')}</span><span className="upcoming-content"><span className="upcoming-date">{relative(note.date < today ? today : note.date)}<span><Clock3 size={11} />{note.date < today ? '进行中' : note.time ?? '全天'}</span></span><strong>{note.title}</strong>{note.endDate && note.endDate !== note.date && <small className="upcoming-range">{note.date} → {note.endDate}</small>}<span className="upcoming-meta"><i className={`dot ${categoryClass[note.category]}`} />{note.category}{note.priority === 3 && <span>高优先级</span>}</span></span></button>) : <div className="upcoming-empty"><Sun size={38} strokeWidth={1} /><strong>暂时没有待办</strong><span>未来的安排会出现在这里。<br />现在，享受一点从容。</span></div>}</div>
      <div className="quiet-card"><span className="quiet-mark">“</span><p>日子不必填满，<br />重要的事，记得就好。</p><span className="quiet-line" /><small>ONE DAY AT A TIME</small></div>
    </aside></div>
    <footer className="statusbar"><span><ShieldCheck size={13} />{native ? '本地 SQLite · 自动备份' : '预览数据仅保存在当前浏览器'}</span><span>CALENDAR DESKTOP SECRETARY<span className="footer-separator">/</span>{format(now, 'yyyy')}</span></footer>
    {editor && <NoteEditor key={`${editor.note?.id ?? 'new'}-${editor.date}`} target={editor} close={() => setEditor(null)} save={save} />}
    {settingsOpen && <SettingsPanel settings={settings} close={() => setSettingsOpen(false)} refresh={refresh} saved={async s => { await api.saveSettings(s); setSettings(s); }} />}
    {deleting && <Dialog label="删除日签" className="delete-dialog" close={() => { if (!busy) setDeleting(null); }}><p className="eyebrow">REMOVE DAILY NOTE</p><h2>删除这条日签？</h2><p>「{deleting.title}」将从日历中移除。</p><div className="dialog-footer"><span /><div><button disabled={busy} className="secondary-button" onClick={() => setDeleting(null)}>取消</button><button disabled={busy} className="danger-button" onClick={() => mutate(async () => { await api.remove(deleting.id); setDeleting(null); })}>确认删除</button></div></div></Dialog>}
  </main>;
}
