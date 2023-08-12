
#! /bin/sh

cd gh-pages

INDEX_FILE="source/index.md"
echo $INDEX_FILE

DATE_STRING=$(date +"%a, %b %d %Y - %I:%M %p")
echo $DATE_STRING

cat <<EOT >> $INDEX_FILE
\`\`\`
$DATE_STRING
branch=$BRANCH_NAME
$GITHUB_SHA
\`\`\`
EOT


webgenr -i source
cd ..


