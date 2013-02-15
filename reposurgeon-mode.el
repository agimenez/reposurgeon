;; A mode for editing the mailbox-like comment dumps produced by reposurgeon.
;;
;; Canonicalizing thousands of comments in a mailbox_out dump is the grottiest
;; part of lifting a repository, but if you don't do it you are probably going
;; to miss things that should turn into reference cookies.  This mode aims to
;; speed up the process.
;;
;; Work in progress - neither code nor bindings should be considered stable.

(defun decimal-digit-after ()
  (and (>= (char-after) ?0) (<= (char-after) ?9)))

(defun svn-cookify ()
  "Turn a Subversion revision number around point into a reference cookie."
  (interactive)
  (if (not (decimal-digit-after))
      (error "Expecting decimal digit."))
  (backward-word)
  ;; Ignore preceding r
  (if (= (char-after 1) ?r)
      (delete-char 1))
  (insert "[[SVN:")
  (while (decimal-digit-after)
    (forward-char 1))
  (insert "]]")
  )

(defun cvs-rev-char-after ()
  (or (== (char-after) ?.) (decimal-digit-after)))

(defun cvs-cookify ()
  "Turn CVS reference around point into a reference cookie."
  (interactive)
  (if (not (cvs-rev-char-after))
      (error "Expecting decimal digit or dot."))
  (backward-word)
  (insert "[[CVS:")
  (while (cvs-rev-char-after)
    (forward-char 1))
  (insert "]]")
  )

(defun cvs-split-summary ()
  "Break the first line of a paragraph comment following git conventions."
  (interactive)
  (delete-horizontal-space)
  (if (= (char-after ?\n)) (delete-char 1))
  (let ((c (char-before)))
	(cond ((member c '(?\. ?\! ?\?))
	       (insert "\n\n"))
	      ((member c '(?\, ?\: [semicolon] ?\,))
	       (insert "\n\n..."))
	      (t
	       (insert "...\n\n...")))))

(defun svn-reference-lift ()
  "Interactively lift probable SVN revision numbers into ref cookies en masse."
  (interactive)
  (query-replace-regexp "\\br\\([0-9][0-9]+\\)\\b" "[[SVN:\\1]]"))

(defvar reposurgeon-mode-map nil "Keymap for reposurgeon-mode")

(when (not reposurgeon-mode-map)
  (setq reposurgeon-mode-map (make-sparse-keymap))
  (define-key reposurgeon-mode-map (kbd "C-x s") 'svn-cookify)
  (define-key reposurgeon-mode-map (kbd "C-x c") 'cvs-cookify)
  (define-key reposurgeon-mode-map (kbd "C-x .") 'cvs-split-summary)
  )

(define-derived-mode reposurgeon-mode
  text-mode "Reposurgeon"
  "Major mode for editing reposurgeon comment dumps.
\\{reposurgeon-mode-map}"
  (setq case-fold-search nil))

;; end


