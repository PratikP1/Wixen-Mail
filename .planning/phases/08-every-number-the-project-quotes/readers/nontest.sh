ref="$1"
tot=0
for f in $(git ls-tree -r --name-only "$ref" -- src | grep '\.rs$' | grep -v -e 'src/presentation/wx_' -e 'src/presentation/managers\.rs' -e 'src/main\.rs' -e 'src/vendor/'); do
  n=$(git show "$ref:$f" | awk '/^#\[cfg\(test\)\]/{print NR-1; found=1; exit} END{if(!found) print NR}')
  tot=$((tot+n))
done
echo "$ref non-test, non-excluded src lines: $tot"
