import { chromium } from 'playwright';
import { spawn } from 'node:child_process';
import assert from 'node:assert/strict';
import { readFile, writeFile } from 'node:fs/promises';

// Launch the release with WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9223.
// Only loopback debugging is used for this test; restart without that env var afterwards.
const browser=await chromium.connectOverCDP('http://127.0.0.1:9223');
const page=browser.contexts().flatMap(c=>c.pages()).find(p=>p.url().includes('tauri.localhost'));
assert.ok(page,'Native Tauri WebView2 page must exist');
const errors=[];page.on('pageerror',e=>errors.push(e.message));
const invoke=(command,args={})=>page.evaluate(({command,args})=>window.__TAURI_INTERNALS__.invoke(command,args),{command,args});
const ids=[];
try {
  await page.getByRole('button',{name:'新建日签',exact:true}).waitFor();
  assert.equal(await page.getByText('浏览器预览',{exact:true}).count(),0);
  const monitors=await invoke('get_monitors');
  assert.ok(monitors.length>=1 && monitors.some(m=>m.primary));
  const startup=JSON.parse((await readFile('work/native-startup.json','utf8')).replace(/^\ufeff/,''));
  const settings=await invoke('get_settings');
  if (settings.preferredDisplay==='display2') {
    const target=monitors.find(m=>m.number===2)??monitors.find(m=>m.primary);
    const rect=startup.firstVisible;
    assert.ok(rect.x+rect.width/2>=target.x && rect.x+rect.width/2<target.x+target.width && rect.y+rect.height/2>=target.y && rect.y+rect.height/2<target.y+target.height,'First visible frame is on the preferred monitor');
  }
  const date=await page.evaluate(()=>{const d=new Date();return `${d.getFullYear()}-${String(d.getMonth()+1).padStart(2,'0')}-${String(d.getDate()).padStart(2,'0')}`;});
  const input={id:null,date,time:null,title:`原生验收-${Date.now()}`,content:'SQLite 持久化检查',category:'学习',priority:3};
  await assert.rejects(()=>invoke('save_note',{input:{...input,date:'2026-02-30'}}));
  const id=await invoke('save_note',{input});ids.push(id);
  let rows=await invoke('list_notes',{start:date,end:date});
  assert.ok(rows.some(n=>n.id===id && !n.completed && n.time===null));
  await invoke('complete_note',{id,completed:true});
  assert.ok(!(await invoke('upcoming_notes',{limit:10})).some(n=>n.id===id));
  assert.equal((await invoke('list_notes',{start:date,end:date})).find(n=>n.id===id).completed,true);
  await invoke('save_note',{input:{...input,id,title:'已编辑的原生日签'}});
  await invoke('complete_note',{id,completed:false});
  await page.reload();
  await page.locator('.daily-list').getByText('已编辑的原生日签',{exact:true}).waitFor();
  const backup=await invoke('backup_data');assert.ok(backup.endsWith('.sqlite'));
  await page.getByRole('button',{name:'新建日签',exact:true}).click();
  assert.equal(await page.getByLabel('日期',{exact:true}).inputValue(),date);
  await page.getByLabel('标题',{exact:true}).fill('UI 到 Rust 到 SQLite');
  await page.getByRole('button',{name:'保存日签',exact:true}).click();
  await page.getByRole('dialog').waitFor({state:'hidden'});
  rows=await invoke('list_notes',{start:date,end:date});
  const created=rows.find(n=>n.title==='UI 到 Rust 到 SQLite');assert.ok(created);ids.push(created.id);
  await page.screenshot({path:'work/screenshots/native.png'});
  const geometry=await page.evaluate(()=>({width:innerWidth,height:innerHeight,scrollHeight:document.documentElement.scrollHeight,scrollWidth:document.documentElement.scrollWidth,devicePixelRatio,screenX,screenY}));
  assert.equal(geometry.scrollWidth,geometry.width);
  await page.getByRole('button',{name:'关闭窗口',exact:true}).click();
  await page.waitForFunction(async()=>!(await window.__TAURI_INTERNALS__.invoke('plugin:window|is_visible',{label:'main'})));
  const executable=process.argv[2];
  if (executable) {
    const second=spawn(executable,[],{windowsHide:true,stdio:'ignore'});
    await new Promise((resolve,reject)=>{second.once('error',reject);second.once('exit',resolve);});
    await page.waitForFunction(()=>window.__TAURI_INTERNALS__.invoke('plugin:window|is_visible',{label:'main'}));
  }
  assert.deepEqual(errors,[]);
  const report={result:'PASS',monitors,geometry,backup,checks:['native IPC validation','SQLite CRUD','reload persistence','Upcoming completion filter','UI→Rust→SQLite','online backup','close to tray',...(executable?['single-instance reopen']:[])]};
  await writeFile('work/native-smoke.json',JSON.stringify(report,null,2));
  console.log(JSON.stringify(report));
} finally {
  for (const id of ids) await invoke('delete_note',{id}).catch(()=>{});
  await page.reload().catch(()=>{});
  await browser.close();
}
