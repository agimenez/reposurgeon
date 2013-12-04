#
# makefile for reposurgeon
#

INSTALL=install
prefix?=/usr/local
mandir?=share/man
target=$(DESTDIR)$(prefix)

VERS=$(shell sed <reposurgeon -n -e '/version=\(.*\)/s//\1/p')

SOURCES = README NEWS AUTHORS COPYING TODO
SOURCES += \
	reposurgeon reposurgeon.xml \
	repopuller repopuller.xml \
	repodiffer repodiffer.xml \
	conversion.mk features.asc \
	reposurgeon-mode.el
SOURCES += Makefile control reposturgeon.png

all: reposurgeon.1 repopuller.1 repodiffer.1 \
     reposurgeon.html repopuller.html repodiffer.html \
     features.html

reposurgeon.1: reposurgeon.xml
	xmlto man reposurgeon.xml

reposurgeon.html: reposurgeon.xml
	xmlto html-nochunks reposurgeon.xml

repopuller.1: repopuller.xml
	xmlto man repopuller.xml

repopuller.html: repopuller.xml
	xmlto html-nochunks repopuller.xml

repodiffer.1: repodiffer.xml
	xmlto man repodiffer.xml

repodiffer.html: repodiffer.xml
	xmlto html-nochunks repodiffer.xml

features.html: features.asc
	asciidoc features.asc

reporting-bugs.html: reporting-bugs.asc
	asciidoc reporting-bugs.asc

PYVERSION=2.7
cyreposurgeon: reposurgeon
	cython --embed reposurgeon -o cyreposurgeon.c
	$(CC) -I /usr/include/python$(PYVERSION) cyreposurgeon.c -lpython$(PYVERSION) -o cyreposurgeon

install: all
	$(INSTALL) -d "$(target)/bin"
	$(INSTALL) -d "$(target)/share/doc/reposurgeon"
	$(INSTALL) -d "$(target)/$(mandir)/man1"
	$(INSTALL) -m 755 reposurgeon repopuller repodiffer "$(target)/bin"
	$(INSTALL) -m 644 README NEWS TODO *.html \
		"$(target)/share/doc/reposurgeon"
	$(INSTALL) -m 644 *.1 "$(target)/$(mandir)/man1"

clean:
	rm -fr  *~ *.1 *.html *.tar.gz MANIFEST *.md5
	rm -fr .rs .rs* test/.rs test/.rs*
	rm -f typescript test/typescript *.pyc
	rm -f cyreposurgeon.c cyreposurgeon

reposurgeon-$(VERS).tar.gz: $(SOURCES)
	@ls $(SOURCES) | sed s:^:reposurgeon-$(VERS)/: >MANIFEST
	@(cd ..; ln -s reposurgeon reposurgeon-$(VERS))
	(cd ..; tar -czf reposurgeon/reposurgeon-$(VERS).tar.gz `cat reposurgeon/MANIFEST`)
	@(cd ..; rm reposurgeon-$(VERS))

version:
	@echo $(VERS)

# Include W1401 in both sets when I get my pylint updated
COMMON_PYLINT = --rcfile=/dev/null --reports=n --include-ids=y
PYLINTOPTS1 = $(COMMON_PYLINT) --disable=C0103,C0111,C0301,C0302,C0322,C0324,C0321,C0323,R0201,R0902,R0903,R0904,R0911,R0912,R0913,R0914,R0915,W0108,W0141,W0142,W0212,W0233,W0603,W0511,W0611,E1101,E1103
PYLINTOPTS2 = $(COMMON_PYLINT) --disable=C0103,C0111,C0301,W0603,W0621,E1101,E1103,R0902,R0903,R0912,R0914,R0915
pylint:
	@pylint --output-format=parseable $(PYLINTOPTS1) reposurgeon
	@pylint --output-format=parseable $(PYLINTOPTS2) repodiffer

check:
	cd test; $(MAKE) --quiet

dist: reposurgeon-$(VERS).tar.gz reposurgeon.1 repopuller.1 repodiffer.1

reposurgeon-$(VERS).md5: reposurgeon-$(VERS).tar.gz
	@md5sum reposurgeon-$(VERS).tar.gz >reposurgeon-$(VERS).md5

zip: $(SOURCES)
	zip -r reposurgeon-$(VERS).zip $(SOURCES)

release: reposurgeon-$(VERS).tar.gz reposurgeon-$(VERS).md5 reposurgeon.html repodiffer.html reporting-bugs.html features.html
	shipper version=$(VERS) | sh -e -x
