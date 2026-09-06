
import re, sys, pathlib, collections
NOUNS = r'(tests?|records?|mutants?|commits?|guards?|percent|%|MB|seconds?|minutes?|hours?|days?|rows?|messages?|files?|modules?|survivors?|misses?|events?|channels?|settings?|shortcuts?)'
NUM = r'(?<![\w.])\d[\d,]*(?:\.\d+)?'
pat = re.compile(NUM + r'\s*' + NOUNS, re.I)
PROV = re.compile(r'(measured|counted|taken|re-measured|as of|20\d\d-\d\d-\d\d|`cargo|`git|`grep|`scripts/)', re.I)
hits=[]
for root in sys.argv[1:]:
    for p in sorted(pathlib.Path(root).rglob("*.md")):
        s=p.as_posix()
        try: text=p.read_text(encoding="utf-8")
        except Exception: continue
        lines=text.split("\n")
        for i,l in enumerate(lines):
            for m in pat.finditer(l):
                window=" ".join(lines[i:i+3])
                hits.append((s,i+1,m.group(0),bool(PROV.search(window)),l.strip()[:110]))
print("number-shaped claims found:", len(hits))
withp=[h for h in hits if h[3]]
print("with a provenance marker within 3 lines:", len(withp))
print("without:", len(hits)-len(withp))
c=collections.Counter(h[0] for h in hits)
print()
print("top files by claims (bare = no provenance marker within 3 lines):")
for f,n in c.most_common(20):
    nb=sum(1 for h in hits if h[0]==f and not h[3])
    print("  %5d (%4d bare)  %s" % (n, nb, f))
