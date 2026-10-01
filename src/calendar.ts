import { addDays, addMonths, addYears, eachDayOfInterval, endOfMonth, endOfWeek, endOfYear, format, isValid, parseISO, startOfMonth, startOfWeek, startOfYear } from 'date-fns';

export type View = 'day' | 'month' | 'year';
export type Category = '普通' | '重要' | '学习' | '生活';
export interface DailyNote {
  id: string; date: string; endDate: string | null; time: string | null; title: string; content: string;
  category: Category; priority: number; completed: boolean; createdAt: string; updatedAt: string;
}
export type NoteInput = Pick<DailyNote, 'date' | 'endDate' | 'time' | 'title' | 'content' | 'category' | 'priority'> & { id?: string };
export const categories: Category[] = ['普通', '重要', '学习', '生活'];
export const categoryClass: Record<Category, string> = { '普通': 'blue', '重要': 'red', '学习': 'purple', '生活': 'green' };
export function dateTint(notes: DailyNote[]) {
  const active = notes.filter(n => !n.completed);
  const note = active.find(n => n.priority === 3 || n.category === '重要') ?? active[0];
  return notes.length ? `has-notes tint-${note ? categoryClass[note.category] : 'gray'}` : '';
}
export const dateKey = (date: Date) => format(date, 'yyyy-MM-dd');
export function monthDays(date: Date, weekStart: 0 | 1) {
  const start = startOfWeek(startOfMonth(date), { weekStartsOn: weekStart });
  return eachDayOfInterval({ start, end: addDays(start, 41) });
}
export function visibleRange(date: Date, view: View, weekStart: 0 | 1) {
  if (view === 'year') return [dateKey(startOfYear(date)), dateKey(endOfYear(date))];
  if (view === 'day') return [dateKey(date), dateKey(date)];
  const days = monthDays(date, weekStart);
  return [dateKey(days[0]) < '1900-01-01' ? '1900-01-01' : dateKey(days[0]), days[41].getFullYear() > 9999 ? '9999-12-31' : dateKey(days[41])];
}
export function navigate(date: Date, view: View, amount: number) {
  return view === 'year' ? addYears(date, amount) : view === 'month' ? addMonths(startOfMonth(date), amount) : addDays(date, amount);
}
export function validateNote(note: NoteInput) {
  for (const value of [note.date, ...(note.endDate == null ? [] : [note.endDate])]) {
    const parsed = parseISO(value);
    if (!/^\d{4}-\d{2}-\d{2}$/.test(value) || !isValid(parsed) || dateKey(parsed) !== value || value < '1900-01-01' || value > '9999-12-31') throw new Error('请输入有效日期（1900–9999 年）');
  }
  if (note.endDate && note.endDate < note.date) throw new Error('结束日期不能早于开始日期');
  if (note.time && !/^([01]\d|2[0-3]):[0-5]\d$/.test(note.time)) throw new Error('请输入有效时间');
  if (!note.title.trim() || [...note.title].length > 200) throw new Error('标题需要 1–200 个字符');
  if (new TextEncoder().encode(note.content).length > 50_000) throw new Error('内容过长');
  if (!categories.includes(note.category) || !Number.isInteger(note.priority) || note.priority < 0 || note.priority > 3) throw new Error('分类或优先级无效');
}
export function upcomingFrom(notes: DailyNote[], now: Date, limit: number) {
  const today = dateKey(now), time = format(now, 'HH:mm');
  const nextDate = (n: DailyNote) => n.date < today ? today : n.date;
  const nextTime = (n: DailyNote) => n.date < today ? '23:59' : n.time ?? '23:59';
  return notes.filter(n => !n.completed && ((n.endDate ?? n.date) > today || ((n.endDate ?? n.date) === today && (n.date < today || !n.time || n.time >= time))))
    .sort((a, b) => nextDate(a).localeCompare(nextDate(b)) || nextTime(a).localeCompare(nextTime(b)) || b.priority - a.priority || a.createdAt.localeCompare(b.createdAt)).slice(0, limit);
}
export function groupNotes(notes: DailyNote[], start: string, end: string) {
  const grouped = new Map<string, DailyNote[]>();
  for (const note of notes) {
    const first = note.date > start ? note.date : start, last = (note.endDate ?? note.date) < end ? note.endDate ?? note.date : end;
    if (first > last) continue;
    for (const day of eachDayOfInterval({ start: parseISO(first), end: parseISO(last) })) {
      const key = dateKey(day), group = grouped.get(key) ?? []; group.push(note); grouped.set(key, group);
    }
  }
  return grouped;
}
export function miniDays(date: Date, weekStart: 0 | 1) {
  return eachDayOfInterval({ start: startOfWeek(startOfMonth(date), { weekStartsOn: weekStart }), end: endOfWeek(endOfMonth(date), { weekStartsOn: weekStart }) });
}
