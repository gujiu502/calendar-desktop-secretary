import test from 'node:test';
import assert from 'node:assert/strict';
import { dateKey, groupNotes, monthDays, navigate, upcomingFrom, validateNote, visibleRange } from '../src/calendar.ts';
import type { DailyNote } from '../src/calendar.ts';

const note = (id: string, date: string, time: string | null, priority = 0, completed = false): DailyNote => ({ id, date, time, priority, completed, title: '作业', content: '', category: '学习', createdAt: '2026-10-01T00:00:00Z', updatedAt: '2026-10-01T00:00:00Z' });
test('month starts on chosen weekday, spans 42 dates, handles leap years and month rollover', () => {
  const october = new Date(2026, 9, 1);
  assert.equal(dateKey(monthDays(october, 1)[0]), '2026-09-28');
  assert.equal(dateKey(monthDays(october, 0)[0]), '2026-09-27');
  assert.equal(monthDays(october, 1).length, 42);
  assert.ok(monthDays(new Date(2028, 1, 1), 1).some(d => dateKey(d) === '2028-02-29'));
  assert.equal(dateKey(navigate(new Date(2026, 0, 31), 'month', 1)), '2026-02-01');
  assert.deepEqual(visibleRange(october, 'year', 1), ['2026-01-01', '2026-12-31']);
  assert.equal(visibleRange(new Date(1900,0,1),'month',0)[0],'1900-01-01');
  assert.equal(visibleRange(new Date(9999,11,1),'month',1)[1],'9999-12-31');
});
test('Upcoming filters past and completed notes; untimed notes sort last with priority tie break', () => {
  const notes = [note('past', '2026-09-30', null), note('elapsed','2026-10-01','09:00'), note('all-day','2026-10-01',null), note('later','2026-10-01','20:00'), note('low','2026-10-02','09:00',1), note('high','2026-10-02','09:00',3), note('done','2026-10-02','08:00',3,true)];
  assert.deepEqual(upcomingFrom(notes,new Date(2026,9,1,16),8).map(n=>n.id), ['later','all-day','high','low']);
  assert.equal(upcomingFrom(notes,new Date(2026,9,1,16),2).length,2);
  assert.equal(groupNotes(notes).get('2026-10-01')?.length,3);
  assert.deepEqual(upcomingFrom([],new Date(),8),[]);
});
test('shared editor validation rejects impossible dates, blank titles and invalid fields', () => {
  const base = { date:'2026-10-01',time:null,title:'作业',content:'',category:'学习' as const,priority:0 };
  assert.doesNotThrow(()=>validateNote(base));
  assert.doesNotThrow(()=>validateNote({...base,date:'2028-02-29',time:'23:59'}));
  for (const input of [{...base,date:'2026-02-29'},{...base,date:'2026-04-31'},{...base,time:'24:00'},{...base,title:'   '},{...base,priority:4}]) assert.throws(()=>validateNote(input));
});
