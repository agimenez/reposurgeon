#!/usr/bin/env python
#
# rs - a repository surgeon.
#
import sys, os, getopt, commands, cStringIO, cmd, tempfile, readline

#
# All knowledge about specific version-control systems lives in the
# following dictionary. The key is the subdirectory name that tells us
# we have a given VCS active.  The values in the tuple are,
# respectively:
#
# * Name of the SCM for diagnostic messages
# * Command to export from the SCM to the interchange format
# * Command to initialize a new repo
# * Command to import from the interchange format
# * Command to check out working copies of the repo files.
#
# Note that some of the commands used here are plugins or extenstions
# that are not part of the basic VCS. Thus these may fail when called;
# we need to be prepared to cope with that.
#
# Subversion/RCS/CVS aren't in this table because exporting from them
# requires fixups of usernames in the committer information to full
# email addresses.  Trying to handle that entirely inside this tool
# would be excessively messy, so we don't. Instead we let the user
# transform dump files and cope with the export/import himself.
#
version="1.0"

vcstypes = [
     ("git",
      ".git",
      "git fast-export -M -C --all >%s",
      "git init",
      "git fast-import <%s",
      "git checkout"),
     # FIXME: hg and bzr methods are untested
     ("hg",
      ".hg",
      "hg-fast-export.sh %s",   # Not part of stock hg
      "hg init",
      "hg fast-import %s",      # Not part of stock hg
      "hg checkout"),
     ("bzr",
      ".bzr",
      "bzr-fast-export --plain %s",
      "bzr init",
      "bzr fast-import %s",
      "bzr checkout"),
    ]

class Action:
    "Represents an instance of a person acting on the repo."
    def __init__(self, person):
        person = person.replace(" <", "|").replace("> ", "|")
        (self.name, self.email, self.when) = person.strip().split("|")
    def __str__(self):
        return self.name + " <" + self.email + "> " + self.when

class Blob:
    "Represent a detached blob of data referenced by a mark."
    def __init__(self, subdir):
        self.mark = None
        self.subdir = subdir
    def blobfile(self):
        return self.subdir + "/blob-" + self.mark
    def __str__(self):
        dp = open(self.blobfile())
        content = dp.read()
        dp.close()
        return "blob\nmark %s\ndata %d\n%s\n" % (self.mark, len(content), content)

class Tag:
    "Represents an annotated tag."
    def __init__(self, name, committish, tagger, content):
        self.name = name
        self.committish = committish
        self.tagger = tagger
        self.comment = content
    def __str__(self):
        return "tag %s\nfrom %s\ntagger %s\ndata %d\n%s\n" \
             % (self.name, self.committish, self.tagger, len(self.comment), self.comment)

class Branch:
    "Represents a branch creation."
    def __init__(self):
        self.ref = None
        self.committish = None
    def __str__(self):
        st = "reset %s\n" % self.ref
        if self.committish:
            st += "from %s\n\n" % self.committish
        return st

class Commit:
    "Generic commit object."
    def __init__(self):
        self.mark = None             # Mark name of commit (may be None)
        self.author = None           # Author of commit
        self.committer = None        # Person responsible for committing it.
        self.comment = None          # Commit comment
        self.parents = []            # List of parent nodes
        self.branch = None           # branch name (deduced optimization hack)
        self.fileops = []            # blob and file operation list
    def __str__(self):
        st = "commit %s\n" % self.branch
        if self.mark:
            st += "mark %s\n" % self.mark
        if self.author:
            st += "author %s\n" % self.author
        if self.committer:
            st += "committer %s\n" % self.committer
        st += "data %d\n%s" % (len(self.comment), self.comment) 
        if self.parents:
            st += "from %s\n" % self.parents[0]
        for ancestor in self.parents[1:]:
            st += "merge %s\n" % self.parents[0]
        for op in self.fileops:
            if type(op) == type(""):
                st += op + "\n"
            else:
                st += " ".join(op[:4]) + "\n"
                if op[2] == 'inline':
                    fp = open(op[4])
                    content = fp.read()
                    fp.close()
                    st += "data %d\n%s\n" % (len(content), content)
        return st + "\n"

class RepoSurgeonException:
    def __init__(self, msg):
        self.msg = msg

class Repository:
    "Generic repository object."
    def __init__(self):
        self.repotype = None
        self.commands = []    # A list of the commands encountered, in order
        self.nmarks = 0
        self.refs_to_branches = {}
        self.import_line = 0
        self.subdir = ".rs"
    def error(self, msg, atline=True):
        if atline:
            raise RepoSurgeonException(msg + (" at line " + `self.import_line`))
        else:
            raise RepoSurgeonException(msg)
    def __del__(self):
        os.system("rm -fr %s" % (self.subdir,))
    def fast_import(self, fp, verbose=False):
        "Initialize repo object from fast-import stream."
        try:
            os.system("rm -fr %s; mkdir %s" % (self.subdir, self.subdir))
        except OSError:
            self.error("can't create operating directory", atline=False)
        self.import_line = 0
        linebuffers = []
        ncommits = 0
        def read_data(dp, line=None):
            if not line:
                line = readline()
            if line.startswith("data <<"):
                delim = line[7:]
                while True:
                    dataline = fp.readline()
                    if dataline == delim:
                        break
                    elif not dataline:
                        raise RepoSurgeonException("EOF while reading blob")
            elif line.startswith("data"):
                try:
                    count = int(line[5:])
                    dp.write(fp.read(count))
                except ValueSelf.Error:
                    raise self.error("bad count in data")
            else:
                raise self.error("malformed data header %s" % `line`)
            return dp
        def readline():
            if linebuffers:
                line = linebuffers.pop()
            else:
                self.import_line += 1
                line = fp.readline()
                if verbose:
                    print line.rstrip()
            return line
        def pushback(line):
            linebuffers.append(line)
        while True:
            line = readline()
            if not line:
                break
            elif not line.strip():
                continue
            elif line.startswith("blob"):
                blob = Blob(self.subdir)
                line = readline()
                if line.startswith("mark"):
                    blob.mark = line[5:].strip()
                    read_data(open(blob.blobfile(), "w")).close()
                    self.nmarks += 1
                else:
                    self.error("missing mark after blob")
                self.commands.append(blob)
            elif line.startswith("data"):
                self.error("unexpected data object")
            elif line.startswith("commit"):
                commitbegin = self.import_line
                commit = Commit()
                commit.branch = line.split()[1]
                ncommits += 1
                inlinecount = 0
                while True:
                    line = readline()
                    if not line:
                        self.error("EOF after commit")
                    elif line.startswith("mark"):
                        commit.mark = line[5:].strip()
                        self.nmarks += 1
                    elif line.startswith("author"):
                        try:
                            commit.author = Action(line[7:])
                        except ValueError:
                            self.error("malformed author line")
                    elif line.startswith("committer"):
                        try:
                            commit.committer = Action(line[10:])
                        except ValueError:
                            self.error("malformed committer line")
                    elif line.startswith("data"):
                        dp = read_data(cStringIO.StringIO(), line)
                        commit.comment = dp.getvalue()
                        dp.close()
                    elif line.startswith("from") or line.startswith("merge"):
                        commit.parents.append(line.split()[1])
                    elif line[0] in ("C", "D", "R"):
                        commit.fileops.append(line.strip().split())
                    elif line == "filedeletall\n":
                        commit.fileops.append("filedeleteall")
                    elif line[0] == "M":
                        (op, mode, ref, path) = line.split()
                        if ref[0] == ':':
                            copyname = self.subdir + "/blob-" + ref
                            commit.fileops.append((op, mode, ref, path, copyname))
                        elif ref[0] == 'inline':
                            copyname = self.subdir + "/inline-" + `inline_count`
                            self.read_data(open(copyname, "w")).close()
                            inline_count += 1
                            commit.fileops.append((op, mode, ref, path, copyname))
                        else:
                            self.error("unknown content type in filemodify")
                    else:
                        pushback(line)
                        break
                if not (commit.mark and commit.author and commit.committer):
                    self.import_line = commitbegin
                    self.error("missing required fields in commit")
                self.commands.append(commit)
            elif line.startswith("reset"):
                branch = Branch()
                branch.ref = line[6:].strip()
                line = readline()
                if line.startswith("from"):
                    branch.committish = line[5:].strip()
                else:
                    pushback(line)
                self.commands.append(branch)
            elif line.startswith("tag"):
                tagname = line[4:].strip()
                line = readline()
                if line.startswith("from"):
                    referent = line[5:].strip()
                else:
                    self.error("missing from after tag")
                line = readline()
                if line.startswith("tagger"):
                        try:
                            tagger = Action(line[7:])
                        except ValueError:
                            self.error("malformed tagger line")
                else:
                    self.error("missing tagger after from in tag")
                dp = read_data(cStringIO.StringIO())
                self.commands.append(Tag(tagname,
                                       referent, tagger, dp.getvalue()))

            else:
                # Simply pass through any line we don't understand.
                commands.append(line)
    def fast_export(self, fp):
        "Dump the repo object in fast-export format."
        for command in self.commands:
            fp.write(str(command))

def read_repo(source, verbose):
    "Read a repository using fast-import."
    if source == '-':
        repo = Repository()
        repo.fast_import(sys.stdin)
    elif not os.path.exists(source):
        print >>sys.stderr, "rs: %s does not exist" % source
        return None
    elif not os.path.isdir(source):
        repo = Repository()
        repo.fast_import(open(source))
    else:
        for (name, dirname, exporter, initializer, importer, checkout) in vcstypes:
            subdir = os.path.join(source, dirname)
            
            if os.path.exists(subdir) and os.path.isdir(subdir):
                break
        else:
            print >>sys.stderr,"rs: could not find a repository under %s" % source
            return None
        if verbose:
            print "rs: recognized %s repository under %s" % (name, source)
        try:
            repo = Repository()
            (tfdesc, tfname) = tempfile.mkstemp()
            cmd = "cd %s >/dev/null;" % source
            cmd += exporter % tfname
            act(cmd)
            tp = open(tfname)
            repo.fast_import(tp);
            tp.close()
        finally:
            os.remove(tfname)
        (repo.type, repo.initializer, repo.importer, repo.checkout) = (name,
                                                                       initializer,
                                                                       importer,
                                                                       checkout)
    return repo

def write_repo(repo, target, verbose):
    "Write a repository using fast-export."
    if target == '-':
        repo.fast_export(sys.stdout)
    # FIXME: Have to decide a policy here

def act(cmd):
    (err, out) = commands.getstatusoutput(cmd)
    if err:
        raise RepoSurgeonException("'%s' failed" % cmd)
    else:
        return out

def fatal(msg):
    print >>sys.stderr, "rs:", msg
    raise SystemExit, 1

class RepoSurgeon(cmd.Cmd):
    "Repository surgeon command interpreter."
    def __init__(self):
        cmd.Cmd.__init__(self)
        self.use_rawinput = True
        self.verbose = 0
        self.prompt = "rs# "
        self.repo = None
    def postcmd(self, stop, line):
        if line == "EOF":
            return True
    def emptyline(self):
        pass
    def help_help(self):
        print "Show help for a command. Follow with a space and the command name"
    def do_verbose(self, line):
        "'verbose 1' enables progress and statistics messages, 'verbose 0' disables them."
        try:
            self.verbose = int(line)
        except ValueError:
            print "rs: verbosity value must be an integer"
    def do_version(self, line):
        "Report the program version and supported version-control systems."
        print "reposurgeon " + version + " supporting " + " ".join(map(lambda x: x[0], vcstypes))
    def do_read(self, line):
        "Read in a repository for surgery."
        if not line:
            line = '.';
        self.repo = read_repo(line, self.verbose)
        if self.verbose:
            print "rs: %d commands, %d blobs, %d commits, %d marks, %d tags" % \
                  (len(self.repo.commands),
                   len(filter(lambda x: isinstance(x,Blob),self.repo.commands)),
                   len(filter(lambda x: isinstance(x,Commit),self.repo.commands)),
                   self.repo.nmarks,
                   len(filter(lambda x: isinstance(x,Tag),self.repo.commands)))
    def help_read(self):
        print """
A read command with no arguments is treated as 'read .', operating on the
current directory.
        
With a directory-name argument, this command attempts to read in the
contents of a repository in any supported version-control system under
that directory.

If the argument is the name of a plain file, it will be read in as a
fast-import stream.

With an argument of <quote>-</quote>, this command reads a fast-import
stream from standard input (this will be useful in filters constructed
with command-line arguments).
"""
    def do_write(self, line):
        "Write out the results of repo surgery."
        if not line:
            line = '-'
        write_repo(self.repo, line, self.verbose)
    def do_shell(self, line):
        "Execute a shell command."
        os.system(line)
    def do_EOF(self, line):
        "Terminate the browser."
        return True

if __name__ == '__main__':
    try:
        interpreter = RepoSurgeon()
        if not sys.argv[1:]:
            sys.argv.append("-")
        for arg in sys.argv[1:]:
            for cmd in arg.split(";"):
                if cmd == '-':
                    interpreter.onecmd("verbose 1")
                    interpreter.cmdloop()
                else:
                    interpreter.onecmd(cmd)
    except RepoSurgeonException, e:
        fatal(e.msg)
    except KeyboardInterrupt:
        print ""

# end
