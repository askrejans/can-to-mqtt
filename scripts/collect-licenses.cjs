/* Release tooling only. Collect exact locked dependency notices without network lookups. */
'use strict';
const fs = require('node:fs');
const path = require('node:path');
const {execFileSync} = require('node:child_process');
const root = path.resolve(__dirname, '..');
const out = path.resolve(process.argv[2] || path.join(root, 'dist/licenses/third-party'));
const metadata = JSON.parse(execFileSync('cargo',['metadata','--locked','--format-version','1'],{cwd:root,encoding:'utf8',maxBuffer:16*1024*1024}));
const packages = metadata.packages.filter(pkg=>pkg.source).sort((a,b)=>`${a.name}-${a.version}`.localeCompare(`${b.name}-${b.version}`));
const licenseName = name => /^(licen[cs]e|copying|copyright|notice|unlicense|authors|patents)/i.test(name) || ['edl-v10','epl-v20'].includes(name);
function copyNotices(source, destination) {
  let count=0;
  if(!fs.existsSync(source)) return count;
  for(const entry of fs.readdirSync(source,{withFileTypes:true})) {
    if(!entry.isFile() || !licenseName(entry.name)) continue;
    const from=path.join(source,entry.name);
    if(fs.statSync(from).size>512*1024) throw Error('Unexpected notice size: '+from);
    fs.mkdirSync(destination,{recursive:true});fs.copyFileSync(from,path.join(destination,entry.name));count++;
  }
  return count;
}
for(const pkg of packages) {
  const name=`${pkg.name}-${pkg.version}`, source=path.dirname(pkg.manifest_path), destination=path.join(out,name);
  let count=copyNotices(source,destination);
  if(pkg.license_file) {
    fs.mkdirSync(destination,{recursive:true});fs.copyFileSync(path.resolve(source,pkg.license_file),path.join(destination,path.basename(pkg.license_file)));count++;
  }
  count+=copyNotices(path.join(root,'licenses/overrides',name),destination);
  if(!count) throw Error('No license text for locked dependency '+name+'; add a verified upstream override');
  // These sys crates compile bundled C source whose own notices must also travel.
  for(const nested of ['openssl','paho.mqtt.c']) copyNotices(path.join(source,nested),path.join(destination,nested));
}
fs.mkdirSync(out,{recursive:true});
fs.copyFileSync(path.join(root,'licenses/overrides/manifest.json'),path.join(out,'override-sources.json'));
const lines = ['# Third-party notices','','First-party can-to-mqtt 0.4.1 is GPL-3.0-only. Third-party components retain the terms below; this table is generated from the exact Cargo.lock resolution, including build and platform-specific dependencies that may not be linked into every binary. Multiple-license choices remain available as specified upstream.','','Release archives include collected original license/copyright/notice texts in `licenses/third-party/`. Run `node scripts/collect-licenses.cjs <output-directory>` from the source checkout to regenerate them after `cargo fetch --locked`. The four overrides in `licenses/overrides/` record pinned upstream texts omitted from their crate packages. Bundled Paho C and OpenSSL notices are included separately inside their sys/source crate directories.','','Linux system libraries such as glibc and libudev are supplied by the host distribution, not included in these archives. Their runtime packages retain their own licensing and source distribution terms. The vendored OpenSSL source crate currently contains OpenSSL 3.6.3; this is the latest compatible published crate in this lockfile and is not a claim that upstream OpenSSL has no newer release.','','| Crate | Version | Declared SPDX license | Upstream |','| --- | --- | --- | --- |'];
for(const pkg of packages) lines.push(`| ${pkg.name} | ${pkg.version} | ${pkg.license||'See original license file'} | ${pkg.repository ? `[source](${pkg.repository})` : `[crate](https://crates.io/crates/${pkg.name}/${pkg.version})`} |`);
fs.writeFileSync(path.join(root,'THIRD_PARTY_NOTICES.md'),lines.join('\n')+'\n');
console.log(`Collected original notices for ${packages.length} locked crates in ${out}`);
