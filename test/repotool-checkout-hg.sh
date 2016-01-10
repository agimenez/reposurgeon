## Test repotool checkout of Mercurial repo

trap "rm -rf /tmp/test-repo$$ /tmp/target$$" 0 12 2 15

./hg-to-fi -n /tmp/test-repo$$ < simple.fi
cd /tmp/test-repo$$
repotool checkout /tmp/target$$; echo Return code: $?
cd - >/dev/null
./dir-md5 /tmp/target$$

#end

