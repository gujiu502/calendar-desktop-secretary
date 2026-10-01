import { useRef, useState } from 'react';
import type { FormEvent } from 'react';
import { ArrowUpRight, Check, X } from 'lucide-react';
import { categories } from './calendar';
import type { DailyNote, NoteInput } from './calendar';
import Dialog from './Dialog';

export interface EditorTarget { date: string; time?: string; note?: DailyNote; quick?: boolean }
export default function NoteEditor({ target, close, save }: { target: EditorTarget; close: () => void; save: (input: NoteInput) => Promise<void> }) {
  const [expanded, setExpanded] = useState(!target.quick || !!target.note);
  const [input, setInput] = useState<NoteInput>(target.note ? { id: target.note.id, date: target.note.date, time: target.note.time, title: target.note.title, content: target.note.content, category: target.note.category, priority: target.note.priority } : { date: target.date, time: target.time ?? null, title: '', content: '', category: '普通', priority: 0 });
  const [busy, setBusy] = useState(false), [error, setError] = useState('');
  const submitting = useRef(false);
  const patch = (value: Partial<NoteInput>) => setInput(previous => ({ ...previous, ...value }));
  const submit = async (e: FormEvent) => {
    e.preventDefault(); if (submitting.current) return;
    submitting.current = true; setBusy(true); setError('');
    try { await save(input); } catch (e) { setError(String(e instanceof Error ? e.message : e)); } finally { submitting.current = false; setBusy(false); }
  };
  return <Dialog label={target.note ? '编辑日签' : expanded ? '新建日签' : '快速添加日签'} close={() => { if (!busy) close(); }} className={expanded ? 'editor' : 'editor quick-editor'}>
    <form onSubmit={submit} onKeyDown={e => { if (e.ctrlKey && e.key === 'Enter') { e.preventDefault(); e.currentTarget.requestSubmit(); } }}>
      <div className="dialog-heading"><div><p className="eyebrow">DAILY NOTE</p><h2>{target.note ? '编辑日签' : expanded ? '新建日签' : '记下一件事'}</h2></div><button className="icon-button" type="button" aria-label="关闭编辑器" onClick={close} disabled={busy}><X size={20} /></button></div>
      <div className="form-row"><label>日期<input type="date" min="1900-01-01" max="9999-12-31" required value={input.date} onChange={e => patch({ date: e.target.value })} /></label><label>时间 <span>可选</span><input aria-label="时间" type="time" value={input.time ?? ''} onChange={e => patch({ time: e.target.value || null })} /></label></div>
      <label>标题<input autoFocus placeholder="有什么值得记住？" maxLength={200} required value={input.title} onChange={e => patch({ title: e.target.value })} /></label>
      {expanded && <><label>内容 <span>可选</span><textarea placeholder="补充细节、想法，或下一步……" rows={4} value={input.content} onChange={e => patch({ content: e.target.value })} /></label><div className="form-row"><label>分类<select aria-label="分类" value={input.category} onChange={e => patch({ category: e.target.value as NoteInput['category'] })}>{categories.map(c => <option key={c}>{c}</option>)}</select></label><label>优先级<select aria-label="优先级" value={input.priority} onChange={e => patch({ priority: Number(e.target.value) })}><option value={0}>普通</option><option value={1}>低</option><option value={2}>中</option><option value={3}>高</option></select></label></div></>}
      {error && <p role="alert" className="form-error">{error}</p>}
      <div className="dialog-footer">{!expanded ? <button className="text-button" type="button" onClick={() => setExpanded(true)}>更多细节<ArrowUpRight size={14} /></button> : <span className="keyboard-hint">Ctrl + Enter 保存</span>}<div><button className="secondary-button" type="button" onClick={close} disabled={busy}>取消</button><button className="primary-button" disabled={busy}><Check size={15} />{busy ? '保存中…' : '保存日签'}</button></div></div>
    </form>
  </Dialog>;
}
