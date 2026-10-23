"""Census original authored ANIM POS sampling before bounded native execution.

Reads all measured DBPF index shapes. Transform descriptor preparation follows
the native initial INFO/POS/ROT template; deformation layouts do not replace
these three slots. Source payloads and diagnostics remain ignored target/.
"""
import argparse
import bisect
from collections import Counter
import hashlib
import json
import math
from pathlib import Path
import shutil
import struct
import subprocess
import time

from dbpf_extract import payload
from gmdl_skin_probe import indexes

TYPE = 0xee17c6ad
TSV_COLUMNS = ['package','type','group','instance','channel','frame_bits','cw',
    'x_bits','y_bits','z_bits','weight_bits','previous','left','right','following','native_case_ordinal']
PLAN_COLUMNS = ['package','type','group','instance','channel','frame_bits','cw',
    'previous','left','right','following','native_case_ordinal','kind',
    'source_row_count','source_upper','local_row_count','local_upper','nonfinite_consumed_source_fields']
SOURCE_SHA256 = '17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a'
HOST_SHA256 = 'd4b2becc97dc710bcba333f751c041a084416a47ec54598ab43820301a906723'
CW = 0x027f


def u32(data, offset):
    return struct.unpack_from('<I', data, offset)[0]


def f32(value):
    return struct.unpack('<f', struct.pack('<f', value))[0]


def bits(value):
    return struct.unpack('<I', struct.pack('<f', value))[0]


def next_float(value, upward):
    word = bits(value)
    if word & 0x7fffffff == 0:
        return struct.unpack('<f', struct.pack('<I', 1 if upward else 0x80000001))[0]
    word += 1 if (value > 0) == upward else -1
    return struct.unpack('<f', struct.pack('<I', word))[0]


def neighbors(count, upper, wrap):
    right = min(upper, count-1)
    left = right if upper == count else max(upper-1, 0)
    previous = left-1 if left else ((count-2) % count if wrap else left)
    following = right+1 if right<count-1 else ((right+2) % count if wrap else right)
    return previous,left,right,following


def planned_samples(frames, wrap):
    """Return frame/kind/full-source neighbors and explicit uncovered spans."""
    first,last=frames[0],frames[-1]
    before=f32(first)
    if before>=first:before=next_float(before,False)
    after=f32(last)
    if after<last:after=next_float(after,True)
    proposed=[(before,'before_first'),(after,'at_or_after_last')]
    if f32(first)==first:
        proposed.append((f32(first),'first_time'))
    distinct=list(dict.fromkeys(frames))
    uncovered=[]
    for left,right in zip(distinct,distinct[1:]):
        midpoint=f32((left+right)/2)
        kind='segment_midpoint'
        if not left<=midpoint<right:
            midpoint=f32(left)
            kind='segment_left_fallback'
            if midpoint<left:midpoint=next_float(midpoint,True)
        if not left<=midpoint<right:
            uncovered.append((left,right,'no_representable_binary32_frame'))
        else:
            proposed.append((midpoint,kind))
    result=[]
    seen=set()
    for frame,kind in proposed:
        if bits(frame) in seen:continue
        seen.add(bits(frame))
        upper=bisect.bisect_right(frames,frame)
        selected=neighbors(len(frames),upper,wrap)
        result.append((frame,kind,selected))
    return result,uncovered,len(distinct)-1


def channels(data):
    if len(data)<512 or data[:4]!=b'ANIM' or u32(data,4)!=len(data):
        raise ValueError('invalid_ANIM_header')
    duration=struct.unpack_from('<f',data,288)[0]
    header_flags=u32(data,280)
    count,table=struct.unpack_from('<2I',data,324)
    if table+4*count>len(data):raise ValueError('channel_table_overrun')
    result=[]
    for channel in range(count):
        start=u32(data,table+4*channel)
        if start+228>len(data) or data[start:start+4]!=b'CHAN':
            raise ValueError('invalid_CHAN_header')
        row_count,key_start,descriptor_count,descriptor_start=struct.unpack_from('<4I',data,start+212)
        if descriptor_start+32*descriptor_count>len(data):raise ValueError('descriptor_overrun')
        descriptors=[struct.unpack_from('<8I',data,descriptor_start+32*i) for i in range(descriptor_count)]
        item=dict(channel=channel,count=row_count,duration=duration,header_flags=header_flags,
            reason=None,descriptors=descriptor_count)
        result.append(item)
        if not row_count:
            item['reason']='empty_channel';continue
        if not math.isfinite(duration):
            item['reason']='nonfinite_duration';continue
        if not descriptors:
            item['reason']='missing_descriptors';continue
        stride=descriptors[0][3]
        if not stride or any(d[3]!=stride or d[2]>=stride for d in descriptors):
            item['reason']='unsupported_inconsistent_stride';continue
        if key_start+row_count*stride>len(data):
            item['reason']='raw_rows_overrun';continue
        info=[d for d in descriptors if d[1]==int.from_bytes(b'INFO','little')]
        native_info=next((d for d in descriptors if d[0]&15==0 and d[1]),None)
        if len(info)!=1 or native_info!=info[0] or info[0][2]+20>stride:
            item['reason']='unsupported_INFO_geometry';continue
        # Fixed initial transform slots. A matched POS retains its enable bit;
        # an unmatched component has enable cleared and slot FFFFFFFF.
        position=[d for d in descriptors if d[0]&15==1 and d[1] in
            [int.from_bytes(b'POS\0','little'),0xffffffff]]
        if not position:
            item['reason']='absent_matched_POS';continue
        if len(position)!=1:
            item['reason']='unsupported_multiple_POS';continue
        position=position[0]
        if position[2]+80>stride:
            item['reason']='unsupported_POS80_geometry';continue
        frames=[u32(data,key_start+i*stride+info[0][2]) for i in range(row_count)]
        if any(a>b for a,b in zip(frames,frames[1:])):
            item['reason']='unordered_source_times';continue
        item.update(stride=stride,key_start=key_start,info_offset=info[0][2],
            pos_offset=position[2],enabled=bool(position[0]&0x10),frames=frames)
    return result


def resources(root):
    for package in sorted(root.rglob('*.package')):
        try:
            entries=indexes(package)
        except Exception as exc:
            yield package,None,None,f'index:{exc!r}'
            continue
        yield package,None,None,None
        for entry in entries:
            if entry['type']!=TYPE:continue
            try:
                data=payload(package,entry)
            except Exception as exc:
                yield package,entry,None,f'payload:{exc!r}'
                continue
            yield package,entry,data,None


def census(root,out):
    stats=Counter();reasons=Counter();kinds=Counter();packages=Counter()
    failures=[]
    with (out/'resources.jsonl').open('w',encoding='utf-8') as source_log, (out/'channels.tsv').open('w',encoding='utf-8') as channel_log:
        channel_log.write('package\tkey\tchannel\tcount\tstatus\tplanned_cases\n')
        for package,entry,data,error in resources(root):
            relative=package.relative_to(root).as_posix()
            if entry is None:
                stats['indexes']+=1
                if error:failures.append(dict(package=relative,error=error))
                continue
            stats['resources']+=1;packages[relative]+=1
            key=f"{entry['type']:08X}:{entry['group']:08X}:{entry['instance']:016X}"
            if error:
                failures.append(dict(package=relative,key=key,error=error));continue
            source_log.write(json.dumps(dict(package=relative,key=key,sha256=hashlib.sha256(data).hexdigest()))+'\n')
            stats['source_payload_bytes']+=len(data)
            try:
                parsed=channels(data)
            except Exception as exc:
                failures.append(dict(package=relative,key=key,error=repr(exc)));continue
            for item in parsed:
                stats['channels']+=1
                if item['reason']:
                    reasons[item['reason']]+=1
                    channel_log.write(f"{relative}\t{key}\t{item['channel']}\t{item['count']}\t{item['reason']}\t0\n")
                    continue
                stats['supported_POS_channels']+=1
                stats['source_POS_rows']+=item['count']
                stats['disabled_POS_channels']+=not item['enabled']
                samples,uncovered,segments=planned_samples(item['frames'],bool(item['header_flags']&8))
                stats['distinct_segments']+=segments
                stats['nonrepresentable_segments']+=len(uncovered)
                planned=0
                for _frame,kind,selected in samples:
                    if selected[1]!=selected[2] and not item['enabled']:
                        stats['disabled_interpolation_samples']+=1;continue
                    planned+=1;kinds[kind]+=1
                    stats['native_copy_cases' if selected[1]==selected[2] else 'native_four_neighbor_cases']+=1
                stats['planned_cases']+=planned
                channel_log.write(f"{relative}\t{key}\t{item['channel']}\t{item['count']}\t{'enabled' if item['enabled'] else 'disabled_copies_only'}\t{planned}\n")
            if stats['resources']%500==0:print(f"CENSUS_PROGRESS resources={stats['resources']} channels={stats['channels']} cases={stats['planned_cases']}",flush=True)
    count=stats['planned_cases']
    report=dict(context='All original package copies; native initial transform slots; explicit CW027F only; selected-neighbor host bypasses full-clip selector.',
        statistics=dict(stats),channel_reasons=dict(reasons),sample_kinds=dict(kinds),packages=dict(packages),failures=failures,
        fixed_request_bytes=1632,fixed_reply_bytes=124,estimated_input_bytes=count*1632,
        estimated_output_bytes=count*124,estimated_tsv_bytes=count*190,
        estimated_total_bytes=count*(1632+124+190),available_workspace_bytes=shutil.disk_usage(out).free,
        expected_tsv_columns=TSV_COLUMNS,host_executed=False,
        source_resource_log_sha256=hashlib.sha256((out/'resources.jsonl').read_bytes()).hexdigest())
    (out/'census.json').write_text(json.dumps(report,indent=2))
    print(json.dumps(report,indent=2),flush=True)
    if failures:raise SystemExit(1)


def file_hash(path):
    digest=hashlib.sha256()
    with path.open('rb') as source:
        while chunk:=source.read(1024*1024):digest.update(chunk)
    return digest.hexdigest()


def normalized_row(data,item,index):
    at=item['key_start']+index*item['stride']
    info=at+item['info_offset'];position=at+item['pos_offset']
    row=data[info:info+20]+data[position:position+80]
    if len(row)!=100:raise ValueError('normalized_row_extent')
    return row


def consumed_nonfinite(rows,copied):
    """Only values/slopes actually read by the bounded position component.

    Unused slope NaNs and the opaque control bytes do not narrow the finite
    API domain. Retain every native output even when this count is nonzero.
    """
    bad=[]
    if copied:
        fields=[(0,20+axis*4,f'value{axis}') for axis in range(4)]
    else:
        fields=[]
        for axis in range(4):
            fields.extend([(1,20+axis*4,f'left.value{axis}'),
                (2,20+axis*4,f'right.value{axis}')])
            control=36+axis*16
            left_mode=rows[1][control+9]&3
            right_mode=rows[2][control+8]&3
            if left_mode==0:fields.append((0,20+axis*4,f'previous.value{axis}'))
            elif left_mode==3:fields.append((1,control+4,f'left.next_slope{axis}'))
            if right_mode==0:fields.append((3,20+axis*4,f'following.value{axis}'))
            elif right_mode==3:fields.append((2,control,f'right.previous_slope{axis}'))
    for index,at,label in fields:
        if not math.isfinite(struct.unpack_from('<f',rows[index],at)[0]):bad.append(label)
    return ','.join(bad) or '-'


def build_source_streams(root,out,census_dir):
    prior=json.loads((census_dir/'census.json').read_text())
    if prior['failures']:raise ValueError('census_has_failures')
    if file_hash(census_dir/'resources.jsonl')!=prior['source_resource_log_sha256']:
        raise ValueError('changed_census_resource_log')
    stats=Counter();reasons=Counter();kinds=Counter()
    with (census_dir/'resources.jsonl').open(encoding='utf-8') as source_log, \
            (out/'input.bin').open('wb') as supplied, \
            (out/'case-plan.tsv').open('w',encoding='utf-8',newline='') as plan, \
            (out/'limitations.tsv').open('w',encoding='utf-8',newline='') as limits:
        plan.write('\t'.join(PLAN_COLUMNS)+'\n')
        limits.write('package\ttype\tgroup\tinstance\tchannel\treason\tframe_bits\tleft_frame\tright_frame\n')
        for package,entry,data,error in resources(root):
            relative=package.relative_to(root).as_posix()
            if error:raise ValueError((relative,error))
            if entry is None:stats['indexes']+=1;continue
            stats['resources']+=1
            key=f"{entry['type']:08X}:{entry['group']:08X}:{entry['instance']:016X}"
            recorded=json.loads(next(source_log))
            actual=dict(package=relative,key=key,sha256=hashlib.sha256(data).hexdigest())
            if actual!=recorded:raise ValueError(('changed_source_resource',actual,recorded))
            stats['source_payload_bytes']+=len(data)
            prefix=[relative,f"{entry['type']:08X}",f"{entry['group']:08X}",f"{entry['instance']:016X}"]
            for item in channels(data):
                stats['channels']+=1
                channel_prefix=prefix+[str(item['channel'])]
                if item['reason']:
                    reasons[item['reason']]+=1
                    limits.write('\t'.join(channel_prefix+[item['reason'],'-','-','-'])+'\n')
                    continue
                stats['supported_POS_channels']+=1
                stats['source_POS_rows']+=item['count']
                stats['disabled_POS_channels']+=not item['enabled']
                samples,uncovered,segments=planned_samples(item['frames'],bool(item['header_flags']&8))
                stats['distinct_segments']+=segments
                stats['nonrepresentable_segments']+=len(uncovered)
                for left,right,reason in uncovered:
                    limits.write('\t'.join(channel_prefix+[reason,'-',str(left),str(right)])+'\n')
                # Extract each source row once per clip channel, preserving all
                # INFO and tangent payload bytes. The host sees only one/four.
                source_rows=[normalized_row(data,item,index) for index in range(item['count'])]
                for frame,kind,selected in samples:
                    copied=selected[1]==selected[2]
                    if not copied and not item['enabled']:
                        stats['disabled_interpolation_samples']+=1
                        limits.write('\t'.join(channel_prefix+['disabled_interpolation',f'{bits(frame):08X}','-','-'])+'\n')
                        continue
                    source_upper=bisect.bisect_right(item['frames'],frame)
                    if copied:
                        local_rows=[source_rows[selected[1]]]
                        local_upper=int(frame>=item['frames'][selected[1]])
                    else:
                        local_rows=[source_rows[index] for index in selected]
                        local_upper=2
                    local_count=len(local_rows)
                    request=struct.pack('<5IHH2I',bits(frame),local_upper,local_count,
                        bits(item['duration']),item['header_flags'],CW,0,0,0)
                    request+=b''.join(local_rows)+bytes((16-local_count)*100)
                    if len(request)!=1632:raise ValueError('request_extent')
                    supplied.write(request)
                    consumed=consumed_nonfinite(local_rows,copied)
                    ordinal=stats['planned_cases']
                    row=channel_prefix+[f'{bits(frame):08X}',f'{CW:04X}']+list(map(str,selected))+[
                        str(ordinal),kind,str(item['count']),str(source_upper),str(local_count),
                        str(local_upper),consumed]
                    if len(row)!=len(PLAN_COLUMNS):raise ValueError('plan_columns')
                    plan.write('\t'.join(row)+'\n')
                    stats['planned_cases']+=1;kinds[kind]+=1
                    stats['native_copy_cases' if copied else 'native_four_neighbor_cases']+=1
                    stats['nonfinite_consumed_source_cases']+=consumed!='-'
            if stats['resources']%500==0:
                print(f"SOURCE_INPUT_PROGRESS resources={stats['resources']} cases={stats['planned_cases']}",flush=True)
        if source_log.read():raise ValueError('missing_source_resources')
    for name,value in prior['statistics'].items():
        if stats[name]!=value:raise ValueError(('census_changed',name,stats[name],value))
    if dict(reasons)!=prior['channel_reasons'] or dict(kinds)!=prior['sample_kinds']:
        raise ValueError('census_reason_or_kind_changed')
    if (out/'input.bin').stat().st_size!=1632*stats['planned_cases']:
        raise ValueError('source_request_file_extent')
    report=dict(statistics=dict(stats),channel_reasons=dict(reasons),sample_kinds=dict(kinds),
        source_resource_log_sha256=prior['source_resource_log_sha256'],
        input_sha256=file_hash(out/'input.bin'),case_plan_sha256=file_hash(out/'case-plan.tsv'),
        limitations_sha256=file_hash(out/'limitations.tsv'),host_executed=False)
    (out/'input-inventory.json').write_text(json.dumps(report,indent=2))
    return report


def verify_source_replies(out,input_report):
    count=input_report['statistics']['planned_cases']
    if (out/'output.bin').stat().st_size!=count*124:raise ValueError('source_reply_file_extent')
    stats=Counter();nonfinite_kinds=Counter();local_policies=Counter()
    with (out/'input.bin').open('rb') as supplied, (out/'output.bin').open('rb') as replies, \
            (out/'case-plan.tsv').open(encoding='utf-8') as plan, \
            (out/'expected.partial.tsv').open('w',encoding='utf-8',newline='') as expected, \
            (out/'excluded.tsv').open('w',encoding='utf-8',newline='') as excluded:
        if next(plan).rstrip('\n').split('\t')!=PLAN_COLUMNS:raise ValueError('plan_header')
        expected.write('\t'.join(TSV_COLUMNS)+'\n')
        excluded.write('\t'.join(TSV_COLUMNS+['reason','kind','nonfinite_consumed_source_fields'])+'\n')
        for ordinal,line in enumerate(plan):
            metadata=line.rstrip('\n').split('\t')
            if len(metadata)!=len(PLAN_COLUMNS) or int(metadata[11])!=ordinal:
                raise ValueError(('plan_ordinal_or_extent',ordinal))
            request=supplied.read(1632);reply=replies.read(124)
            if len(request)!=1632 or len(reply)!=124:raise ValueError(('short_record',ordinal))
            frame,upper,local_count=struct.unpack_from('<f2I',request)
            if [local_count,upper]!=list(map(int,metadata[15:17])):
                raise ValueError(('local_geometry',ordinal))
            selected=neighbors(local_count,upper,bool(u32(request,16)&8))
            actual_neighbors=struct.unpack_from('<4I',reply,108)
            if actual_neighbors!=selected:raise ValueError(('native_local_neighbors',ordinal,actual_neighbors,selected))
            exact,success=struct.unpack_from('<iI',reply,100)
            local_left,local_right=selected[1:3]
            left_time=u32(request,32+100*local_left);right_time=u32(request,32+100*local_right)
            wanted_exact=local_left if frame==left_time else local_right if frame==right_time else -1
            if (exact,success)!=(wanted_exact,int(wanted_exact!=-1)):
                raise ValueError(('native_exact_key_result',ordinal,exact,success,wanted_exact))
            if local_count==1:
                if int(metadata[8])!=int(metadata[9]):raise ValueError(('one_row_noncopy',ordinal))
                if reply[:100]!=request[32:132]:raise ValueError(('native_full_row_copy',ordinal))
                stats['native_full_row_copies_checked']+=1
            elif local_count==4:
                if selected!=(0,1,2,3) or int(metadata[8])==int(metadata[9]):
                    raise ValueError(('four_row_neighbor_map',ordinal))
                if reply[:20]!=request[132:152] or reply[36:100]!=b'\xa5'*64:
                    raise ValueError(('native_interpolation_store_extent',ordinal))
                stats['native_four_row_extents_checked']+=1
            else:raise ValueError(('unexpected_local_count',ordinal))
            stats['cases']+=1
            output_bits=struct.unpack_from('<4I',reply,20)
            output_finite=all(math.isfinite(value) for value in struct.unpack_from('<4f',reply,20))
            source_finite=metadata[17]=='-'
            stats['finite_position_weight_outputs' if output_finite else 'nonfinite_position_weight_outputs']+=1
            stats['finite_consumed_source_cases' if source_finite else 'nonfinite_consumed_source_cases']+=1
            row=metadata[:7]+[f'{word:08X}' for word in output_bits]+metadata[7:12]
            if len(row)!=16:raise ValueError('expected_columns')
            if output_finite and source_finite:
                expected.write('\t'.join(row)+'\n');stats['finite_API_cases']+=1
            else:
                reason=','.join(x for x,yes in [('nonfinite_consumed_source',not source_finite),
                    ('nonfinite_native_output',not output_finite)] if yes)
                excluded.write('\t'.join(row+[reason,metadata[12],metadata[17]])+'\n')
                nonfinite_kinds[reason]+=1;stats['excluded_cases']+=1
            local_policies[f'{local_count}_rows_CW{metadata[6]}']+=1
            if stats['cases']%100000==0:
                print(f"SOURCE_OUTPUT_PROGRESS cases={stats['cases']} finite_API={stats['finite_API_cases']} excluded={stats['excluded_cases']}",flush=True)
        if supplied.read(1) or replies.read(1) or stats['cases']!=count:
            raise ValueError('extra_records_or_count_mismatch')
    (out/'expected.partial.tsv').replace(out/'expected.tsv')
    return dict(statistics=dict(stats),exclusion_reasons=dict(nonfinite_kinds),local_policies=dict(local_policies),
        output_sha256=file_hash(out/'output.bin'),expected_sha256=file_hash(out/'expected.tsv'),
        excluded_sha256=file_hash(out/'excluded.tsv'))


def execute(root,out,census_dir,host_dir):
    manifest=json.loads((host_dir/'manifest.json').read_text())
    host=(host_dir/'host-x86.exe').resolve()
    if manifest['source_sha256']!=SOURCE_SHA256 or file_hash(root/'SporeBin'/'SporeApp.exe')!=SOURCE_SHA256:
        raise ValueError('source_executable_hash')
    if manifest['host_exe_sha256']!=HOST_SHA256 or file_hash(host)!=HOST_SHA256:
        raise ValueError('unreviewed_host_hash')
    if manifest['stub']['audit']['outbound_relative_transfers']!=0:
        raise ValueError('host_stub_relocation_audit')
    if manifest['protocol']['input_bytes']!=1632 or manifest['protocol']['output_bytes']!=124:
        raise ValueError('host_protocol')
    input_report=build_source_streams(root,out,census_dir)
    started=time.monotonic()
    # File handles bound memory use and avoid bidirectional pipe deadlocks.
    # The reviewed host accepts successful zero-byte EOF at a fresh record.
    print(f"SOURCE_HOST_START cases={input_report['statistics']['planned_cases']}",flush=True)
    with (out/'input.bin').open('rb') as supplied, (out/'output.bin').open('wb') as replies, \
            (out/'stderr.bin').open('wb') as errors:
        result=subprocess.run([str(host)],stdin=supplied,stdout=replies,stderr=errors,
            creationflags=subprocess.CREATE_NO_WINDOW,timeout=300)
    elapsed=time.monotonic()-started
    host_run=dict(exit=result.returncode,elapsed_seconds=elapsed,
        reply_bytes=(out/'output.bin').stat().st_size,stderr_bytes=(out/'stderr.bin').stat().st_size)
    (out/'host-run.json').write_text(json.dumps(host_run,indent=2))
    if result.returncode or host_run['stderr_bytes']:
        raise ValueError(('source_host_failed',host_run))
    reply_report=verify_source_replies(out,input_report)
    report=dict(source_sha256=SOURCE_SHA256,host_exe_sha256=HOST_SHA256,
        original_x86_instructions_executed=True,retail_process_or_initializer_run=False,
        native_control_word_established=False,explicit_x87_control_word='027F',
        explicit_MXCSR='1F80',input_report=input_report,reply_report=reply_report,
        host_run=host_run,input_output_guards_intact=True,
        code_data_allowlist_unchanged=True,quaternion_deformation_branches_unreachable=True,
        scope='All source ANIM copies; fixed one/four-row INFO+POS only. Original full-channel strict-greater selection supplies row maps; host receives upper explicitly. Rust must select from original full source independently. Disabled interpolation and nonfinite cases remain named limitations.',
        expected_tsv_columns=TSV_COLUMNS,
        one_row_copy_metadata='Native local indices are all zero; source previous/following metadata is retained from full-channel selection and is not consumed by endpoint copies.')
    (out/'inventory.json').write_text(json.dumps(report,indent=2))
    print(json.dumps(report,indent=2),flush=True)


def main():
    ap=argparse.ArgumentParser()
    ap.add_argument('root',type=Path)
    ap.add_argument('--out',type=Path,required=True)
    ap.add_argument('--execute',action='store_true')
    ap.add_argument('--census-dir',type=Path)
    ap.add_argument('--host-dir',type=Path)
    args=ap.parse_args()
    target=(Path(__file__).resolve().parent.parent/'target').resolve()
    args.out.resolve().relative_to(target)
    if args.execute:
        if not args.census_dir or not args.host_dir:ap.error('--execute requires --census-dir and --host-dir')
        args.census_dir.resolve().relative_to(target)
        args.host_dir.resolve().relative_to(target)
    args.out.mkdir(parents=True,exist_ok=False)
    if args.execute:execute(args.root,args.out,args.census_dir,args.host_dir)
    else:census(args.root,args.out)


if __name__=='__main__':main()
