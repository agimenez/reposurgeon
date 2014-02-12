## Tests of selection-set syntax and parser features
echo 1
read <simple.fi
=H resolve Special set resolution
15 resolve Event number resolution
=TR resolve Special cimbination
@min(=TR) resolve min operator
@max(=TR) resolve max operator
24..97 resolve Range
24..97&=C resolve Range and conjunction
24..97? resolve Neighborhood extension
(24..97?) & 20..40 resolve Conjunct
(24..97?) & (20..40?) resolve Conjunct with grouping
24..97 | 20..40 resolve Disjunct
(24..97 | 20..40) & =C resolve Commit selection
24..97 & =C | 20..40 & =C resolve Disjunct and conjunct
24..97 & (=C | 20..40) resolve Parenthesis grouping
3,:15 resolve Comma syntax selecting doubleton
/master/b resolve Branch
/operating theater/B resolve Blobs
<lightweight-sample> resolve Branch tip
/annotated-sample/b resolve Tag
<annotated-sample> resolve Tag location
/regression/ resolve Text search
/Raymond/ resolve Commit comment search
[Makefile] resolve Path search
~[Makefile] resolve Negated path search 
=B & [Makefile] resolve Blob path search
=C & [Makefile] resolve Commit path search
[/Ma.*le/] resolve Regexp commit search
=B & [/Ma.*le/] resolve Regexp patch search for blobs
=C & [/Ma.*le/] resolve Regexp patch search for commits
[/^Ma.*le$/] resolve Anchored regexp commit search
=B & [/^Ma.*le$/] resolve Anchored regexp patch search for blobs
=C & [/^Ma.*le$/] resolve Anchored regexp patch search for commits
[/D.ME.\.txt/] resolve Regexp escape
[/Makefile/a] resolve Author match
[/^Make/a] resolve Anchored author match
[/^test/] resolve Text search
[/^test/c] resolve Comment search
[/^r/ca] resolve Commit and author search
[/r/ca] resolve Commit and author search
<2010-10-27T18:43:32Z> resolve Date resolution
<2010-10-27T12:07:32Z!esr@thyrsus.com> resolve Action stamp resolution
<2010-10-27> resolve Partial-date resolution
@amp(1) resolve resolve amplified nonempty set
@amp(/mytzlpyk/) resolve amplified empty set
# Test here-doc syntax
echo 0
authors read <<EOF
esr = Eric Raymond <esr@thyrsus.com>
EOF
echo 1
write -
# Test multiline commands
resolve \
<annotated-sample>
resolve \
24..97\
&\
=C
