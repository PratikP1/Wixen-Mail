import re, sys, collections
src = sys.argv[1]
text = open(src, encoding='utf-8').read()
chunks = text.split('[[guard]]')[1:]
print("records:", len(chunks))
per_record_files = []
file_to_records = collections.Counter()
red_counts = []
for c in chunks:
    name = re.search(r'^name = "(.*)"', c, re.M)
    files = re.findall(r'\{\s*file\s*=\s*"([^"]+)"\s*,\s*tests\s*=\s*(\d+)\s*\}', c)
    fs = sorted(set(f for f,_ in files))
    per_record_files.append(len(fs))
    for f in fs:
        file_to_records[f]+=1
    red = re.search(r'^red = \[(.*?)^\]', c, re.M|re.S)
    if red:
        red_counts.append(len(re.findall(r'"', red.group(1)))//2)
    else:
        red_counts.append(0)
d = collections.Counter(per_record_files)
print("records by number of distinct files in tests_last_seen:")
for k in sorted(d): print("  ", k, "file(s):", d[k])
print("records with no tests_last_seen at all:", d.get(0,0))
print()
print("top 12 files by how many records name them:")
for f,n in file_to_records.most_common(12): print(f"  {n:5d}  {f}")
print()
print("distinct files named:", len(file_to_records))
rc = collections.Counter(red_counts)
print("red-list sizes: min", min(red_counts), "max", max(red_counts), "mean", round(sum(red_counts)/len(red_counts),2))
print("total named red tests:", sum(red_counts))
print("records with red list of size 1:", rc.get(1,0))
