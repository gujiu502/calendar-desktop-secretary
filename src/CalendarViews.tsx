import { format, isSameMonth, parseISO, startOfYear, addMonths } from 'date-fns';
import { Plus } from 'lucide-react';
import { categories, categoryClass, dateKey, dateTint, miniDays, monthDays } from './calendar';
import type { DailyNote, View } from './calendar';

interface Props {
  anchor: Date; selected: string; today: string; view: View; weekStart: 0 | 1;
  grouped: Map<string, DailyNote[]>;
  select: (date: string, quick?: boolean, time?: string) => void;
  month: (date: Date) => void; edit: (note: DailyNote) => void;
}
const weekdays = (weekStart: 0 | 1) => weekStart ? ['一','二','三','四','五','六','日'] : ['日','一','二','三','四','五','六'];
function Dots({ notes }: { notes: DailyNote[] }) {
  return <span className="dots" aria-hidden="true">{categories.filter(c => notes.some(n => !n.completed && n.category === c)).map(c => <i key={c} className={categoryClass[c]} />)}{notes.length > 0 && notes.every(n => n.completed) && <i className="gray" />}</span>;
}
export default function CalendarViews(p: Props) {
  if (p.view === 'year') return <div className="year-grid" aria-label="年视图">{Array.from({ length: 12 }, (_, i) => {
    const month = addMonths(startOfYear(p.anchor), i);
    return <section className="mini-month" key={i}>
      <button className="mini-title" onClick={() => p.month(month)}>{i + 1}<span>月</span></button>
      <div className="mini-grid">{weekdays(p.weekStart).map((d, i) => <span className="mini-week" key={i}>{d}</span>)}
        {miniDays(month, p.weekStart).map(day => { const key = dateKey(day), notes = p.grouped.get(key) ?? []; return isSameMonth(day, month) ? <button key={key} aria-label={key} onClick={() => p.select(key)} className={`${dateTint(notes)} ${key === p.today ? 'mini-today' : ''} ${key === p.selected ? 'mini-selected' : ''}`}><span>{day.getDate()}</span><Dots notes={notes} /></button> : <span key={key} />; })}
      </div>
    </section>;
  })}</div>;
  if (p.view === 'day') {
    const notes = p.grouped.get(dateKey(p.anchor)) ?? [];
    return <div className="day-timeline" aria-label="日视图">
      <div className="all-day"><span>全天</span><div>{notes.filter(n => !n.time || n.date < dateKey(p.anchor)).map(n => <button key={n.id} className={`timeline-note ${categoryClass[n.category]} ${n.completed ? 'completed' : ''}`} onClick={() => p.edit(n)}>{n.title}</button>)}<button className="timeline-add" aria-label="添加全天日签" onClick={() => p.select(dateKey(p.anchor), true)}><Plus size={14} /></button></div></div>
      {Array.from({ length: 24 }, (_, h) => <div className="hour-row" key={h}><span>{String(h).padStart(2, '0')}:00</span><div>{notes.filter(n => n.date === dateKey(p.anchor) && n.time && Number(n.time.slice(0, 2)) === h).map(n => <button key={n.id} className={`timeline-note ${categoryClass[n.category]} ${n.completed ? 'completed' : ''}`} onClick={() => p.edit(n)}><b>{n.time}</b> {n.title}</button>)}<button className="timeline-add" aria-label={`添加 ${h}:00 日签`} onClick={() => p.select(dateKey(p.anchor), true, `${String(h).padStart(2,'0')}:00`)}><Plus size={14} /></button></div></div>)}
    </div>;
  }
  return <div className="month-view" aria-label="月视图"><div className="weekdays">{weekdays(p.weekStart).map((day, i) => <span className={i > 4 ? 'weekend' : ''} key={day}>{day}</span>)}</div><div className="month-grid">
    {monthDays(p.anchor, p.weekStart).map(day => {
      const key = dateKey(day), notes = p.grouped.get(key) ?? [];
      return <button aria-label={key} disabled={day.getFullYear() < 1900 || day.getFullYear() > 9999} aria-pressed={key === p.selected} onClick={() => p.select(key, true)} key={key} className={`calendar-cell ${!isSameMonth(day, p.anchor) ? 'outside' : ''} ${key === p.today ? 'today' : ''} ${key === p.selected ? 'selected' : ''} ${dateTint(notes)}`}>
        <span className="day-number">{format(day, 'd')}</span>{key === p.today && <span className="today-label">今天</span>}<Dots notes={notes} />{notes.length > 0 && <span className="cell-count">{notes.length}</span>}
      </button>;
    })}
  </div><div className="calendar-legend">{categories.map(c => <span key={c}><i className={`dot ${categoryClass[c]}`} />{c}</span>)}<span className="legend-hint">点击日期，记下一件事</span></div></div>;
}

export const weekdayLabel = (key: string) => format(parseISO(key), 'EEEE', { });
