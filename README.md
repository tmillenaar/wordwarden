# WordWarden

A command-line tool that finds undesired strings in files. It is intended to be used as a pre-commit hook to prevent committing or pushing if certain words are found in the changed files, such as debug statements or FIXME notices.

## General usage
```target/debug/word_warden <file1> <file2> 'word1' 'word2' 'word3']```

Use --casecheck and --no-casecheck to determine whether the check should be case-sensitive. The default is to not be case sensitive.

Use '--escape=skip-this-line' to ignore occurances of words found on a line with the specified escape string. The default escape string is 'wordwarden:skip-line'


## Pre-commit hook
Word Warden can be used as a pre-commit hook. To use it, add wordwarden to your .pre-commit-config.yaml.
It would look something like:
```
repos:
  - repo: https://github.com/tmillenaar/wordwarden
    rev: v0.1.2
    hooks:
      - id: wordwarden
        name: Check for Debug Statements
        entry: word_warden
        args: ["breakpoint()", ".set_trace()"]
```

Repository: [tmillenaar/wordwarden](https://github.com/tmillenaar/wordwarden)
