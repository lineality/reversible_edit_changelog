# buttons_reversible_edit_changelog

Buttons-Undo-Redo: A Reversible File-byte-edit Changelog System
"If we button up again, we'll not be all undone."
2025/10/30

# Project: reversible_edit_changelog.rs module
- Rust Language
- goal: write & read config files to byte operations.
```
e.g.
// a file-editor that can log character level changes:

fn button_make_character_action_changelog(option(character),file_position,action_type, log_directory_path:path(pathbuf?)) (note: undo or redo is determined by log_directory_path)

fn button_make_hexedit_changelog(character,file_position)

// a file-editor that can undo the LIFO change:
fn button_undo_next_changelog_lifo(path_to_log_directory)

```
- Base: https://github.com/lineality/basic_file_byte_operations


# overall

A change-log is the opposite of what the user did on a character level (add or remove a character).
The "change log" is instructions to undo/redo what the user did/undid.
If the user adds "hello"
the change-logs are:

remove h position
remove e position
remove l position
remove l position
remove o position

It does not matter (per se) what character is there (unless you are double checking).

If the user deletes "hello"

the change-logs are:

at position add o
at position add l
at position add l
at position add e
at position add h


If the user hex-edits a byte to become hex-61 but was FF.

The change-logs is:
at position, hex-edit FF.



## Disambiguation: "undo" vs. "redo": The term 'redo' is sometimes used to describe re-creating something that was deleted. "Un-replacing," "unplacing?" is not clear terminology. "Replacing" or "restoring" or "redo-ing" from the log is clearer.

But a redo (undo-the-undo) functionality may be a very practical very trivial extension of the undo-system.

"Pedantic LIFO":
“A foolish consistency is the hobgoblin of little minds"
-Ralph Waldo Emerson
Undo-actions cannot be considered by the overall system to be 'normal user actions' because if you simplistically created a new undo log for each undo action then you would be stuck in a loop: undoing and redoing one action (not helpful, and not a coherent definition of 'undo' (utterly useless, if technically logical).

But if (at the "router function" level) you created a 'redo-undo' log for the undo-action and put that in a ./changelog_redo_{file "name" without extension}}/ directory that small step having a second change_log directory should make a parallel redo functionality work. Both make-a-log-router and undo-next-LIFO-router functions take a directory, they doesn't care which one: this doesn't require any new functions or redo-flags.

It may be best if that is the end of the line, and not have another undo-redo-undo directory, etc.

One edge case is handling normal user actions mixed with undo-actions. The simplest way to handle this may be removing all redo-logs upon each normal-action. In ascii-times it would be fine to try redoing, but with multi-byte utf-8 splitting a character would be too risky (for most people).


Redo is not required, but implementation should not need to be a massive scope-increasing jump. Make another directory, empty the redo-logs. Only a slight tweak of undo-functionality.


# Rules
- no loading files
- no loading lines
- no third party crates (the basic_file_byte_operations crate I wrote for this, that is not a cargo 3rd party import)
(see more below)
- no hiding: no hidden files the user can't find, no binary files or secret file formats or compressions no one can read.


## Parts:
1. log file format
2. character to byte to character translation
- if the highest number has letter, keep going until you get to 'z' (the last)
3. character-level change logger (making a set of byte level logs to record a change)
4. character level re-do-er (processing a set of byte level logs to undo that change)

## Leveling:
- Character change log creation
- byte level change log files
- character level 'undo' change-log reverse processing.


## System
- LIFO Button Pancake Plate file stack. 'undo' undoes the most recent character level edit.
- one log directory per file (in same dir as file)
- one byte per log file
- logs generated one character add / one character add remove at a time. Or one hex edit at a time.
- sets of logs identified by the log names (see below)
- one 'undo' operate at a time: one character (or one hex edit) (there is no undo-all, no undo-session, no undo-party-time; there is one character-level (or one hex edit) undo)
- at least for now, since Rust is utf-8 based, all encoding is utf-8.
- logs are persistant, no auto-delete of logs
- Made for large files and file systems: Plan A: use u128 for file size. so max file size is MAX::u128 bytes?, this is part of a set of tools intended to handle large files (that software often cannot handle). (If 'counting' to very large numbers is an issue, there is a 'ribbon' system for counting on the fly without any upper-bound (beyond drive space). But u128 is probably big enough for now.)
- character-level (hex-edits aside): The change-character log process happens at the character level. If there isn't a valid utf-8 character to begin with, then there isn't a successful utf-8 character action to log (and to try to undo).
- use https://github.com/lineality/basic_file_byte_operations system for edit,add,remove with safe file handling and error management.
- use absolute paths: e.g. use std::fs::canonicalize() to convert all input paths to absolute paths before operating on them
- as per rules, all errors/exceptions are to be handled, nothing should crash the system. See "assert, catch-handle" specifications below.
- log file errors: log file errors, broken log files, etc. can go in a ./log_file_erros/ dir in the log dir.
- when writing a character back to file, it has to be a character, this is one of many places where there could be an error and the action cannot be undone (redone? undo'ed?).
- one log file for one byte does not mean you need to fully process that log on the fly. If you get a log with a letter (see more below) it makes sense to read the whole 1-3 more bytes and validate before carrying out the log-undo-ing.

- Functions should be accessible for, for example, a file-editor that can log character level changes: button_make_file_byte_edit_changelog(option(character),file_position,action_type)
button_undo_next_changelog_lifo(path_to_log_directory)

note: it is important that button-undo actions NOT be treated as standard actions, because then there would be a loop of undoing and redoing the same last undo (pedantic LIFO). Non-undo actions have change-logs made for them. But Button-Undo action must not have change-logs made for them. (This may connect to the topic of Redo...e.g. putting the change-log of an Undo in a 2nd log directory, redo-logs)

- Context: putting this all together, the use-case and user-needs become more clear. This system needs to be clear and reliable, modular, maintainable, not fast. It takes people an hour to write 1000 words. In that hour, there may be 5-10 undo actions, or none. If it took a whole second to process an undo, probably most people would not care as long as it worked. It would be difficult to design a system to process one byte slowly enough for people to notice the slowness. Making the system unmaintainable to optimize 1,000,000,000 undo actions (which is maybe more than have ever happened in the history of the world) would be completely missing the needs of the user: transparency, reliability, file-safety, maintainability, security, for a feature they will use rarely but care that it works and can be understood and inspected.

# File Structure
Functions etc. from basic_file_byte_operations module can be either called with mod or super mod, from basic_file_byte_operations.rs

## Error Handling & Validation
- https://github.com/lineality/basic_file_byte_operations: This module handles safe edits that will not alter the original file in the case of any failure. No 'recovery-reversal' is needed for an operation failure: move on (log if you can).
- button-undo-system error logs per file at
```
./undoredo_errorlogs_{file "name" without extension}}/ ?
```
e.g. each time there is an error-log to make:
1. make a (sub) directory for that log
./undoredo_errorlogs_{file "name" without extension}}/{timestamp}/
2. In that directory, put whatever error log files there are.
This way, file-name collisions are not a problem.

- each entry to the button-undo-system error logs should be a timestamped directory (with a timestamp probably sufficing for the directory name) in which are any files: error-log files, change-log files, etc.
- There are various ways that a UTF-8 character could be handled.
Because there are a maximum of 4 bytes, it is feasible to load all four bytes (I know, that's a lot of memory, right?) all four bytes into the Button-system to check if the character can be validated as a UTF-8 character. Then if there is an issue, a log file and the bytes (maybe in a dir) can be dumped into the error log dir. That may be easier than writing each byte and hoping we don't have to un-write them due to an error. If there is no problem, then Button-System can write the four bytes.
But there would need to be some kind of log-set state, to know how many bytes to try to undo and what those were. Either way they need to be stored, so probably measure-twice and cut once is best.

For a multi-byte character there are only a few things that can go wrong.
1. the resulting set of bytes isn't a character.
2. a log file got messed up and can't be read...which results in the same thing: no final byte.
3. If there isn't a byte to do something with, there's nothing to do. log-error what you can and move on without crashing.

The button-system follows the 'Assert, Catch-Handle' (see more details below) workflow for debugging and testing vs. production-release-builds.
Asserts and verbose error messages only exist in testing and debugging.
Production binaries must not contain testing and debugging code.
In production, errors are narrow and terse (no user or system information exposed for security) and the primary goal is that the system does not fail-crash-panic. When (not if) a log cannot be carried out, don't crash and move on. Where possible, write an error log about it.

Using structs, enums, etc., the structure and viability of the contents of a log file should be largely verifiable/validate-able.
- Does it have the needed sections?
- Do the section items have valid values?

Cases such as a file position not being found in the file itself must be handled (never crash) after looking at that file itself (not just inspecting the log).

If a log cannot be successfully processed (undone), it should be moved into a subdirectory in the error-logs directory with the error log (probably timestamped) if possible. It cannot remain in the change_log directory, obviously, or LIFO would keep trying it forever.


### log files: one byte per log-file
- data types: string, decimal, hex
- disambiguate the position-number by making it decimal, so it cannot be confused with the hx-byte (like struct-enums with different data types).

Human readable example
```
type = add/remove/edit # enum/string
file position = 123 # int ...ribbon?
byte ff # hex, remove has no byte
```
## Real file format (simpler)
```
add/rmv/edt <- always three letters
{int decimal} <- always decimal
FF <- always one hex byte (two nibbles)
```
e.g. add-byte
```
add
22
BF
```
e.g. remove-byte
```
rmv
23
```



### logs in same parent directory as file,
- one directory per file
- name: ./changlog_{file "name" without extension}/


## Buttons' Back to Front
There both for remove-bytes and for add-bytes for multi-byte characters, there are two approaches. One is more classically correct, the other seems like a cheap-trick but it may be better.

A. Last in, first out, you rebuild the string of bytes from the first letter (which is last on the stack).

B. The cheap-trick: You add all the bytes to the same (first) position, starting from the last character. The only position used for each log file is the insert-position at the beginning. Each time you add the next byte, it pushes the current byte down the line. The last byte you put in is the first one.


Let's take this Kanji:
- E9 98 BF (That looks delicious, I'll have that.)

If this is added, the reverse is just three removes.

If this is removed: the reverse is more buttony: three adds, starting with the first (closest to start) position, to reconstruct the character.

A. classical
```
1. 1.b add E9 @ 20(first byte first (last into stack))
2. 1.a add 98 @ 21
3. 1 add BF @ 22(last byte last (first into stack)
```
or
B. cheap trick
```
1. 1.b add BF @ 20(last byte (last into stack) handled first)
2. 1.a add 98 @ 20
3. 1   add E9 @ 20(first byte (first into stack) handled last)
```

The cheap trick involves less reversal-confusion and fewer moving parts.

The first byte, gets the first number, and goes into that stack first.
The later bytes get the same position as the first.
Much simpler!




Example 2: analogy, because Kanji is hard to understand (and Kanji sequence can be reversed anyway...)

This is only an analogy, because these are all 1-byte, not multi-byte.

48 65 6C 6C 6F 2C 20 77 6F 72 6C 64 21
H  e  l  l  o  ,  ⎕  w  o  r  l  d  !


Imagine that 'Hello, world!" is a big character of many bytes. (this only an analogy!)

Someone had typed, "H  e  l  l  o  ,  ⎕  w  o  r  l  d  !"
And then they deleted it.
But then, they wanted it back!
So they hit 'undo.'

To (classically) re-build the unit, the first part of the set must be LAST (LIFO) in the stack, with the highest-letter value.

The last character has no letter, and is at the bottom of the stack (relative to the other letters in the set).


If this is the sequence of bytes in Kanji:
```
E9 98 BF
```

1.b E9  (last in stack, first out, highest letter)
1.a 98
1   BF  (last part, first in stack, last out, lowest sort value.


The name determines the sort value of the record. So in theory, you could add the log files in any sequence (assuming you knew they would not be sorted by last-modified date-time).

If there are four bytes. The last byte gets the highest letter and goes in the stack first. The first byte in the kanji gets just a number, signaling to the system that there are no more bytes in that kanji.


But they need to be ordered backwards: CBA not ABC
because we need to find the last one first, and know which is the last in the set (no letter)

When sorted, we will get the later letter first, and the bare number last,
there should be some lite-weight process to see if it is just a letter.
is int?
split string on .
has extension
whatever is lite

```
if filename.find('.').is_some() {
    // Has letter suffix: "0.a"
} else {
    // Bare integer: "0"
}
Costs:
- No allocation: Pure stack-based
- Early exit: Stops at first . found (doesn't parse full string)
- Single pass: One linear scan
- Constant small overhead: Just pointer arithmetic
```

The only detail of the name that the system cares about is if there is a '.' in the name. The number does not matter, the letter does not matter. Those only exist for sorting: for which file is the 'next' to grab.

The only thing about the changelog file-name that matters is whether it is the last file for a multi-byte character or not: The last file will not have a '.', it will be a bare integer. The process stops at the last file (hence "last" file).

UTF-8 characters are 1-4 bytes (nothing bigger).

This is LIFO, last in, first out.
So you put the

The first byte must get the "highest" letter to b


One character gets one number.

Letters indicate a multi-byte character and the sequence of those bytes.

one-byte character -> one number (no letters)

multi-byte character -> a set of log files with the same number and letters indicating the sort-sequence for LIFO retrieval.


For remove-byte-edits:

If we are removing a multibyte kanji, this gets interesting, as there are maybe two different approaches.

A. we erase from back to front
B. You erase the front N times.

There may be advantages to the strange-button method of erasing the first character position 3 times.
- Fewer moving parts, fewer things to go wrong.

Another way to say this is that when making the log files, do you record all the positions, or do you record just the first character position N times?
I think N times is safer and easier to maintain.

E.g.
1. user deletes a Kanji
2. look at the character bytes: there are three
3. look at the positions
4. take the first position
5. make a set of three logs that all have the same position.


## Note: extra validation for RMV - remove-action
The remove-action does not 'need' to take an input, but if it has one, it can be used to validate what should be in the position that it is removing, as an extra safe-guard against removing a character or byte in the wrong place.


## Tests:
- Write, unwrite, rewrite a file.
- test multi-byte kanji
- test hex-edit, edit in place.

### About
I am making a text editor (that does not load files or lines or use heap memory) and I want to set up an undo(redo)/change-log-file system.

The idea is, if every change made can be recorded to be run in reverse, and that change saved in a file, then each file could be 'run backwards' (undo) to reverse that change (redo).

changelog files names are sequential numbers

There are three types of changes:
1. hex edits in place at a position
2. deleting bytes at a position
3. adding one or more characters from/at a file position.


This code is one flat file.

Each change-log is a separate sequence-numbered file 0 1 2 3 4 etc.
undo starts at the most recent change log file and reverses it.

Each character removed is broken into N byte units. (Buttons!)
- "If we button up again, we'll not be all undone."


===

# Notes:
1. do-one-thing-well functions are good, solid.
2. swiss-army-knife functions are bad, brittle, a liability.

Question: Which functionalities does it make sense to combine?
Which functions should stay simple and separate?

("redundancy" is a pedantic phobia, this is engineering, not insanity)

# Dev note step ideas:

### structs enums for items and datatype
- ByteValue: u8 (2-char hex string)
- FilePosition: u128 (decimal string in file)
- EditType: String enum add, rmv, edt enum: Add/Rmv/Edt (3-letter strings)
- LogfileName: integers with optional letter suffixes after dot

## helper functions
- function to clear redo_log directory
fn button_clear_all_redo_logs(redo_path_dir)


## one byte char
- function to make log file for single byte char remove
fn button_remove_byte_make_log_file(log_path:path(pathbuf?), edit_file_position(u128), log_directory_path:path(pathbuf?))

- function to make log file for single byte char add
fn button_add_byte_make_log_file(log_path:path(pathbuf?), edit_file_position(u128), byte_hex, log_directory_path:path(pathbuf?))

- function to make log file for single byte (hex)edit in place
fn button_hexeditinplace_byte_make_log_file(log_path:path(pathbuf?), edit_file_position(u128), byte_hex, log_directory_path:path(pathbuf?))


## multi-byte char
- function to make log file for multi-byte char remove
fn button_remove_mulitbyte_make_log_file(log_path:path(pathbuf?), edit_file_position(u128), log_directory_path:path(pathbuf?))


- function to make log file for multi-byte char add
fn button_add_mulitbyte_make_log_file(log_path:path(pathbuf?), edit_file_position(u128), byte_hex, log_directory_path:path(pathbuf?))



- router-function to direct character level action to make log file
fn button_make_character_action_changelog(option(character),file_position,action_type, log_directory_path:path(pathbuf?)) (note: undo or redo is determined by log_directory_path)


- one function for hex edit make log.
fn button_make_hexedit_changelog(character,file_position)

- function to undo log file for single byte char remove
- function to undo log file for single byte char add
- function to undo log file for single byte (hex)edit in place

- function to undo log file for multi-byte char remove
- function to undo log file for multi-byte char add
- function to undo log file for multi-byte (hex)edit in place

- router-function to direct character level action to undo log file:
fn button_undo_next_changelog_lifo_router(path_to_log_directory)


# integration Notes:

The first use of this Button-Undo module is in the Lines-Editor.

Button-System will have its own error handling, but integration be done into the host-project by adding an error impl (example in code).
](Buttons-Undo-Redo: A Reversible File-byte-edit Changelog System
"If we button up again, we'll not be all undone."
2025/10/30

# Project: reversible_edit_changelog.rs module
- Rust Language
- goal: write & read config files to byte operations.
```
e.g.
// a file-editor that can log character level changes:

fn button_make_character_action_changelog(option(character),file_position,action_type, log_directory_path:path(pathbuf?)) (note: undo or redo is determined by log_directory_path)

fn button_make_hexedit_changelog(character,file_position)

// a file-editor that can undo the LIFO change:
fn button_undo_next_changelog_lifo(path_to_log_directory)

```
- Base: https://github.com/lineality/basic_file_byte_operations


# overall

### Walkthrough:
1. The user successfully makes a change in a file (adding a character, removing a character, or hex-editing a byte in place).
2. An inverse-change-log (instructions for reversing the user's action) are recorded in a directory (same as the file)
```path
./changelog_{filename without dot in suffix/extension}/
```
3. The user can 'undo' that action by using:
```rust
button_undo_next_changelog_lifo(&file_path, &undo_log_directory)
```
Three things happen:
- the inverse-change-log (instructions for reversing the user's action) is used to undo what the user-did.
- That log is 'popped off the LIFO stack' (deleted)
- A new inverse-change-log (instructions for reversing the undoing action (to undo the undo (to 'redo')) is made in the
```path
./changelog_redo_{filename without dot in suffix/extension}/

```
4. The user can then redo what they undid by calling (the same function)
```rust
button_undo_next_changelog_lifo(&file_path, &redo_log_directory)
```
but pointing at the redo-log stack.

The function recognizes the "changelog_redo_{}" path, and does NOT make another log about this action (so no circular loop).

See a more detailed walkthrough in appendix 1.


### About

A change-log is the opposite of what the user did on a character level (add or remove a character).
The "change log" is instructions to undo/redo what the user did/undid.
If the user adds "hello"
the change-logs are:

remove h position
remove e position
remove l position
remove l position
remove o position

It does not matter (per se) what character is there (unless you are double checking).

If the user deletes "hello"

the change-logs are:

at position add o
at position add l
at position add l
at position add e
at position add h


If the user hex-edits a byte to become hex-61 but was FF.

The change-logs is:
at position, hex-edit FF.



## Disambiguation: "undo" vs. "redo": The term 'redo' is sometimes used to describe re-creating something that was deleted. "Un-replacing," "unplacing?" is not clear terminology. "Replacing" or "restoring" or "redo-ing" from the log is clearer.

But a redo (undo-the-undo) functionality may be a very practical very trivial extension of the undo-system.

"Pedantic LIFO":
“A foolish consistency is the hobgoblin of little minds"
-Ralph Waldo Emerson
Undo-actions cannot be considered by the overall system to be 'normal user actions' because if you simplistically created a new undo log for each undo action then you would be stuck in a loop: undoing and redoing one action (not helpful, and not a coherent definition of 'undo' (utterly useless, if technically logical).

But if (at the "router function" level) you created a 'redo-undo' log for the undo-action and put that in a ./changelog_redo_{filename without dot in suffix/extension}}/ directory that small step having a second change_log directory should make a parallel redo functionality work. Both make-a-log-router and undo-next-LIFO-router functions take a directory, they doesn't care which one: this doesn't require any new functions or redo-flags.

It may be best if that is the end of the line, and not have another undo-redo-undo directory, etc.

One edge case is handling normal user actions mixed with undo-actions. The simplest way to handle this may be removing all redo-logs upon each normal-action. In ascii-times it would be fine to try redoing, but with multi-byte utf-8 splitting a character would be too risky (for most people).


Redo is not required, but implementation should not need to be a massive scope-increasing jump. Make another directory, empty the redo-logs. Only a slight tweak of undo-functionality.


# Rules
- no loading files
- no loading lines
- no third party crates (the basic_file_byte_operations crate I wrote for this, that is not a cargo 3rd party import)
(see more below)
- no hiding: no hidden files the user can't find, no binary files or secret file formats or compressions no one can read.


## Parts:
1. log file format
2. character to byte to character translation
- if the highest number has letter, keep going until you get to 'z' (the last)
3. character-level change logger (making a set of byte level logs to record a change)
4. character level re-do-er (processing a set of byte level logs to undo that change)

## Leveling:
- Character change log creation
- byte level change log files
- character level 'undo' change-log reverse processing.


## System
- LIFO Button Pancake Plate file stack. 'undo' undoes the most recent character level edit.
- one log directory per file (in same dir as file)
- one byte per log file
- logs generated one character add / one character add remove at a time. Or one hex edit at a time.
- sets of logs identified by the log names (see below)
- one 'undo' operate at a time: one character (or one hex edit) (there is no undo-all, no undo-session, no undo-party-time; there is one character-level (or one hex edit) undo)
- at least for now, since Rust is utf-8 based, all encoding is utf-8.
- logs are persistant, no auto-delete of logs
- Made for large files and file systems: Plan A: use u128 for file size. so max file size is MAX::u128 bytes?, this is part of a set of tools intended to handle large files (that software often cannot handle). (If 'counting' to very large numbers is an issue, there is a 'ribbon' system for counting on the fly without any upper-bound (beyond drive space). But u128 is probably big enough for now.)
- character-level (hex-edits aside): The change-character log process happens at the character level. If there isn't a valid utf-8 character to begin with, then there isn't a successful utf-8 character action to log (and to try to undo).
- use https://github.com/lineality/basic_file_byte_operations system for edit,add,remove with safe file handling and error management.
- use absolute paths: e.g. use std::fs::canonicalize() to convert all input paths to absolute paths before operating on them
- as per rules, all errors/exceptions are to be handled, nothing should crash the system. See "assert, catch-handle" specifications below.
- log file errors: log file errors, broken log files, etc. can go in a ./log_file_erros/ dir in the log dir.
- when writing a character back to file, it has to be a character, this is one of many places where there could be an error and the action cannot be undone (redone? undo'ed?).
- one log file for one byte does not mean you need to fully process that log on the fly. If you get a log with a letter (see more below) it makes sense to read the whole 1-3 more bytes and validate before carrying out the log-undo-ing.

- Functions should be accessible for, for example, a file-editor that can log character level changes: button_make_file_byte_edit_changelog(option(character),file_position,action_type)
button_undo_next_changelog_lifo(path_to_log_directory)

note: it is important that button-undo actions NOT be treated as standard actions, because then there would be a loop of undoing and redoing the same last undo (pedantic LIFO). Non-undo actions have change-logs made for them. But Button-Undo action must not have change-logs made for them. (This may connect to the topic of Redo...e.g. putting the change-log of an Undo in a 2nd log directory, redo-logs)

- Context: putting this all together, the use-case and user-needs become more clear. This system needs to be clear and reliable, modular, maintainable, not fast. It takes people an hour to write 1000 words. In that hour, there may be 5-10 undo actions, or none. If it took a whole second to process an undo, probably most people would not care as long as it worked. It would be difficult to design a system to process one byte slowly enough for people to notice the slowness. Making the system unmaintainable to optimize 1,000,000,000 undo actions (which is maybe more than have ever happened in the history of the world) would be completely missing the needs of the user: transparency, reliability, file-safety, maintainability, security, for a feature they will use rarely but care that it works and can be understood and inspected.

# File Structure
Functions etc. from basic_file_byte_operations module can be either called with mod or super mod, from basic_file_byte_operations.rs

## Error Handling & Validation
- https://github.com/lineality/basic_file_byte_operations: This module handles safe edits that will not alter the original file in the case of any failure. No 'recovery-reversal' is needed for an operation failure: move on (log if you can).
- button-undo-system error logs per file at
```
./undoredo_errorlogs_{filename without dot in suffix/extension}}/ ?
```
e.g. each time there is an error-log to make:
1. make a (sub) directory for that log
./undoredo_errorlogs_{filename without dot in suffix/extension}}/{timestamp}/
2. In that directory, put whatever error log files there are.
This way, file-name collisions are not a problem.

- each entry to the button-undo-system error logs should be a timestamped directory (with a timestamp probably sufficing for the directory name) in which are any files: error-log files, change-log files, etc.
- There are various ways that a UTF-8 character could be handled.
Because there are a maximum of 4 bytes, it is feasible to load all four bytes (I know, that's a lot of memory, right?) all four bytes into the Button-system to check if the character can be validated as a UTF-8 character. Then if there is an issue, a log file and the bytes (maybe in a dir) can be dumped into the error log dir. That may be easier than writing each byte and hoping we don't have to un-write them due to an error. If there is no problem, then Button-System can write the four bytes.
But there would need to be some kind of log-set state, to know how many bytes to try to undo and what those were. Either way they need to be stored, so probably measure-twice and cut once is best.

For a multi-byte character there are only a few things that can go wrong.
1. the resulting set of bytes isn't a character.
2. a log file got messed up and can't be read...which results in the same thing: no final byte.
3. If there isn't a byte to do something with, there's nothing to do. log-error what you can and move on without crashing.

The button-system follows the 'Assert, Catch-Handle' (see more details below) workflow for debugging and testing vs. production-release-builds.
Asserts and verbose error messages only exist in testing and debugging.
Production binaries must not contain testing and debugging code.
In production, errors are narrow and terse (no user or system information exposed for security) and the primary goal is that the system does not fail-crash-panic. When (not if) a log cannot be carried out, don't crash and move on. Where possible, write an error log about it.

Using structs, enums, etc., the structure and viability of the contents of a log file should be largely verifiable/validate-able.
- Does it have the needed sections?
- Do the section items have valid values?

Cases such as a file position not being found in the file itself must be handled (never crash) after looking at that file itself (not just inspecting the log).

If a log cannot be successfully processed (undone), it should be moved into a subdirectory in the error-logs directory with the error log (probably timestamped) if possible. It cannot remain in the change_log directory, obviously, or LIFO would keep trying it forever.


### log files: one byte per log-file
- data types: string, decimal, hex
- disambiguate the position-number by making it decimal, so it cannot be confused with the hx-byte (like struct-enums with different data types).

Human readable example
```
type = add/remove/edit # enum/string
file position = 123 # int ...ribbon?
byte ff # hex, remove has no byte
```
## Real file format (simpler)
```
add/rmv/edt <- always three letters
{int decimal} <- always decimal
FF <- always one hex byte (two nibbles)
```
e.g. add-byte
```
add
22
BF
```
e.g. remove-byte
```
rmv
23
```



### logs in same parent directory as file,
- one directory per file
- name: ./changlog_{filename without dot in suffix/extension}/


## Buttons' Back to Front
There both for remove-bytes and for add-bytes for multi-byte characters, there are two approaches. One is more classically correct, the other seems like a cheap-trick but it may be better.

A. Last in, first out, you rebuild the string of bytes from the first letter (which is last on the stack).

B. The cheap-trick: You add all the bytes to the same (first) position, starting from the last character. The only position used for each log file is the insert-position at the beginning. Each time you add the next byte, it pushes the current byte down the line. The last byte you put in is the first one.


Let's take this Kanji:
- E9 98 BF (That looks delicious, I'll have that.)

If this is added, the reverse is just three removes.

If this is removed: the reverse is more buttony: three adds, starting with the first (closest to start) position, to reconstruct the character.

A. classical
```
1. 1.b add E9 @ 20(first byte first (last into stack))
2. 1.a add 98 @ 21
3. 1 add BF @ 22(last byte last (first into stack)
```
or
B. cheap trick
```
1. 1.b add BF @ 20(last byte (last into stack) handled first)
2. 1.a add 98 @ 20
3. 1   add E9 @ 20(first byte (first into stack) handled last)
```

The cheap trick involves less reversal-confusion and fewer moving parts.

The first byte, gets the first number, and goes into that stack first.
The later bytes get the same position as the first.
Much simpler!




Example 2: analogy, because Kanji is hard to understand (and Kanji sequence can be reversed anyway...)

This is only an analogy, because these are all 1-byte, not multi-byte.

48 65 6C 6C 6F 2C 20 77 6F 72 6C 64 21
H  e  l  l  o  ,  ⎕  w  o  r  l  d  !


Imagine that 'Hello, world!" is a big character of many bytes. (this only an analogy!)

Someone had typed, "H  e  l  l  o  ,  ⎕  w  o  r  l  d  !"
And then they deleted it.
But then, they wanted it back!
So they hit 'undo.'

To (classically) re-build the unit, the first part of the set must be LAST (LIFO) in the stack, with the highest-letter value.

The last character has no letter, and is at the bottom of the stack (relative to the other letters in the set).


If this is the sequence of bytes in Kanji:
```
E9 98 BF
```

1.b E9  (last in stack, first out, highest letter)
1.a 98
1   BF  (last part, first in stack, last out, lowest sort value.


The name determines the sort value of the record. So in theory, you could add the log files in any sequence (assuming you knew they would not be sorted by last-modified date-time).

If there are four bytes. The last byte gets the highest letter and goes in the stack first. The first byte in the kanji gets just a number, signaling to the system that there are no more bytes in that kanji.


But they need to be ordered backwards: CBA not ABC
because we need to find the last one first, and know which is the last in the set (no letter)

When sorted, we will get the later letter first, and the bare number last,
there should be some lite-weight process to see if it is just a letter.
is int?
split string on .
has extension
whatever is lite

```
if filename.find('.').is_some() {
    // Has letter suffix: "0.a"
} else {
    // Bare integer: "0"
}
Costs:
- No allocation: Pure stack-based
- Early exit: Stops at first . found (doesn't parse full string)
- Single pass: One linear scan
- Constant small overhead: Just pointer arithmetic
```

The only detail of the name that the system cares about is if there is a '.' in the name. The number does not matter, the letter does not matter. Those only exist for sorting: for which file is the 'next' to grab.

The only thing about the changelog file-name that matters is whether it is the last file for a multi-byte character or not: The last file will not have a '.', it will be a bare integer. The process stops at the last file (hence "last" file).

UTF-8 characters are 1-4 bytes (nothing bigger).

This is LIFO, last in, first out.
So you put the

The first byte must get the "highest" letter to b


One character gets one number.

Letters indicate a multi-byte character and the sequence of those bytes.

one-byte character -> one number (no letters)

multi-byte character -> a set of log files with the same number and letters indicating the sort-sequence for LIFO retrieval.


For remove-byte-edits:

If we are removing a multibyte kanji, this gets interesting, as there are maybe two different approaches.

A. we erase from back to front
B. You erase the front N times.

There may be advantages to the strange-button method of erasing the first character position 3 times.
- Fewer moving parts, fewer things to go wrong.

Another way to say this is that when making the log files, do you record all the positions, or do you record just the first character position N times?
I think N times is safer and easier to maintain.

E.g.
1. user deletes a Kanji
2. look at the character bytes: there are three
3. look at the positions
4. take the first position
5. make a set of three logs that all have the same position.


## Note: extra validation for RMV - remove-action
The remove-action does not 'need' to take an input, but if it has one, it can be used to validate what should be in the position that it is removing, as an extra safe-guard against removing a character or byte in the wrong place.


# Example functions (some details may have changed)

### structs enums for items and datatype
- ByteValue: u8 (2-char hex string)
- FilePosition: u128 (decimal string in file)
- EditType: String enum add, rmv, edt enum: Add/Rmv/Edt (3-letter strings)
- LogfileName: integers with optional letter suffixes after dot

## helper functions
- function to clear redo_log directory
fn button_clear_all_redo_logs(redo_path_dir)


## one byte char
- function to make log file for single byte char remove
fn button_remove_byte_make_log_file(log_path:path(pathbuf?), edit_file_position(u128), log_directory_path:path(pathbuf?))

- function to make log file for single byte char add
fn button_add_byte_make_log_file(log_path:path(pathbuf?), edit_file_position(u128), byte_hex, log_directory_path:path(pathbuf?))

- function to make log file for single byte (hex)edit in place
fn button_hexeditinplace_byte_make_log_file(log_path:path(pathbuf?), edit_file_position(u128), byte_hex, log_directory_path:path(pathbuf?))


## multi-byte char
- function to make log file for multi-byte char remove
fn button_remove_mulitbyte_make_log_file(log_path:path(pathbuf?), edit_file_position(u128), log_directory_path:path(pathbuf?))


- function to make log file for multi-byte char add
fn button_add_mulitbyte_make_log_file(log_path:path(pathbuf?), edit_file_position(u128), byte_hex, log_directory_path:path(pathbuf?))



- router-function to direct character level action to make log file
fn button_make_character_action_changelog(option(character),file_position,action_type, log_directory_path:path(pathbuf?)) (note: undo or redo is determined by log_directory_path)



- one function for hex edit make log.
fn button_make_hexedit_changelog(character,file_position)

- function to undo log file for single byte char remove
- function to undo log file for single byte char add
- function to undo log file for single byte (hex)edit in place

- function to undo log file for multi-byte char remove
- function to undo log file for multi-byte char add
- function to undo log file for multi-byte (hex)edit in place

- router-function to direct character level action to undo log file:
fn button_undo_next_changelog_lifo_router(path_to_log_directory)


# error handling integration Notes:

Here is an example of importing the Button's error system into the host error system:


```
/// Automatic conversion from ToggleCommentError to LinesError
impl From<ToggleCommentError> for LinesError {
    fn from(err: ToggleCommentError) -> Self {
        // Map ToggleCommentError variants to appropriate LinesError categories
        match err {
            ToggleCommentError::FileNotFound
            | ToggleCommentError::NoExtension
            | ToggleCommentError::UnsupportedExtension => {
                LinesError::InvalidInput(err.to_string())
            }
            ToggleCommentError::LineNotFound { .. } => LinesError::InvalidInput(err.to_string()),
            ToggleCommentError::IoError(_) => {
                LinesError::Io(io::Error::new(io::ErrorKind::Other, err.to_string()))
            }
            ToggleCommentError::PathError => LinesError::StateError(err.to_string()),
            ToggleCommentError::LineTooLong { .. } => LinesError::InvalidInput(err.to_string()),
            ToggleCommentError::InconsistentBlockMarkers => {
                LinesError::StateError(err.to_string())
            }
            ToggleCommentError::RangeTooLarge { .. } => LinesError::InvalidInput(err.to_string()),
        }
    }
}

/// Automatic conversion from ToggleIndentError to LinesError
impl From<ToggleIndentError> for LinesError {
    fn from(err: ToggleIndentError) -> Self {
        match err {
            ToggleIndentError::FileNotFound => LinesError::InvalidInput(err.to_string()),
            ToggleIndentError::LineNotFound { .. } => LinesError::InvalidInput(err.to_string()),
            ToggleIndentError::IoError(_) => {
                LinesError::Io(io::Error::new(io::ErrorKind::Other, err.to_string()))
            }
            ToggleIndentError::PathError => LinesError::StateError(err.to_string()),
            ToggleIndentError::LineTooLong { .. } => LinesError::InvalidInput(err.to_string()),
        }
    }
}
```


# Appendix 1: Demo Example

From main.rs:
```rust
   // =========================================================================
    // MANUAL TEST: Interactive Walkthrough
    // =========================================================================
    println!("─────────────────────────────────────────────────────────────");
    println!("MANUAL TEST: Interactive Undo/Redo Walkthrough");
    println!("─────────────────────────────────────────────────────────────");
    println!();

    let manual_test_file = test_dir.join("manual_test.txt");
    let manual_log_dir = test_dir.join("changelog_manual_testtxt");
    let manual_redo_dir = test_dir.join("changelog_redo_manual_testtxt");

    // =========================================
    // Step 1: Create empty file
    // =========================================
    println!("STEP 1: Starting with EMPTY FILE");
    println!("─────────────────────────────────────────────────────────────");
    fs::write(&manual_test_file, b"")?;
    println!("File: {}", manual_test_file.display());
    println!("Content: (empty)");
    println!("File size: 0 bytes");
    println!();
    println!("Press ENTER to continue...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    println!();

    // =========================================
    // Step 2: User adds 'a' (log says remove)
    // =========================================
    println!("STEP 2: USER ADDS CHARACTER 'a'");
    println!("─────────────────────────────────────────────────────────────");
    fs::write(&manual_test_file, b"a")?;
    println!("File content: 'a'");
    println!("File size: 1 byte");
    println!();

    println!("Creating changelog: RMV at position 0");
    button_remove_byte_make_log_file(&fs::canonicalize(&manual_test_file)?, 0, &manual_log_dir)
        .expect("Failed to create log");
    println!("✓ Changelog created in: {}", manual_log_dir.display());
    println!();
    println!("Press ENTER to continue...");
    std::io::stdin().read_line(&mut input)?;
    println!();

    // =========================================
    // Step 3: User performs UNDO
    // =========================================
    println!("STEP 3: USER PERFORMS UNDO");
    println!("─────────────────────────────────────────────────────────────");
    println!("Executing: button_undo_redo_next_inverse_changelog_pop_lifo()");
    button_undo_redo_next_inverse_changelog_pop_lifo(&manual_test_file, &manual_log_dir)
        .expect("Failed to undo");
    println!("✓ Undo operation completed");
    println!();

    let undo_result = fs::read_to_string(&manual_test_file)?;
    println!("File content after undo: {:?}", undo_result);
    println!("File size: {} bytes", undo_result.len());
    println!();

    if undo_result.is_empty() {
        println!("✅ CORRECT: 'a' was removed (file is empty again)");
    } else {
        println!(
            "❌ ERROR: File should be empty but contains: {:?}",
            undo_result
        );
    }
    println!();
    println!("Notice: Redo logs were automatically created in:");
    println!("{}", manual_redo_dir.display());
    println!();
    println!("Press ENTER to continue...");
    std::io::stdin().read_line(&mut input)?;
    println!();

    // =========================================
    // Step 4: User performs REDO
    // =========================================
    println!("STEP 4: USER PERFORMS REDO");
    println!("─────────────────────────────────────────────────────────────");
    println!("Executing: button_undo_redo_next_inverse_changelog_pop_lifo() with REDO directory");
    button_undo_redo_next_inverse_changelog_pop_lifo(&manual_test_file, &manual_redo_dir)
        .expect("Failed to redo");
    println!("✓ Redo operation completed");
    println!();

    let redo_result = fs::read_to_string(&manual_test_file)?;
    println!("File content after redo: {:?}", redo_result);
    println!("File size: {} bytes", redo_result.len());
    println!();

    if redo_result == "a" {
        println!("✅ CORRECT: 'a' was restored (file contains 'a' again)");
    } else {
        println!(
            "❌ ERROR: File should contain 'a' but contains: {:?}",
            redo_result
        );
    }
    println!();
    println!("Notice: The system automatically detected the redo directory");
    println!("and did NOT create another redo log (prevents infinite loops)");
    println!();
    println!("Press ENTER to continue...");
    std::io::stdin().read_line(&mut input)?;
    println!();

    // =========================================
    // Step 5: User makes NEW edit (clears redo)
    // =========================================
    println!("STEP 5: USER MAKES NEW EDIT (adds 'b')");
    println!("─────────────────────────────────────────────────────────────");
    fs::write(&manual_test_file, b"ab")?;
    println!("File content: 'ab'");
    println!();

    println!("Creating new changelog: RMV at position 1 for 'b'");
    button_remove_byte_make_log_file(&fs::canonicalize(&manual_test_file)?, 1, &manual_log_dir)
        .expect("Failed to create log");
    println!("✓ New changelog created");
    println!();

    println!("Clearing redo logs (new edit invalidates redo history)");
    _ = button_clear_all_redo_logs(&manual_test_file);
    println!("✓ Redo logs cleared");
    println!();
    println!("Notice: The redo directory is now empty");
    println!("This is crucial: after a new edit, you can't redo the old 'a' anymore");
    println!();
    println!("Press ENTER to continue...");
    std::io::stdin().read_line(&mut input)?;
    println!();

    // =========================================
    // Step 6: Try to redo (should fail - no logs)
    // =========================================
    println!("STEP 6: ATTEMPT TO REDO (should fail - no logs)");
    println!("─────────────────────────────────────────────────────────────");
    println!("Attempting: button_undo_redo_next_inverse_changelog_pop_lifo() with REDO directory");

    match button_undo_redo_next_inverse_changelog_pop_lifo(&manual_test_file, &manual_redo_dir) {
        Ok(_) => {
            println!("❌ ERROR: Should have failed (no redo logs)");
        }
        Err(e) => {
            println!("✓ Operation failed as expected");
            println!("Error: {}", e);
            println!();
            println!("✅ CORRECT: Cannot redo because redo logs were cleared");
        }
    }
    println!();
    println!("Press ENTER to continue...");
    std::io::stdin().read_line(&mut input)?;
    println!();

    // =========================================
    // Step 7: Undo the new 'b' addition
    // =========================================
    println!("STEP 7: UNDO THE NEW 'b' ADDITION");
    println!("─────────────────────────────────────────────────────────────");
    println!("File before undo: 'ab'");
    button_undo_redo_next_inverse_changelog_pop_lifo(&manual_test_file, &manual_log_dir)
        .expect("Failed to undo");

    let final_result = fs::read_to_string(&manual_test_file)?;
    println!("File after undo: {:?}", final_result);
    println!();

    if final_result == "a" {
        println!("✅ CORRECT: Back to 'a' (only 'b' was removed)");
    }
    println!();

    println!();
    println!("Press ENTER to remove test files...");
    std::io::stdin().read_line(&mut input)?;
    println!();

    // Cleanup
    let _ = fs::remove_file(&manual_test_file);
    let _ = fs::remove_dir_all(&manual_log_dir);
    let _ = fs::remove_dir_all(&manual_redo_dir);

    println!("─────────────────────────────────────────────────────────────");
    println!("MANUAL TEST COMPLETE");
    println!("─────────────────────────────────────────────────────────────");
    println!();

```
)
