#!/usr/bin/env python3
# T23.reader-compat.3 orchestrator. For each fixture + each raw-TZif reader, compare the reader's
# interpretation of zic-rs output vs reference-zic output at a spread of UTC instants. Verdict per
# (reader, fixture). Readers that cannot consume raw TZif are classified separately (not run).
import subprocess, os
BIN="/tmp/readers/bin"
ZRS, REF = "/tmp/prov/zrs", "/tmp/prov/ref"
ZRSR, REFR = "/tmp/prov/zrs-right", "/tmp/prov/ref-right"

INSTANTS=[-2208988800,-2147483648,-631152000,0,946684800,1000000000,
          1561939200,1577836800,2147483647,2147483648,7258118400,1483228827]
IN="\n".join(str(i) for i in INSTANTS)+"\n"

# fixture: (label, zone-name, zrs-dir, ref-dir, note)
FIX=[
 ("Europe/Lisbon","Europe/Lisbon",ZRS,REF,"slim residual (structural edge)"),
 ("Asia/Singapore","Asia/Singapore",ZRS,REF,"odd historical offsets"),
 ("Pacific/Kiritimati","Pacific/Kiritimati",ZRS,REF,"+14 max offset"),
 ("Europe/London","Europe/London",ZRS,REF,"footer-heavy + double summer time"),
 ("Europe/Dublin","Europe/Dublin",ZRS,REF,"negative DST"),
 ("America/New_York[right]","America/New_York",ZRSR,REFR,"leap/right profile"),
]

def run(cmd, env=None):
    p=subprocess.run(cmd, input=IN, capture_output=True, text=True, env=env)
    return p.returncode, p.stdout, p.stderr

# each reader: (name, fn(zone,dir)->(rc,out), compare_fields)  fields: which columns matter
def glibc(zone,d):  return run([f"{BIN}/glibc_probe", os.path.join(d,zone)])
def goread(zone,d): return run([f"{BIN}/goprobe", os.path.join(d,zone), zone])
def absl(zone,d):
    e=dict(os.environ); e["TZDIR"]=d; return run([f"{BIN}/absl_probe", zone], env=e)

# normalize a probe output to comparable tuples, dropping fields the reader does not expose
def norm(out, drop_isdst=False):
    rows=[]
    for ln in out.strip().splitlines():
        p=ln.split()
        if len(p)<4: continue
        e,off,isd,abbr=p[0],p[1],p[2],p[3]
        rows.append((e,off, ("?" if drop_isdst else isd), abbr))
    return rows

READERS=[("glibc-localtime",glibc,False),("go-time",goread,True),("cctz-absl",absl,False)]

print("reader\tfixture\tverdict\tnote")
findings=[]
tally={}
for rname,fn,drop in READERS:
    for label,zone,zd,rd,note in FIX:
        rcz,oz,ez=fn(zone,zd); rcr,orr,er=fn(zone,rd)
        if rcz!=0 and rcr!=0:
            # cannot consume this file CLASS at all, on BOTH zic-rs and reference -> reader capability limit
            v="unsupported_by_reader"; n="reader cannot load this file class (fails on BOTH zic-rs AND reference): "+(ez or er).strip()[:80]
        elif rcz!=0 or rcr!=0:
            v="mismatch"; n="asymmetric load: zic-rs rc=%d ref rc=%d (%s)"%(rcz,rcr,(ez or er).strip()[:60]); findings.append((rname,label,[("load",str(rcz))],[("load",str(rcr))]))
        else:
            az,ar=norm(oz,drop),norm(orr,drop)
            if az==ar and len(az)==len(INSTANTS):
                v="match"; n=f"{len(az)} instants identical (zic-rs == ref)"
            else:
                v="mismatch"; n="differs"; findings.append((rname,label,az,ar))
        # leap capability annotation for the right/ fixture
        if "right" in label and v=="match" and rname in ("go-time","cctz-absl"):
            v="match_with_known_reader_limitation"; n="equivalent; reader ignores leap-seconds (no TAI offset applied) — capability, not a zic-rs issue"
        tally[v]=tally.get(v,0)+1
        print(f"{rname}\t{label}\t{v}\t{n}")

# readers that structurally cannot consume raw TZif (consume their own compiled DB)
for rname,mech in [("java-time","consumes $JAVA_HOME/lib/tzdb.dat (TZDB format), no public raw-TZif file loader"),
                   ("php-timelib","consumes bundled timelib DB (php -i: 'Timezone Database => internal', Olson 2026.1)"),
                   ("icu4c","consumes ICU zoneinfo64.res resource bundle, not raw TZif")]:
    for label,*_ in FIX:
        print(f"{rname}\t{label}\tunsupported_by_reader\t{mech}")
        tally["unsupported_by_reader"]=tally.get("unsupported_by_reader",0)+1

print("\n=== TALLY ===")
for k in ("match","match_with_known_reader_limitation","unsupported_by_reader","mismatch","unavailable"):
    if k in tally: print(f"  {k}: {tally[k]}")
if findings:
    print("\n=== MISMATCH FINDINGS (named, not hidden) ===")
    for rname,label,az,ar in findings:
        print(f"  {rname} / {label}:")
        for a,b in zip(az,ar):
            if a!=b: print(f"    inst {a[0]}: zic-rs={a[1:]} ref={b[1:]}")
