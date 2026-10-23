/* Real local service/browser check. Hardware and external brokers are never opened. */
const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const net = require('node:net');
const {spawn} = require('node:child_process');
const {chromium} = require(process.env.PLAYWRIGHT_MODULE || 'playwright-core');
const root = path.resolve(__dirname, '..');
const output = process.env.CAN_UI_SCREENSHOTS || '/tmp/can-to-mqtt-ui';
const delay = ms => new Promise(resolve => setTimeout(resolve, ms));
(async () => {
  const port = await new Promise(resolve => {const server = net.createServer(); server.listen(0,'127.0.0.1',()=>{const p = server.address().port;server.close(()=>resolve(p));});});
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'can-ui-'));
  const token = 'a'.repeat(64);
  const draft = path.join(dir, "private draft's config.json");
  const config = path.join(dir, 'settings.json');
  fs.writeFileSync(config, JSON.stringify({http_port:port,http_token:token,http_token_file:path.join(dir,'token'),config_draft_path:draft,prometheus_enabled:false,car:'test-car',mqtt_password:'browser-secret-fixture',mqtt_port:1}));
  const child = spawn(process.env.CAN_BINARY || path.join(root,'target/debug/can-to-mqtt'), ['--config',config,'--web-only'], {stdio:['ignore','pipe','pipe']});
  let log = '';child.stdout.on('data',chunk=>log+=chunk);child.stderr.on('data',chunk=>log+=chunk);
  let browser;
  try {
    const url = `http://127.0.0.1:${port}/`;
    let connected = false;
    for(let n=0;n<100;n++) {try {if((await fetch(url)).ok){connected=true;break;}} catch {} await delay(50);}
    assert.equal(connected,true,log);
    assert.equal((await fetch(url+'api/v1/config')).status,401);
    browser = await chromium.launch({headless:true,...(process.env.CHROME_PATH ? {executablePath:process.env.CHROME_PATH}:{})});
    const page = await browser.newPage({viewport:{width:1440,height:1000}});
    const errors=[];page.on('pageerror',err=>errors.push(err.message));
    await page.goto(url);
    await page.locator('#token').fill(token);
    await page.locator('#login button').click();
    await page.locator('#product').waitFor({state:'visible'});
    assert.equal(await page.locator('#car').inputValue(),'test-car');
    assert.equal(await page.locator('#mqtt_password').inputValue(),'');
    assert.equal((await page.locator('body').textContent()).includes('browser-secret-fixture'),false);
    await page.locator('[data-step="1"]').click();
    await page.locator('#car').fill('edited-car');
    await delay(700);
    assert.equal(await page.locator('#car').inputValue(),'edited-car','Polling erased an unsaved edit');
    await page.locator('#settings [type=submit]').click();
    await page.waitForFunction(()=>document.querySelector('#saved').textContent.includes('Draft saved:'));
    assert.equal(JSON.parse(fs.readFileSync(draft,'utf8')).car,'edited-car');
    assert.equal(fs.statSync(draft).mode & 0o777,0o600);
    assert.match(await page.locator('#saved').textContent(),/private draft/);
    const running = await (await fetch(url+'api/v1/config',{headers:{Authorization:`Bearer ${token}`}})).json();
    assert.equal(running.car,'test-car','Saving changed acquisition without restart');
    assert.equal(running.mqtt_password,'<redacted>');
    const state={version:'0.4.0',can_connected:true,mqtt_connected:true,ready:true,mode:'obd',frames:12,published:3,dropped:0,ecu:{id:2024,extended:false},supported_pids:[12,13],channels:[{name:'rpm',value:2000,ageMs:17,source:'obd2/mode01'},{name:'ecuSpeedKmh',value:0,ageMs:30,source:'obd2/mode01'}]};
    await page.route('**/api/v1/status',route=>route.fulfill({contentType:'application/json',body:JSON.stringify(state)}));
    await page.locator('[data-step="2"]').click();
    await page.waitForFunction(()=>document.querySelector('#result').textContent.includes('verified on this computer'));
    assert.equal(await page.locator('#channels tr').count(),2);
    assert.equal(await page.locator('#channels tr').nth(1).locator('td').nth(1).textContent(),'0');
    fs.mkdirSync(output,{recursive:true});
    for(const theme of ['dark','light']) {
      await page.evaluate(theme=>document.documentElement.dataset.theme=theme,theme);
      for(const width of [320,390,768,1440]) {
        await page.setViewportSize({width,height:900});
        for(const step of [0,1,2]) {
          await page.locator(`[data-step="${step}"]`).click();
          assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),true,`Overflow at ${width}, step ${step}`);
        }
        await page.screenshot({path:path.join(output,`${theme}-${width}-live.png`),fullPage:true});
      }
    }
    state.ready=false;state.can_connected=false;state.mqtt_connected=false;state.channels=[];state.error='CAN interface unavailable';
    await page.waitForFunction(()=>document.querySelector('#result').textContent==='CAN interface unavailable');
    assert.equal(await page.locator('#channels tr').count(),0);
    assert.equal(await page.locator('#empty').isVisible(),true);
    assert.equal(await page.locator('#result').evaluate(el=>el.classList.contains('good')),false);
    assert.deepEqual(errors,[]);
    console.log('PASS: real process auth/redaction, 0600 draft save, preserved unsaved edits, fresh/zero/missing data, safe offline status, 3 steps at 4 widths in 2 themes. Screenshots: '+output);
  } finally {
    if(browser) await browser.close();
    child.kill('SIGTERM');await Promise.race([new Promise(resolve=>child.once('exit',resolve)),delay(1500)]);if(child.exitCode===null)child.kill('SIGKILL');
    fs.rmSync(dir,{recursive:true,force:true});
  }
})().catch(error=>{console.error(error);process.exitCode=1;});
