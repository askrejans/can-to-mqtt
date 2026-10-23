/* Actual local Mosquitto TLS check with throwaway certificates; no CAN sockets. */
'use strict';
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const os = require('node:os');
const net = require('node:net');
const {spawn,execFileSync} = require('node:child_process');
const delay = ms => new Promise(resolve=>setTimeout(resolve,ms));
const root = path.resolve(__dirname,'..');
async function port() {return new Promise(resolve=>{const s=net.createServer();s.listen(0,'127.0.0.1',()=>{const p=s.address().port;s.close(()=>resolve(p));});});}
async function stop(child) {if(!child || child.exitCode!==null)return;child.kill('SIGTERM');await Promise.race([new Promise(resolve=>child.once('exit',resolve)),delay(1500)]);if(child.exitCode===null)child.kill('SIGKILL');}
(async()=>{
  const dir = fs.mkdtempSync(path.join(os.tmpdir(),'can-mqtt-tls-'));
  const token='b'.repeat(64),brokerPort=await port(),apiPort=await port();
  let broker,bridge;
  try {
    execFileSync('openssl',['req','-x509','-newkey','rsa:2048','-sha256','-days','1','-nodes','-subj','/CN=localhost','-addext','subjectAltName=DNS:localhost','-keyout',path.join(dir,'key.pem'),'-out',path.join(dir,'ca.pem')],{stdio:'ignore'});
    const brokerConfig=path.join(dir,'mosquitto.conf');
    fs.writeFileSync(brokerConfig,`listener ${brokerPort} 127.0.0.1\nallow_anonymous true\ncertfile ${dir}/ca.pem\nkeyfile ${dir}/key.pem\n`);
    broker=spawn(process.env.MOSQUITTO_BIN||'mosquitto',['-c',brokerConfig],{stdio:'ignore'});
    await delay(250);assert.equal(broker.exitCode,null,'Mosquitto did not start');
    for(const [host,ca,expected] of [['localhost',true,true],['127.0.0.1',true,false],['localhost',false,false]]) {
      const config=path.join(dir,'settings.json');
      fs.writeFileSync(config,JSON.stringify({car:'tls-test',http_port:apiPort,http_token:token,http_token_file:path.join(dir,'token'),config_draft_path:path.join(dir,'draft.json'),prometheus_enabled:false,mqtt_host:host,mqtt_port:brokerPort,mqtt_tls:true,mqtt_ca_file:ca?path.join(dir,'ca.pem'):null}));
      bridge=spawn(process.env.CAN_BINARY||path.join(root,'target/debug/can-to-mqtt'),['--config',config,'--web-only'],{stdio:'ignore'});
      let state;
      for(let i=0;i<(expected?80:120);i++) {
        try {state=await(await fetch(`http://127.0.0.1:${apiPort}/api/v1/status`,{headers:{Authorization:`Bearer ${token}`}})).json();}catch{}
        if(state?.mqtt_connected===true)break;
        await delay(50);
      }
      assert.equal(state?.mqtt_connected,expected,`TLS result for ${host}, trusted CA ${ca}`);
      await stop(bridge);bridge=null;
    }
    console.log('PASS: actual Mosquitto TLS accepts matching trusted hostname; rejects wrong hostname and untrusted certificate. Acquisition remains stopped.');
  } finally {await stop(bridge);await stop(broker);fs.rmSync(dir,{recursive:true,force:true});}
})().catch(error=>{console.error(error);process.exitCode=1;});
