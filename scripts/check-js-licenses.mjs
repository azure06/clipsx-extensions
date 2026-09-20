import fs from 'node:fs';
const lock=JSON.parse(fs.readFileSync(new URL('../package-lock.json',import.meta.url),'utf8'));const denied=/(^|\s|\()AGPL|SSPL|BUSL|Commons Clause/i;const failures=[];const reviewedMissingMetadata=new Set(['node_modules/khroma']);
for(const [name,pkg] of Object.entries(lock.packages)){if(!name)continue;if((!pkg.license&&!reviewedMissingMetadata.has(name))||denied.test(pkg.license??''))failures.push(`${name}: ${pkg.license??'missing'}`)}
if(failures.length){console.error(`Dependency license policy failed:\n${failures.join('\n')}`);process.exit(1)}console.log(`Checked ${Object.keys(lock.packages).length-1} dependency records.`);
