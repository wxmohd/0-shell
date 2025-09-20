echo "Enter directory name"
read NAME
if [ -d "$NAME" ]; then
  echo "Directory exist"
else
  mkdir "$NAME"
  echo "Directory created"
fi
