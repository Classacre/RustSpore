"""Compare four retained manifest prefixes with the independent raw-unit report."""
import hashlib,json,subprocess,struct
from pathlib import Path
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
directory=Path('target/source-cell/manifest-four-candidates-v1');report=directory/'report.json';exe=Path('target/debug/examples/manifest_four_compare_v1.exe');locale=Path('target/source-locale/corpus-v2/sources/01169.locale')
assert sha(report)=='0f2e690bfb98d6a943b632c8b3de84d416a79855960b3d5d08078fa1b4f61309'
assert sha(exe)=='ec90d37aae28c7a8a7d694cff4866d96499f03d8ab0364efbd90b00c7baeea3e'
r=json.loads(report.read_text());assert sha(locale)==r['locale_sha256'] and sha(r['package'])==r['package_sha256']
paths=[];expected=[]
for row in r['rows']:
 low=row['key'].split(':')[0][-8:];path=directory/(low+'.bin');assert sha(path)==row['decoded_sha256'];paths.append(path)
 first,name,optional=row['fields'];assert first['nonempty'] and name['nonempty'] and optional['sentinel'];parts=[]
 for field in (first,name):
  b=bytes.fromhex(field['raw_utf16le_hex']);parts.append(''.join(f'{u:04x}' for u in struct.unpack('<'+'H'*(len(b)//2),b)))
 expected.append(f"{low}\t{first['id']}\t{name['id']}\t{parts[0]}\t{parts[1]}\n")
p=subprocess.run([str(exe),str(locale),*[str(path) for path in paths]],capture_output=True,check=True)
assert p.stdout.decode().replace('\r\n','\n')==''.join(expected)
result={'matches':len(paths),'report_sha256':sha(report),'comparator_sha256':sha(exe),'locale_sha256':sha(locale),'package_sha256':r['package_sha256'],'stdout_sha256':hashlib.sha256(p.stdout).hexdigest(),'source_executed':False,'scope':'explicit supplied table and manifest prefix; no runtime provider or postprocessing'}
(directory/'rust-comparison.json').write_text(json.dumps(result,indent=2)+'\n',encoding='utf8');print(json.dumps(result))
