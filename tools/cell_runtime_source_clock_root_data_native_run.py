"""Root-only five-host source metadata composition; default preflight, --run required.
No retail activity or broad cohort reruns. Every actual native reply is external
to immutable request stages; all true failures and mapping-only attempts retained.
"""
import argparse,json,struct,sys
from contextlib import contextmanager
from pathlib import Path
import cell_runtime_source_clock_root_data_fixture as stages
import cell_runtime_clock_root_data_native_run as legacy
import cell_runtime_clip_metadata_native_probe as metadata_probe
from cell_runtime_clip_metadata_native_run import inspect as inspect_metadata
from cell_runtime_clock_native_run import inspect as inspect_clock
import cell_runtime_curves_native_probe_v3 as curve_probe
import cell_runtime_curves_fixture as source
from cell_runtime_data_native_run import digest,pinned,reviewed_file_run
from cell_schema_probe import SourceImage,SOURCE_SHA256
ROOT=Path(__file__).resolve().parent.parent
INITIAL=ROOT/'target/source-cell/runtime-source-clock-root-data-join-v4-metadata'
HARDPINS=ROOT/'target/source-cell/runtime-source-clock-root-data-runner-preparation-v1/hardpins.json'
HARDPINS_SHA='a982ac94deaf682a79aa01ecb8626a0ed3041e878c38315f5edecfaeb4266499'
def save(path,value):
 with path.open('x')as f:json.dump(value,f,indent=2);f.write('\n')
def word(raw,at):return struct.unpack_from('<I',raw,at)[0]
def metadata(raw,at):
 cw,sw,csr=struct.unpack_from('<HHI',raw,at);assert cw==0x027f and sw&0x3800==0 and csr&~0x3f==0x1f80
def all_files(folder):return[p.resolve()for p in folder.rglob('*')if p.is_file()]
def unique(paths):return list(dict.fromkeys(p.resolve()for p in paths))
def channel_data(folder,case):
    payload=(folder/case['clip']).read_bytes();prepared=(folder/case['prepared']).read_bytes()
    assert payload[:4]==b'ANIM'and word(payload,4)==len(payload)
    channel=case['channel'];count,table=word(payload,324),word(payload,328)
    assert channel<count and channel<64 and table+4*count<=len(payload)
    at=word(payload,table+4*channel);assert payload[at:at+4]==b'CHAN'
    rows,keys,n,descriptors=struct.unpack_from('<4I',payload,at+212)
    assert 0<rows<=160 and n<=24 and len(prepared)==n*32 and descriptors+n*32<=len(payload)
    assert word(payload,at+172)&1
    original=[struct.unpack_from('<8I',payload,descriptors+32*i)for i in range(n)]
    actual=[struct.unpack_from('<8I',prepared,32*i)for i in range(n)]
    info=next(d for d in original if d[0]&15==0 and d[1])
    assert info[3]>0 and keys+(rows-1)*info[3]+info[2]+20<=len(payload)
    mask=0;custom=[]
    for index,(a,b)in enumerate(zip(actual,original)):
        assert a[1:4]==b[1:4]and a[5:]==b[5:]
        if a[0]&15!=3:continue
        assert b[0]&15==3 and a[0]in[0x93,0xB3]and 3<=a[4]<=8 and a[0]&0x40==0
        assert b[3]>0 and keys+(rows-1)*b[3]+b[2]+40<=len(payload)
        block=b''.join(payload[keys+i*info[3]+info[2]:keys+i*info[3]+info[2]+20]+
            payload[keys+i*b[3]+b[2]:keys+i*b[3]+b[2]+40]for i in range(rows))
        assert len(block)==rows*60;custom.append((index,a,block));mask|=1<<index
    binding=case['binding'];packed=binding[3].to_bytes(4,'little')
    assert binding[0:2]==[1,0]and binding[2]&0xffffff==mask and packed[0]==0 and packed[1]==channel and packed[3]&9==0
    assert len(custom)in[2,3]
    return payload,prepared,rows,custom


def validate_curves(folder,clock_output):
    cases=json.loads((folder/'cases.json').read_text());records=json.loads((folder/'curve-cases.json').read_text())
    assert len(records)==72
    clocks=clock_output.read_bytes();assert len(clocks)==32*28
    with (folder/'curve-input.bin').open('rb')as inputs:
        ordinal=0
        for case in cases:
            payload,_,rows,custom=channel_data(folder,case);frame=word(clocks,28*case['case']+16)
            for component,descriptor,block in custom:
                record=records[ordinal];request=inputs.read(9632)
                assert record['curve_case']==ordinal and record['clock_case']==case['case']and record['component']==component
                policy=curve_probe.POLICIES.index((descriptor[0],descriptor[4]))
                assert struct.unpack_from('<8I',request)==(frame,rows,word(payload,288),word(payload,280)&8,0x027F,policy,0,0)
                assert request[32:32+rows*60]==block and request[32+rows*60:]==bytes(9600-rows*60)
                assert record['frame_bits']==f'{frame:08X}'
                ordinal+=1
        assert ordinal==72 and inputs.read(1)==b''


def inspect_curves(input_path,reply_path):
    assert input_path.stat().st_size==72*9632 and reply_path.stat().st_size==72*476
    with input_path.open('rb')as inputs,reply_path.open('rb')as replies:
        for case in range(72):
            req,reply=inputs.read(9632),replies.read(476);metadata(reply,468)
            frame,count,duration,wrap,cw,policy,r0,r1=struct.unpack_from('<8I',req)
            assert cw==0x027F and r0==r1==0 and 0<count<=160 and wrap in[0,8]
            assert source.finite(frame)and source.finite(duration)
            flags,slot=curve_probe.POLICIES[policy];times=[word(req,32+60*i)for i in range(count)]
            upper=source.upper_index(times,source.number(frame),cw);neighbors=source.neighbors(count,upper,bool(wrap))
            assert word(reply,464)==upper and list(struct.unpack_from('<4I',reply,448))==neighbors
            left,right=neighbors[1:3];value=source.number(frame)
            exact=left if value==source.unsigned_key(times[left],cw)else right if value==source.unsigned_key(times[right],cw)else 0xffffffff
            assert word(reply,440)==exact and word(reply,444)==int(exact!=0xffffffff)
            assert reply[:20]==req[32+60*left:32+60*left+20]
            offset=curve_probe.SLOTS[slot];extent=40 if left==right else 8 if flags&16 else 0
            if left==right:assert reply[offset:offset+40]==req[32+60*left+20:32+60*left+60]
            for at in range(20,440):
                if not offset<=at<offset+extent:assert reply[at]==0xA5
            assert all(source.finite(word(reply,offset+4*i))for i in range(2))
    return dict(cases=72,whole_rows=True,upper_neighbors_exact_index_and_padding_checked=True,event_arguments='null by the independently reviewed fixed host contract')


def validate_contributions(folder,curve_output):
    cases=json.loads((folder/'cases.json').read_text());records=json.loads((folder/'curve-cases.json').read_text())
    samples={i:[]for i in range(32)}
    with curve_output.open('rb')as replies:
        for record in records:
            reply=replies.read(476);slot=record['slot'];value,weight=struct.unpack_from('<2I',reply,200+(slot-3)*40)
            samples[record['clock_case']].append((record['component'],slot,value,weight))
    with (folder/'contribution-input.bin').open('rb')as inputs:
        for case in cases:
            req=inputs.read(1268);_,prepared,_,_=channel_data(folder,case);n=len(prepared)//32
            assert word(req,0)==0x3F800000 and word(req,4)==0x027F and word(req,8)==case['binding'][2]&0xffffff
            assert word(req,12)==word(req,16)==word(req,24)==word(req,28)==word(req,32)==0 and word(req,20)==n
            assert req[36:196]==b'\xA5'*160 and req[244:244+len(prepared)]==prepared and req[244+len(prepared):]==bytes(1024-len(prepared))
            slots=[(0xA5A5A5A5,0xA5A5A5A5)]*6
            for component,slot,value,weight in sorted(samples[case['case']]):slots[slot-3]=(value,weight)
            assert req[196:244]==b''.join(struct.pack('<2I',*slot)for slot in slots)
        assert inputs.read(1)==b''


def inspect_contributions(input_path,reply_path):
    assert input_path.stat().st_size==32*1268 and reply_path.stat().st_size==32*184
    with input_path.open('rb')as inputs,reply_path.open('rb')as replies:
        for case in range(32):
            req,reply=inputs.read(1268),replies.read(184);metadata(reply,176)
            assert word(reply,172)==1 and word(reply,164)==word(req,8)and word(reply,0)<=8
            last=None
            for i in range(word(req,20)):
                at=244+32*i
                if word(req,8)&(1<<i)and word(req,at)&15==3:last=word(req,at+16)
            assert last is not None and word(reply,168)==word(req,200+8*(last-3))
            for i in range(8):assert word(reply,4+20*i+12)==0xA5A5A5A5
    return dict(cases=32,active_mask_last_weight_and_scalar_unwritten_words_checked=True)


def validate_consumers(folder,contribution_output):
    assert(folder/'consumer-input.bin').stat().st_size==32*200
    with (folder/'consumer-input.bin').open('rb')as inputs,contribution_output.open('rb')as replies:
        for case in range(32):
            req,reply=inputs.read(200),replies.read(184)
            assert struct.unpack_from('<9I',req)==(0,1,1,1,0x027F,0,0,0,0)and req[36:]==reply[:164]


def extended32(bits):
    sign=(bits>>31)<<15;exponent=(bits>>23)&255;fraction=bits&0x7fffff
    assert exponent!=255
    if exponent:mantissa=((1<<23)|fraction)<<40;exponent=exponent-127+16383
    elif fraction:
        leading=fraction.bit_length()-1;mantissa=fraction<<(63-leading);exponent=leading-149+16383
    else:mantissa=0
    return struct.pack('<QH',mantissa,sign|exponent)


def inspect_consumers(reply_path):
    assert reply_path.stat().st_size==32*28
    with reply_path.open('rb')as replies:
        for case in range(32):
            reply=replies.read(28);metadata(reply,20)
            assert reply[10:12]==bytes(2)and word(reply,16)==1 and source.finite(word(reply,12))
            assert reply[:10]==extended32(word(reply,12))
    return dict(cases=32,retained80_commit32_manager_call_and_padding_checked=True)


@contextmanager
def input_pins(paths,folder):
    folder.mkdir(parents=True,exist_ok=False)
    with pinned(unique(paths),folder):yield


def audit_metadata(facts,image):
 folder=ROOT/facts['directory'];manifest=json.loads((folder/'manifest.json').read_text());linked=json.loads((folder/'linked-audit.json').read_text());base=metadata_probe.base
 saved=(base.CODE,base.EXITS,base.BUFFERS)
 try:
  assert digest(folder/'host-x86.exe')==facts['exe_sha256']==linked['host_exe_sha256']
  assert digest(folder/'host-x86.asm')==facts['asm_sha256']and digest(folder/'linked-audit.json')==facts['linked_sha256']
  assert metadata_probe.audit_source(image)==manifest['code_allowlist']
  assert metadata_probe.host_source(image)==(folder/'host-x86.asm').read_text();base.BUFFERS=metadata_probe.BUFFERS
  actual=base.linked_audit(folder/'host-x86.exe',folder/'host-x86.map',image);assert all(linked.get(k)==v for k,v in actual.items())
  pe=base.PE(folder/'host-x86.exe');nt=word(pe.data,60);entry=pe.base+word(pe.data,nt+40);assert entry==int(linked['symbols']['HostMain'],16)and linked['pe_entrypoint_hostmain']
  return dict(**facts,entrypoint=f'{entry:08X}',entrypoint_matches_HostMain=True,fresh_original_and_linked_audit=True)
 finally:base.CODE,base.EXITS,base.BUFFERS=saved
def preflight(args):
 assert digest(HARDPINS)==HARDPINS_SHA;config=json.loads(HARDPINS.read_text())
 for path,expected in config['pins'].items():assert digest(ROOT/path)==expected,path
 assert args.initial_dir.resolve()==INITIAL.resolve();stage=stages.verify_stage(args.initial_dir);assert stage['stage']=='metadata'
 raw=(args.initial_dir/'metadata-input.bin').read_bytes();cases=json.loads((args.initial_dir/'metadata-cases.json').read_text());assert len(raw)==5*32 and len(cases)==5
 components=0;nonloop=0
 for case in cases:
  q=raw[32*case['metadata_case']:32*(case['metadata_case']+1)];payload,_,_,custom=channel_data(args.initial_dir,case)
  assert word(payload,8)==25 and q[:4]==bytes.fromhex('7f020000')and q[16:]==bytes(16)
  flags,interval,frames=struct.unpack_from('<III',payload,280);assert struct.unpack_from('<III',q,4)==(flags,frames,interval)
  assert stages.bounded(frames,interval)
  if not(flags&8):nonloop+=1;components+=8*len(custom)
 assert nonloop==4 and components==72
 for entry in config['acceptances']:
  r=json.loads((ROOT/entry['path']).read_text());assert r.get('cases',r.get('records'))==entry['cases']
  if 'accepted'in r:assert r['accepted']
  if 'exact'in r:assert r['exact']
  elif 'modeled_sha256'in r:assert r['modeled_sha256']==r['native_projection_sha256']
 image=SourceImage(args.source_exe);four={k:v for k,v in config['hosts'].items()if k!='metadata'};hosts=legacy.audit_hosts(dict(hosts=four),image);hosts['metadata']=audit_metadata(config['hosts']['metadata'],image)
 return config,hosts,dict(metadata_cases=5,clock_cases=32,fresh_curve_requests=72,constructor_loop_rejections=8,source_v25_header_mapping_checked=True,whole_rows_and_prepared_descriptors_checked=True,clock_domain='Derived from actual native metadata before dependent host execution')
def derive(parent,destination,flag,native,previous_outputs,run_folder,label):
 with input_pins(all_files(parent)+previous_outputs,run_folder/'pins'/f'{label}-preparation'):
  previous=sys.argv
  try:sys.argv=[str(Path(stages.__file__)),'--input-dir',str(parent),'--out',str(destination),flag,str(native)];stages.main()
  finally:sys.argv=previous
  stages.verify_stage(destination)
def run_host(config,name,request,stage_folder,native_folder,previous_outputs):
 stage=stages.verify_stage(stage_folder);assert stage['stage']==name
 native_folder.mkdir(parents=True,exist_ok=False)
 with input_pins(all_files(stage_folder)+previous_outputs,native_folder/'input-pins'):
  host=ROOT/config['hosts'][name]['directory']/'host-x86.exe';status,reply=reviewed_file_run(host.resolve(),request,native_folder,name)
  assert status['exit']==0 and status['stderr_bytes']==0 and status['exception']is None and status['cleanup_exception']is None and not status['timeout'],status
  return reply,status
def execute(args,config,hosts):
 native=args.out/'native';native.mkdir();derived=args.out/'stages';derived.mkdir();paths=[ROOT/p for p in config['pins']]+[args.source_exe,HARDPINS,Path(__file__),args.out/'preflight.json'];phase='metadata'
 with input_pins(paths,args.out/'global-pins'):
  try:
   meta_reply,meta_status=run_host(config,'metadata',args.initial_dir/'metadata-input.bin',args.initial_dir,native/'metadata',[]);inspect_metadata(args.initial_dir/'metadata-input.bin',meta_reply,native/'metadata/native-projection16.bin');meta_summary=dict(cases=5,projection_bytes_per_case=16,metadata_checked=True)
   for at in range(0,5*24,24):
    reply=meta_reply.read_bytes()[at:at+24];assert struct.unpack_from('<Q',reply,4)[0]==struct.unpack('<Q',struct.pack('<d',source.number(word(reply,0))))[0]
   phase='clock preparation';clock_stage=derived/'clock';derive(args.initial_dir,clock_stage,'--metadata-output',meta_reply,[meta_reply],args.out,'clock');stages.strict_clock_domain(clock_stage)
   phase='clock';clock_reply,clock_status=run_host(config,'clock',clock_stage/'clock-input.bin',clock_stage,native/'clock',[meta_reply]);clock_summary=inspect_clock(clock_stage/'clock-input.bin',clock_reply,native/'clock/native-projection20.bin');assert clock_summary['cases']==32
   phase='curve preparation';curve_stage=derived/'curve';derive(clock_stage,curve_stage,'--clock-output',clock_reply,[meta_reply,clock_reply],args.out,'curve');validate_curves(curve_stage,clock_reply)
   phase='curve';curve_reply,curve_status=run_host(config,'curve',curve_stage/'curve-input.bin',curve_stage,native/'curve',[meta_reply,clock_reply]);curve_summary=inspect_curves(curve_stage/'curve-input.bin',curve_reply)
   phase='contribution preparation';contribution_stage=derived/'contribution';derive(curve_stage,contribution_stage,'--curve-output',curve_reply,[meta_reply,clock_reply,curve_reply],args.out,'contribution');validate_contributions(contribution_stage,curve_reply)
   phase='contribution';contribution_reply,contribution_status=run_host(config,'contribution',contribution_stage/'contribution-input.bin',contribution_stage,native/'contribution',[meta_reply,clock_reply,curve_reply]);contribution_summary=inspect_contributions(contribution_stage/'contribution-input.bin',contribution_reply)
   phase='consumer preparation';consumer_stage=derived/'consumer';derive(contribution_stage,consumer_stage,'--contribution-output',contribution_reply,[meta_reply,clock_reply,curve_reply,contribution_reply],args.out,'consumer');validate_consumers(consumer_stage,contribution_reply)
   phase='consumer';consumer_reply,consumer_status=run_host(config,'consumer',consumer_stage/'consumer-input.bin',consumer_stage,native/'consumer',[meta_reply,clock_reply,curve_reply,contribution_reply]);consumer_summary=inspect_consumers(consumer_reply)
   phase='projection';projection=args.out/'native-projection268.bin';outputs=[meta_reply,clock_reply,curve_reply,contribution_reply,consumer_reply]
   with input_pins(all_files(consumer_stage)+outputs,args.out/'pins/final-projection'):
    m=meta_reply.read_bytes();cases=json.loads((consumer_stage/'cases.json').read_text())
    with clock_reply.open('rb')as clocks,(consumer_stage/'contribution-input.bin').open('rb')as requests,contribution_reply.open('rb')as contributions,consumer_reply.open('rb')as consumers,projection.open('xb')as output:
     for case in cases:
      at=24*case['metadata_case'];output.write(m[at:at+16]+clocks.read(28)[:20]+requests.read(1268)[196:244]+contributions.read(184)[:164]+consumers.read(28)[:20])
   assert projection.stat().st_size==32*268
   command=['tools/build.bat','run','-p','spore-core','--example','cell_runtime_source_clock_root_data_verify','--',str(consumer_stage.resolve()),*[str(p.resolve())for p in outputs],str((args.out/'rust-projection268.bin').resolve())]
   r=dict(schema='fresh-native-source-metadata-clock-root-data-composition-v1',cases=32,metadata_cases=5,source_channels=4,fresh_curve_requests=72,constructor_loop_rejections=8,hosts=hosts,metadata=meta_summary,clock=clock_summary,curves=curve_summary,contributions=contribution_summary,consumer=consumer_summary,final_stage=str(consumer_stage.resolve()),metadata_output=str(meta_reply.resolve()),clock_output=str(clock_reply.resolve()),curve_output=str(curve_reply.resolve()),contribution_output=str(contribution_reply.resolve()),consumer_output=str(consumer_reply.resolve()),native_output_hashes={str(p.resolve()):digest(p)for p in outputs},native_projection268_sha256=digest(projection),native_executed=True,rust_compared=False,pending_rust_command=command,scope='Five actual source constructor metadata outputs ->32 first-duration nonloop clock requests ->72fresh full-channel curves ->ordered scalar contributions ->data2 consumer. Explicit selectedv25resource/rootbinding/actorcontext,positiveblend1; no live registry/queued forcedloop/eventcounter/scheduling claim.')
   save(args.out/'report.json',r);print(json.dumps(r,indent=2))
  except BaseException as e:save(args.out/'failure.json',dict(phase=phase,error=repr(e),true_failure_retained=True));raise
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,required=True);p.add_argument('--initial-dir',type=Path,default=INITIAL);p.add_argument('--source-exe',type=Path,default=Path('D:/Games/SPORE Collection/SporeBin/SporeApp.exe'));p.add_argument('--run',action='store_true');args=p.parse_args();args.out.resolve().relative_to((ROOT/'target').resolve());config,hosts,domain=preflight(args);args.out.mkdir(parents=True,exist_ok=False);save(args.out/'preflight.json',dict(schema='source-metadata-clock-root-data-native-preflight-v1',domain=domain,hosts=hosts,hardpins_sha256=HARDPINS_SHA,hardpinned_files=len(config['pins']),source_sha256=SOURCE_SHA256,runner_sha256=digest(Path(__file__)),native_execution_requested=args.run))
 if not args.run:print(json.dumps(dict(preflight=True,native_executed=False,**domain)));return
 execute(args,config,hosts)
if __name__=='__main__':main()
