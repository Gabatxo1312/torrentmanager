#! /usr/bin/env bash

OUTPUT="fork-awesome.html"

cat << EOF > "$OUTPUT"
<!DOCTYPE html>
<html>
    <head>
        <link rel="stylesheet" href="fork-awesome.css">
    </head>
    <body>
EOF

grep -P -B 1 'content: ' fork-awesome.css 2>&1 | while read -r entry; do
    if [[ "$entry" = "--" ]] || echo "$entry" | grep "content: " 2>&1 >/dev/null ; then
        continue
    fi
    NAME="$(echo "$entry" | sed 's/:before.*$//' | sed 's/^.//')"
    echo "  <i class=\"fa $NAME\"></i> $NAME<br>" >> "$OUTPUT"
done

cat << EOF >> "$OUTPUT"
    </body>
</body>
EOF
