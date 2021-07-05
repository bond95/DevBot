#!/bin/bash

ssh  "$DEPLOY_LOGIN@$DEPLOY_HOST"  <<'EOL'
	kill $(ps aux | grep 'dev_bot' | awk '{print $2}')
	echo "Killed dev_bot"
	exit
EOL

cd target/release

echo "Copying toxic bot to host..."
scp ./dev_bot "$DEPLOY_LOGIN@$DEPLOY_HOST":~/

ssh  "$DEPLOY_LOGIN@$DEPLOY_HOST"  <<'EOL'
	echo "Starting dev bot"
	nohup ./dev_bot > /dev/null &
	exit
EOL
