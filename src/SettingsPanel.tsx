import { useEffect, useState } from 'react';
import { Download, FolderOpen, Monitor as MonitorIcon, Save, ShieldCheck, Upload, X } from 'lucide-react';
import Dialog from './Dialog';
import { api, native, onNativeEvent } from './services';
import type { Monitor, Settings } from './services';

export default function SettingsPanel({ settings, close, saved, refresh }: { settings: Settings; close: () => void; saved: (settings: Settings) => Promise<void>; refresh: () => Promise<void> }) {
  const [draft, setDraft] = useState(settings), [tab, setTab] = useState('通用');
  const [monitors, setMonitors] = useState<Monitor[]>([]), [message, setMessage] = useState(''), [error, setError] = useState(''), [busy, setBusy] = useState(false);
  const patch = (s: Partial<Settings>) => setDraft(old => ({ ...old, ...s }));
  useEffect(() => {
    let active = true, unlisten = () => {};
    const load = () => { api.monitors().then(m => { if (active) setMonitors(m); }).catch(e => { if (active) setError(String(e)); }); };
    load(); void onNativeEvent('monitors-changed', load).then(fn => { if (active) unlisten = fn; else fn(); });
    return () => { active = false; unlisten(); };
  }, []);
  const run = async (fn: () => Promise<unknown>, success: string) => {
    setBusy(true); setError(''); setMessage('');
    try { const result = await fn(); if (result !== null) setMessage(typeof result === 'string' ? `${success}：${result}` : typeof result === 'number' ? `已导入 ${result} 条新日签` : success); } catch (e) { setError(String(e instanceof Error ? e.message : e)); } finally { setBusy(false); }
  };
  const toggle = (key: keyof Settings, label: string, hint?: string, desktopOnly = false) => <label className="setting-toggle"><div><span>{label}</span>{hint && <small>{hint}</small>}</div><input type="checkbox" disabled={desktopOnly && !native} checked={Boolean(draft[key])} onChange={e => patch({ [key]: e.target.checked })} /></label>;
  return <Dialog label="设置" close={() => { if (!busy) close(); }} className="settings-dialog">
    <div className="dialog-heading"><div><p className="eyebrow">MAKE IT YOURS</p><h2>设置</h2></div><button className="icon-button" aria-label="关闭设置" onClick={close} disabled={busy}><X size={20} /></button></div>
    <div className="settings-layout"><nav aria-label="设置分类">{['通用','显示器','外观','日历','数据'].map(t => <button key={t} className={tab === t ? 'active' : ''} onClick={() => { setTab(t); setMessage(''); setError(''); }}>{t}</button>)}</nav><div className="settings-content">
      {tab === '通用' && <><h3>留在桌面，随时可用</h3>{toggle('autostart','开机自动启动','登录 Windows 后启动',true)}{toggle('closeToTray','关闭窗口时隐藏到托盘','可从托盘菜单退出应用',true)}{toggle('showOnStart','启动后显示窗口',undefined,true)}<div className="settings-tip">快捷键 <kbd>Ctrl N</kbd> 新建日签 · <kbd>Esc</kbd> 关闭弹窗</div>{!native && <p className="settings-tip">当前为浏览器预览。窗口、托盘和自启动设置在桌面应用中可用。</p>}</>}
      {tab === '显示器' && <><h3>你的第二屏日历</h3><label>默认显示位置<select disabled={!native} value={draft.preferredDisplay} onChange={e => patch({ preferredDisplay: e.target.value as Settings['preferredDisplay'] })}><option value="display2">Windows 显示器 2（首选）</option><option value="primary">主显示器</option><option value="last">上次使用的显示器</option><option value="manual">手动选择</option></select></label>{draft.preferredDisplay === 'manual' && <label>显示器<select value={draft.manualMonitor ?? ''} onChange={e => patch({ manualMonitor: e.target.value })}><option value="">请选择</option>{monitors.map(m => <option key={m.id} value={m.id}>显示器 {m.number ?? '?'} · {m.name}</option>)}</select></label>}{toggle('autoReturn','首选显示器连接后自动移回',undefined,true)}{toggle('rememberPosition','记住每块屏幕的窗口位置',undefined,true)}{toggle('rememberSize','记住窗口尺寸',undefined,true)}<div className="monitor-list">{monitors.map(m => <div key={m.id}><MonitorIcon size={18} /><span>显示器 {m.number ?? '?'} · {m.name}<small>{m.width} × {m.height} · {Math.round(m.scale * 100)}%{m.primary ? ' · 主屏幕' : ''}</small></span></div>)}</div><p className="settings-tip">首选屏幕不可用时，自动回到主屏幕。</p></>}
      {tab === '外观' && <><h3>一层恰到好处的玻璃</h3><label>背景不透明度 <span>{Math.round(draft.opacity * 100)}%</span><input type="range" min="35" max="100" value={Math.round(draft.opacity * 100)} onChange={e => patch({ opacity: Number(e.target.value) / 100 })} /></label><label>窗口效果<select value={draft.effect} onChange={e => patch({ effect: e.target.value as Settings['effect'] })}><option value="acrylic">Acrylic · 毛玻璃</option><option value="mica">Mica · 云母</option><option value="blur">Blur · 模糊</option><option value="none">无</option></select></label><label>圆角 <span>{draft.cornerRadius}px</span><input type="range" min="0" max="28" value={draft.cornerRadius} onChange={e => patch({ cornerRadius: Number(e.target.value) })} /></label><p className="settings-tip">Mica 在不支持的 Windows 版本上自动尝试 Acrylic / Blur。文字保持清晰。</p></>}
      {tab === '日历' && <><h3>按你的习惯安排</h3><label>每周开始于<select value={draft.weekStart} onChange={e => patch({ weekStart: Number(e.target.value) as 0 | 1 })}><option value={1}>星期一</option><option value={0}>星期日</option></select></label><label>Upcoming 显示数量<select value={draft.upcomingCount} onChange={e => patch({ upcomingCount: Number(e.target.value) as 5 | 8 | 10 })}><option value={5}>5 条</option><option value={8}>8 条</option><option value={10}>10 条</option></select></label><p className="settings-tip">只显示未来未完成日签。当天无时间的日签排在最后。</p></>}
      {tab === '数据' && <><h3>只在你的电脑里</h3><p className="settings-tip">无需账号，无需联网。桌面应用使用本地 SQLite；自动每日备份一次，保留最近 30 份。</p><div className="data-actions"><button disabled={busy} onClick={() => run(() => api.export('json'),'JSON 已导出')}><Download size={17} />导出 JSON</button><button disabled={busy} onClick={() => run(() => api.export('csv'),'CSV 已导出')}><Download size={17} />导出 CSV</button><button disabled={busy || !native} onClick={() => run(async () => { const count = await api.import(); if (count !== null) await refresh(); return count; },'导入完成')}><Upload size={17} />导入 JSON</button><button disabled={busy || !native} onClick={() => run(() => api.backup(),'今日备份已就绪')}><ShieldCheck size={17} />备份数据库</button><button disabled={busy || !native} onClick={() => run(() => api.openFolder(),'已打开数据库目录')}><FolderOpen size={17} />打开数据目录</button></div><p className="settings-tip">导入会合并新日签，保留已有 ID 的记录。浏览器预览数据与桌面数据库独立。</p></>}
    </div></div>
    {message && <p className="form-success" role="status">{message}</p>}{error && <p className="form-error" role="alert">{error}</p>}
    <div className="dialog-footer"><span className="keyboard-hint">CALENDAR DESKTOP SECRETARY · V1</span><div><button className="secondary-button" onClick={close} disabled={busy}>取消</button><button className="primary-button" disabled={busy} onClick={() => run(async () => { await saved(draft); close(); },'设置已保存')}><Save size={15} />保存设置</button></div></div>
  </Dialog>;
}
