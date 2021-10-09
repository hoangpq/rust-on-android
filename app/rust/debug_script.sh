set enforce 0
/data/local/tmp/gdbserver :1337 --attach $(ps -A | grep com.node.sample | awk '{print $2}')