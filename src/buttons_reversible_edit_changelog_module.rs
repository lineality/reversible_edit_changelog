//! buttons_reversible_edit_changelog_module.rs
//! Buttons: Reversible Character and Byte Undo Redo Changelog System
//!
//! A transparent, file-based undo system for byte-level file edits.
//! Creates human-readable changelog files (one byte per file) that can be
//! processed in LIFO order to undo character-level changes.

use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};
/*
Rules & Policies

# Rust rules:
- Always best practice.
- Always extensive doc strings.
- Always comments.
- Always cargo tests (where possible).
- Never remove documentation.
- Always clear, meaningful, unique names (e.g. variables, functions).
- Always absolute file paths.
- Always error handling.
- Never unsafe code.
- Never use unwrap.

- Load what is needed when it is needed: Do not ever load a whole file or line, rarely load a whole anything. increment and load only what is required pragmatically. Do not fill 'state' with every possible piece of un-used information. Do not insecurity output information broadly in the case of errors and exceptions.

- Always defensive best practice
- Always error handling: Every part of code, every process, function, and operation will fail at some point, if only because of cosmic-ray bit-flips (which are common), hardware failure, power-supply failure, adversarial attacks, etc. There must always be fail-safe error handling where production-release-build code handles issues and moves on without panic-crashing ever. Every failure must be handled smoothly: let it fail and move on.


Safety, reliability, maintainability, fail-safe, communication-documentation, are the goals: not ideology, aesthetics, popularity, momentum-tradition, bad habits, convenience, nihilism, lazyness, lack of impulse control, etc.

## No third party libraries (or very strictly avoid third party libraries where possible).

## Rule of Thumb, ideals not absolute rules: Follow NASA's 'Power of 10 rules' where possible and sensible (as updated for 2025 and Rust (not narrowly 2006 c for embedded systems):
1. no unsafe stuff:
- no recursion
- no goto
- no pointers
- no preprocessor

2. upper bound on all normal-loops, failsafe for all always-loops

3. Pre-allocate all memory (no dynamic memory allocation)

4. Clear function scope and Data Ownership: Part of having a function be 'focused' means knowing if the function is in scope. Functions should be neither swiss-army-knife functions that do too many things, nor scope-less micro-functions that may be doing something that should not be done. Many functions should have a narrow focus and a short length, but definition of actual-project scope functionality must be explicit. Replacing one long clear in-scope function with 50 scope-agnostic generic sub-functions with no clear way of telling if they are in scope or how they interact (e.g. hidden indirect recursion) is unsafe. Rust's ownership and borrowing rules focus on Data ownership and hidden dependencies, making it even less appropriate to scatter borrowing and ownership over a spray of microfunctions purely for the ideology of turning every operation into a microfunction just for the sake of doing so. (See more in rule 9.)

5. Defensive programming: debug-assert, test-assert, prod safely check & handle, not 'assert!' panic
For production-release code:
1. check and handle without panic/halt in production
2. return result (such as Result<T, E>) and smoothly handle errors (not halt-panic stopping the application): no assert!() outside of test-only code
3. test assert: use #[cfg(test)] assert!() to test production binaries (not in prod)
4. debug assert: use debug_assert to test debug builds/runs (not in prod)
5. use defensive programming with recovery of all issues at all times
- use cargo tests
- use debug_asserts
- do not leave assertions in production code.
- use no-panic error handling
- use Option
- use enums and structs
- check bounds
- check returns
- note: a test-flagged assert can test a production release build (whereas debug_assert cannot); cargo test --release
```
#[cfg(test)]
assert!(
```

e.g.
# "Assert & Catch-Handle" 3-part System

// template/example for check/assert format
//    =================================================
// // Debug-Assert, Test-Asset, Production-Catch-Handle
//    =================================================
// This is not included in production builds
// assert: only when running in a debug-build: will panic
debug_assert!(
    INFOBAR_MESSAGE_BUFFER_SIZE > 0,
    "Info bar buffer must have non-zero capacity"
);
// This is not included in production builds
// assert: only when running cargo test: will panic
#[cfg(test)]
assert!(
    INFOBAR_MESSAGE_BUFFER_SIZE > 0,
    "Info bar buffer must have non-zero capacity"
);
// Catch & Handle without panic in production
// This IS included in production to safe-catch
if !INFOBAR_MESSAGE_BUFFER_SIZE == 0 {
    // state.set_info_bar_message("Config error");
    return Err(LinesError::GeneralAssertionCatchViolation(
        "zero buffer size error".into(),
    ));
}


Avoid heap for error messages and for all things:
Is heap used for error messages because that is THE best way, the most secure, the most efficient, proper separate of debug testing vs. secure production code?
Or is heap used because of oversights and apathy: "it's future dev's problem, let's party."
We can use heap in debug/test modes/builds only.
Production software must not insecurely output debug diagnostics.
Debug information must not be included in production builds: "developers accidentally left development code in the software" is a classic error (not a desired design spec) that routinely leads to security and other issues. That is NOT supposed to happen. It is not coherent to insist the open ended heap output 'must' or 'should' be in a production build.

This is central to the question about testing vs. a pedantic ban on conditional compilation; not putting full traceback insecurity into production code is not a different operational process logic tree for process operations.

Just like with the pedantic "all loops being bounded" rule, there is a fundamental exception: always-on loops must be the opposite.
With conditional compilations: code NEVER to EVER be in production-builds MUST be always "conditionally" excluded. This is not an OS conditional compilation or a hardware conditional compilation. This is an 'unsafe-testing-only or safe-production-code' condition.

Error messages and error outcomes in 'production' 'release' (real-use, not debug/testing) must not ever contain any information that could be a security vulnerability or attack surface. Failing to remove debugging inspection is a major category of security and hygiene problems.

Security: Error messages in production must NOT contain:
- File paths (can reveal system structure)
- File contents
- environment variables
- user, file, state, data
- internal implementation details
- etc.

All debug-prints not for production must be tagged with
```
#[cfg(debug_assertions)]
```

Production output following an error must be managed and defined, not not open to whatever an api or OS-call wants to dump out.

6. Manage ownership and borrowing

7. Manage return values:
- use null-void return values
- check non-void-null returns

8. Navigate debugging and testing on the one hand and not-dangerous conditional compilation on the other hand

9. Communicate:
- use doc strings, use comments,
- Document use-cases, edge-cases, and policies (These are project specific and cannot be telepathed from generic micro-function code. When a Mars satellite failed because one team used SI-metric units and another team did not, that problem could not have been detected by looking at, and auditing, any individual function in isolation without documentation. Breaking a process into innumerable undocumented micro-functions can make scope and policy impossible to track. To paraphrase Jack Welch: "The most dangerous thing in the world is a flawless operation that should never have been done in the first place.")

10. Use state-less operations when possible:
- a seemingly invisibly small increase in state often completely destroys projects
- expanding state destroys projects with unmaintainable over-reach

Vigilance: We should help support users and developers and the people who depend upon maintainable software. Maintainable code supports the future for us all.

 */

/*
 * Uses:
 * https://github.com/lineality/timestamps_rust_vanilla
 * https://github.com/lineality/basic_file_byte_operations
 */

// //! basic_file_byte_operations
// use std::{
//     fs::{self, File, OpenOptions},
//     io::{self, Read, Seek, SeekFrom, Write},
//     path::{Path, PathBuf},
// };
/*

# File Identities & Workflow
At the granular level of these operations, it may be best to avoid user-abstractions such as 'add' or 'remove' or 'modify' and 'original' or 'copy' when speaking of the actual mechanical steps. We should to look instead at specific well-defined steps and actions. The semantics may seem counter-intuitive, as to effect the same result we never make any changes to either the original file (preserved for safety) or to the new file, which we describe as 'altered' meaning that it is different from the original, not that 'change operations' were ever performed on the file as such. For example reconstructing a file after frameshifting does not ever literally happen (as it would need to if there were only one file without a backup).

It may be possible to effect the desired end-state (retroactively described as 'add' or 'remove') with steps such as these:

1. Create a draft file.

2. Append bytes (from the original file, to the draft-file) up to the 'file byte position of the change operation' in question:
append byte by byte, or append with a small bucket-brigade buffer.

3. Performing Operation at 'file byte position of the change operation':
- For Remove-a-byte-operation: no action taken for draft-file, nothing written. This is an effective frame shift/advance in reading the original file one byte.
- For Add-a-byte-operation: append the 'new' (not in original file) byte to the draft file. Do not shift original file read-location.
- For Hex-edit: append the 'new' (not in original file) byte to the draft file.

4. Performing Operation ~after 'file byte position of the change operation':
- For hex-edit: Append bytes (from the original file, to the draft-file) after the 'file byte position of the change operation' in question:
append byte by byte, or append with a small bucket-brigade buffer.
- For remove-byte: Append bytes (from the original file, to the draft-file), after the 'file byte position of the change operation' in question: append byte by byte, or append with a small bucket-brigade buffer. This is similar to hex-edit, except that nothing is added AT the target position, effecting a frame-shift.
- For Add-byte Edit: Append bytes (from the original file, to the draft-file), FROM/INCLUDING the 'file byte position of the change operation' in question: append byte by byte, or append with a small bucket-brigade buffer, effecting a frame-shift.


In theory, this process only 'need' apply to Add-a-byte-operation and Remove-a-byte-operation not (hex-edit)change-a-byte-in-place. An in-place byte change can be done simple on a file. However, what is better:
1. A standard process of building a new file cleanly and not making any internal changes to it and which is a single process always used, or
2. Having two different workflows in the same tool-kit, whereby in-place edit makes a complete copy of a file and then navigates back to the change-spot and changes it and resaves the file. Is that simpler than writing the file per-design in the first place with a standard workflow, especially when a backup copy would be made for safety in either case? We will assume that a more uniform workflow is more practical.

Using these steps we are not 'altering' any file per-se; we are constructing the 'altered' (relatively speaking) file in one clean workflow.

# Test, Check, And Verify
There can also be checking steps such as:
- (double)checking original vs. new file: total byte length
- (double)checking original vs. new file: pre-position byte length similarity (possible a hash-check)
- (double)checking original vs. new file value: at-position, must be dissimilarity
- (double)checking original vs. new file: post-position, must be similarity given frame-shift or not (possible a hash-check)
 - - hex-edit in place: no frameshift: post-position must be the same
 - - remove byte: -1 frameshift in new file compared with original: given -1 frameshift post position must be the same
 - - add byte: +1 frameshift in new file compared with original: given +1 frameshift, post position must be the same


Remove-Byte Operation Workflow
Let me restate the remove-byte operation using your precise mechanical terminology:
Draft File Construction Process
Step 1: Create Draft File

Open original file for reading (read position starts at 0)
Create empty draft file for writing (write position starts at 0)

Step 2: Append Pre-Position Bytes

Read from original: bytes at positions 0 through byte_position_from_start - 1
Append to draft: all these bytes sequentially
Original read position after: at byte_position_from_start
Draft write position after: at byte_position_from_start

Step 3: Perform Remove Operation AT Position

Original file: advance read position by 1 (skip the byte at byte_position_from_start)

Read position moves from byte_position_from_start to byte_position_from_start + 1


Draft file: write nothing, take no action

Write position remains at byte_position_from_start


Effect: The byte at byte_position_from_start in the original is never appended to draft

Step 4: Append Post-Position Bytes

Read from original: bytes starting at position byte_position_from_start + 1 through EOF

(Original read position is already at byte_position_from_start + 1 from Step 3)


Append to draft: all remaining bytes sequentially
Effect: These bytes are written to draft starting at position byte_position_from_start

This creates the -1 frame-shift automatically
*/

/// Computes a simple checksum for a byte slice (for verification purposes)
///
/// Uses a basic XOR-based checksum for speed and simplicity.
/// This is sufficient for integrity checking, not cryptographic security.
fn compute_simple_checksum(bytes: &[u8]) -> u64 {
    let mut checksum: u64 = 0;
    for (i, &byte) in bytes.iter().enumerate() {
        // Mix position and value to detect transpositions
        checksum ^= (byte as u64).rotate_left((i % 64) as u32);
        checksum = checksum.wrapping_add(byte as u64);
    }
    checksum
}

/// Performs comprehensive verification of a byte replacement operation.
///
/// # Verification Steps
/// 1. **Total byte length check**: Ensures file sizes match exactly
/// 2. **Pre-position similarity**: Verifies all bytes before edit position are identical
/// 3. **At-position verification**: Two-part check:
///    - Check if new value equals old value (edge case warning)
///    - Verify draft has the correct new byte value
/// 4. **Post-position similarity**: Verifies all bytes after edit position are identical
///
/// # Parameters
/// - `original_path`: Path to the original file
/// - `modified_path`: Path to the modified file (draft)
/// - `byte_position`: Position where byte was replaced
/// - `expected_old_byte`: The original byte value that should have been replaced
/// - `expected_new_byte`: The new byte value that should be at the position
///
/// # Returns
/// - `Ok(())` if all verifications pass
/// - `Err(io::Error)` if any verification fails
fn verify_byte_replacement_operation(
    original_path: &Path,
    modified_path: &Path,
    byte_position: usize,
    expected_old_byte: u8,
    expected_new_byte: u8,
) -> io::Result<()> {
    #[cfg(debug_assertions)]
    println!("\n=== Comprehensive Verification Phase ===");

    // =========================================
    // Step 1: Total Byte Length Check
    // =========================================
    #[cfg(debug_assertions)]
    println!("1. Verifying total byte length...");

    let original_metadata = fs::metadata(original_path)?;
    let modified_metadata = fs::metadata(modified_path)?;
    let original_size = original_metadata.len() as usize;
    let modified_size = modified_metadata.len() as usize;

    // Debug-Assert, Test-Assert, Production-Catch-Handle
    debug_assert_eq!(
        original_size, modified_size,
        "File sizes must match for in-place edit"
    );

    #[cfg(test)]
    {
        assert_eq!(
            original_size, modified_size,
            "File sizes must match for in-place edit"
        );
    }

    if original_size != modified_size {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "File size mismatch: original={}, modified={}",
                original_size, modified_size
            ),
        ));
    }

    #[cfg(debug_assertions)]
    println!("   ✓ File sizes match: {} bytes", original_size);

    // Open both files for reading
    let mut original_file = File::open(original_path)?;
    let mut modified_file = File::open(modified_path)?;

    // =========================================
    // Step 2: Pre-Position Similarity Check
    // =========================================
    #[cfg(debug_assertions)]
    {
        if byte_position > 0 {
            println!(
                "2. Verifying pre-position bytes (0 to {})...",
                byte_position.saturating_sub(1)
            );
        } else {
            println!("2. Verifying pre-position bytes (none - position is 0)...");
        }
    }

    if byte_position > 0 {
        // Read and compare bytes before the edit position
        const VERIFICATION_BUFFER_SIZE: usize = 64;
        let mut original_buffer = [0u8; VERIFICATION_BUFFER_SIZE];
        let mut modified_buffer = [0u8; VERIFICATION_BUFFER_SIZE];

        let mut pre_position_original_checksum: u64 = 0;
        let mut pre_position_modified_checksum: u64 = 0;
        let mut bytes_verified: usize = 0;

        while bytes_verified < byte_position {
            let bytes_to_read =
                std::cmp::min(VERIFICATION_BUFFER_SIZE, byte_position - bytes_verified);

            let original_bytes_read = original_file.read(&mut original_buffer[..bytes_to_read])?;
            let modified_bytes_read = modified_file.read(&mut modified_buffer[..bytes_to_read])?;

            // Verify same number of bytes read
            if original_bytes_read != modified_bytes_read {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Pre-position read mismatch",
                ));
            }

            // Update checksums
            pre_position_original_checksum = pre_position_original_checksum.wrapping_add(
                compute_simple_checksum(&original_buffer[..original_bytes_read]),
            );
            pre_position_modified_checksum = pre_position_modified_checksum.wrapping_add(
                compute_simple_checksum(&modified_buffer[..modified_bytes_read]),
            );

            // Byte-by-byte comparison for pre-position bytes
            for i in 0..original_bytes_read {
                if original_buffer[i] != modified_buffer[i] {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!(
                            "Pre-position byte mismatch at position {}: original=0x{:02X}, modified=0x{:02X}",
                            bytes_verified + i,
                            original_buffer[i],
                            modified_buffer[i]
                        ),
                    ));
                }
            }

            bytes_verified += original_bytes_read;
        }

        // Verify checksums match
        if pre_position_original_checksum != pre_position_modified_checksum {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "Pre-position checksum mismatch: original={:016X}, modified={:016X}",
                    pre_position_original_checksum, pre_position_modified_checksum
                ),
            ));
        }

        #[cfg(debug_assertions)]
        println!(
            "   ✓ Pre-position bytes match (checksum: {:016X})",
            pre_position_original_checksum
        );
    } else {
        #[cfg(debug_assertions)]
        println!("   ✓ No pre-position bytes to verify (position is 0)");
    }

    // =========================================
    // Step 3: At-Position Verification (Two-Part Check)
    // =========================================
    #[cfg(debug_assertions)]
    println!("3. Verifying at-position byte change...");

    let mut original_byte = [0u8; 1];
    let mut modified_byte = [0u8; 1];

    original_file.read_exact(&mut original_byte)?;
    modified_file.read_exact(&mut modified_byte)?;

    // Part 1: Verify original byte is what we expected
    if original_byte[0] != expected_old_byte {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Original byte mismatch at position {}: expected=0x{:02X}, actual=0x{:02X}",
                byte_position, expected_old_byte, original_byte[0]
            ),
        ));
    }

    // Part 2: Verify modified byte is what we set
    if modified_byte[0] != expected_new_byte {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Modified byte mismatch at position {}: expected=0x{:02X}, actual=0x{:02X}",
                byte_position, expected_new_byte, modified_byte[0]
            ),
        ));
    }

    // // Edge case check: warn if old and new values are the same
    // if original_byte[0] == modified_byte[0] {
    //     #[cfg(debug_assertions)]
    //     println!(
    //         "   ⚠ Warning: New byte value (0x{:02X}) equals old byte value (operation is idempotent)",
    //         original_byte[0]
    //     );
    // }

    #[cfg(debug_assertions)]
    println!(
        "   ✓ At-position byte correctly changed: 0x{:02X} -> 0x{:02X}",
        original_byte[0], modified_byte[0]
    );

    // =========================================
    // Step 4: Post-Position Similarity Check
    // =========================================
    #[cfg(debug_assertions)]
    {
        if byte_position + 1 < original_size {
            println!(
                "4. Verifying post-position bytes ({} to EOF)...",
                byte_position + 1
            );
        } else {
            println!("4. Verifying post-position bytes (none - edit was at last byte)...");
        }
    }

    const POST_VERIFICATION_BUFFER_SIZE: usize = 64;
    let mut original_post_buffer = [0u8; POST_VERIFICATION_BUFFER_SIZE];
    let mut modified_post_buffer = [0u8; POST_VERIFICATION_BUFFER_SIZE];

    let mut post_position_original_checksum: u64 = 0;
    let mut post_position_modified_checksum: u64 = 0;
    let mut post_bytes_verified: usize = 0;

    loop {
        let original_bytes_read = original_file.read(&mut original_post_buffer)?;
        let modified_bytes_read = modified_file.read(&mut modified_post_buffer)?;

        // Both files should reach EOF at the same time
        if original_bytes_read != modified_bytes_read {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "Post-position read size mismatch: original={}, modified={}",
                    original_bytes_read, modified_bytes_read
                ),
            ));
        }

        // Check if we've reached EOF
        if original_bytes_read == 0 {
            break;
        }

        // Update checksums
        post_position_original_checksum = post_position_original_checksum.wrapping_add(
            compute_simple_checksum(&original_post_buffer[..original_bytes_read]),
        );
        post_position_modified_checksum = post_position_modified_checksum.wrapping_add(
            compute_simple_checksum(&modified_post_buffer[..modified_bytes_read]),
        );

        // Byte-by-byte comparison for post-position bytes
        for i in 0..original_bytes_read {
            if original_post_buffer[i] != modified_post_buffer[i] {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!(
                        "Post-position byte mismatch at offset +{}: original=0x{:02X}, modified=0x{:02X}",
                        post_bytes_verified + i + 1,
                        original_post_buffer[i],
                        modified_post_buffer[i]
                    ),
                ));
            }
        }

        post_bytes_verified += original_bytes_read;
    }

    // Verify post-position checksums match
    if post_position_original_checksum != post_position_modified_checksum {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Post-position checksum mismatch: original={:016X}, modified={:016X}",
                post_position_original_checksum, post_position_modified_checksum
            ),
        ));
    }

    #[cfg(debug_assertions)]
    {
        if post_bytes_verified > 0 {
            println!(
                "   ✓ Post-position bytes match ({} bytes, checksum: {:016X})",
                post_bytes_verified, post_position_original_checksum
            );
        } else {
            println!("   ✓ No post-position bytes (edit was at last byte)");
        }
    }

    // =========================================
    // Final Verification Summary
    // =========================================
    #[cfg(debug_assertions)]
    {
        println!("\n=== Verification Summary ===");
        println!("✓ Total byte length: VERIFIED ({} bytes)", original_size);
        println!("✓ Pre-position similarity: VERIFIED");
        println!("✓ At-position change: VERIFIED");
        println!("✓ Post-position similarity: VERIFIED (no frame-shift)");
        println!("All verification checks PASSED\n");
    }

    Ok(())
}

/// Performs an in-place byte replacement operation on a file using a safe copy-and-replace strategy.
///
/// # Overview
/// This function (effectively) "replaces" a single byte at a specified position
/// "in" a file without changing file length. The method is a defensive "build-new-file"
/// approach rather than modifying/changing the original file directly in any way,
/// allowing for a completely unaltered original file in the case of any errors or exceptions.
///
/// # Memory Safety
/// - Uses pre-allocated 64-byte buffer (no heap allocation)
/// - Never loads entire file into memory
/// - Processes file chunk-by-chunk using a "bucket brigade" pattern
/// - No dynamic memory allocation (pre-allocated stack only)
///
/// # File Safety Strategy
/// 1. Creates a backup copy of the original file (.backup extension)
/// 2. Builds a new draft file (.draft extension) with the modified byte
/// 3. Verifies that the operation succeeded
/// 4. Atomically replaces original with draft
/// 5. Removes backup only after verification tests pass and successful completion
///
/// # Operation Behavior
/// - Copies all bytes before target position unchanged
/// - Replaces the byte at target position with new_byte_value
/// - Copies all bytes after target position unchanged
/// - File length remains exactly the same
/// - No frame-shifting occurs
///
/// # Parameters
/// - `original_file_path`: Absolute path to the file to modify
/// - `byte_position_from_start`: Zero-indexed position of byte to replace
/// - `new_byte_value`: The new byte value to write at the specified position
///
/// # Returns
/// - `Ok(())` on successful byte replacement
/// - `Err(io::Error)` if file operations fail or position is invalid
///
/// # Error Conditions
/// - File does not exist
/// - Byte position exceeds file length
/// - Insufficient permissions
/// - Disk full
/// - I/O errors during read/write
///
/// # Recovery Behavior
/// - If operation fails before replacing original, draft is removed, backup remains
/// - If operation fails during replacement, backup file is preserved for manual recovery
/// - Orphaned .draft files indicate incomplete operations
/// - Orphaned .backup files indicate failed replacements
///
/// # Edge Cases
/// - Empty file: Returns error (no bytes to edit)
/// - Position equals file length: Returns error (position out of bounds)
/// - Position > file length: Returns error (position out of bounds)
/// - Single byte file: Replaces that byte if position is 0
/// - Same byte value: Completes operation (idempotent)
/// - Very large files: Processes in chunks, no memory issues
///
/// # Example
/// ```no_run
/// # use std::io;
/// # use std::path::PathBuf;
/// # fn replace_single_byte_in_file(path: PathBuf, pos: usize, byte: u8) -> io::Result<()> { Ok(()) }
/// let file_path = PathBuf::from("/absolute/path/to/file.dat");
/// let position = 1024; // Replace byte at position 1024
/// let new_byte = 0xFF; // Replace with 0xFF
/// let result = replace_single_byte_in_file(file_path, position, new_byte);
/// assert!(result.is_ok());
/// # Ok::<(), io::Error>(())
/// ```
pub fn replace_single_byte_in_file(
    original_file_path: PathBuf,
    byte_position_from_start: usize,
    new_byte_value: u8,
) -> io::Result<()> {
    // =========================================
    // Input Validation Phase
    // =========================================
    #[cfg(debug_assertions)]
    println!("=== In-Place Byte Replacement Operation ===");
    #[cfg(debug_assertions)]
    println!("Target file: {}", original_file_path.display());
    #[cfg(debug_assertions)]
    println!("Byte position: {}", byte_position_from_start);
    #[cfg(debug_assertions)]
    println!("New byte value: 0x{:02X}", new_byte_value);
    #[cfg(debug_assertions)]
    println!();

    // Verify file exists before any operations
    if !original_file_path.exists() {
        let error_message = format!(
            "Target file does not exist: {}",
            original_file_path.display()
        );
        eprintln!("ERROR: {}", error_message);
        return Err(io::Error::new(io::ErrorKind::NotFound, error_message));
    }

    // Verify file is actually a file, not a directory
    if !original_file_path.is_file() {
        let error_message = format!(
            "Target path is not a file: {}",
            original_file_path.display()
        );
        eprintln!("ERROR: {}", error_message);
        return Err(io::Error::new(io::ErrorKind::InvalidInput, error_message));
    }

    // Get original file metadata for validation
    let original_metadata = fs::metadata(&original_file_path)?;
    let original_file_size = original_metadata.len() as usize;

    // Validate byte position is within file bounds
    if byte_position_from_start >= original_file_size {
        let error_message = format!(
            "Byte position {} exceeds file size {} (valid range: 0-{})",
            byte_position_from_start,
            original_file_size,
            original_file_size.saturating_sub(1)
        );
        eprintln!("ERROR: {}", error_message);
        return Err(io::Error::new(io::ErrorKind::InvalidInput, error_message));
    }

    // Handle empty file case
    if original_file_size == 0 {
        let error_message = "Cannot edit byte in empty file (file size is 0)";
        eprintln!("ERROR: {}", error_message);
        return Err(io::Error::new(io::ErrorKind::InvalidInput, error_message));
    }

    // =========================================
    // Path Construction Phase
    // =========================================

    // Build backup and draft file paths
    let backup_file_path = {
        let mut backup_path = original_file_path.clone();
        let file_name = backup_path
            .file_name()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid file name"))?
            .to_string_lossy();
        let backup_name = format!("{}.backup", file_name);
        backup_path.set_file_name(backup_name);
        backup_path
    };

    let draft_file_path = {
        let mut draft_path = original_file_path.clone();
        let file_name = draft_path
            .file_name()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid file name"))?
            .to_string_lossy();
        let draft_name = format!("{}.draft", file_name);
        draft_path.set_file_name(draft_name);
        draft_path
    };
    #[cfg(debug_assertions)]
    println!("Backup path: {}", backup_file_path.display());
    #[cfg(debug_assertions)]
    println!("Draft path: {}", draft_file_path.display());
    #[cfg(debug_assertions)]
    println!();

    // =========================================
    // Backup Creation Phase
    // =========================================
    #[cfg(debug_assertions)]
    println!("Creating backup copy...");
    fs::copy(&original_file_path, &backup_file_path).map_err(|e| {
        eprintln!("ERROR: Failed to create backup: {}", e);
        e
    })?;
    #[cfg(debug_assertions)]
    println!("Backup created successfully");

    // =========================================
    // Draft File Construction Phase
    // =========================================
    #[cfg(debug_assertions)]
    println!("Building modified draft file...");

    // Open original for reading
    let mut source_file = File::open(&original_file_path)?;

    // Create draft file for writing
    let mut draft_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&draft_file_path)?;

    // Pre-allocated buffer for bucket brigade operations
    const BUCKET_BRIGADE_BUFFER_SIZE: usize = 64;
    let mut bucket_brigade_buffer = [0u8; BUCKET_BRIGADE_BUFFER_SIZE];

    // =================================================
    // Debug-Assert, Test-Assert, Production-Catch-Handle
    // =================================================

    // Debug build assertion
    debug_assert!(
        BUCKET_BRIGADE_BUFFER_SIZE > 0,
        "Bucket brigade buffer must have non-zero size"
    );

    // Test build assertion
    #[cfg(test)]
    {
        assert!(
            BUCKET_BRIGADE_BUFFER_SIZE > 0,
            "Bucket brigade buffer must have non-zero size"
        );
    }

    // Production safety check and handle
    if BUCKET_BRIGADE_BUFFER_SIZE == 0 {
        // Clean up draft file on error
        let _ = fs::remove_file(&draft_file_path);
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid buffer configuration",
        ));
    }

    // Tracking variables
    let mut total_bytes_processed: usize = 0;
    let mut chunk_number: usize = 0;
    let mut byte_was_replaced = false;

    // Safety limit to prevent infinite loops
    const MAX_CHUNKS_ALLOWED: usize = 16_777_216; // ~1GB at 64-byte chunks

    // =========================================
    // Main Processing Loop
    // =========================================

    loop {
        // =================================================
        // Debug-Assert, Test-Assert, Production-Catch-Handle
        // =================================================

        // Debug build assertion
        debug_assert!(
            chunk_number < MAX_CHUNKS_ALLOWED,
            "Exceeded maximum chunk limit"
        );

        // Test build assertion
        #[cfg(test)]
        {
            assert!(
                chunk_number < MAX_CHUNKS_ALLOWED,
                "Exceeded maximum chunk limit"
            );
        }

        // Production safety check and handle
        if chunk_number >= MAX_CHUNKS_ALLOWED {
            eprintln!("ERROR: Maximum chunk limit exceeded for safety");
            // Clean up files
            let _ = fs::remove_file(&draft_file_path);
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "File too large or infinite loop detected",
            ));
        }

        // Clear buffer before reading (prevent data leakage)
        for i in 0..BUCKET_BRIGADE_BUFFER_SIZE {
            bucket_brigade_buffer[i] = 0;
        }

        chunk_number += 1;

        // Read next chunk from source
        let bytes_read = source_file.read(&mut bucket_brigade_buffer)?;

        // EOF detection
        if bytes_read == 0 {
            #[cfg(debug_assertions)]
            println!("Reached end of file");
            break;
        }

        // =================================================
        // Debug-Assert, Test-Assert, Production-Catch-Handle
        // =================================================

        // Debug build assertion
        debug_assert!(
            bytes_read <= BUCKET_BRIGADE_BUFFER_SIZE,
            "Read more bytes than buffer size"
        );

        // Test build assertion
        #[cfg(test)]
        {
            assert!(
                bytes_read <= BUCKET_BRIGADE_BUFFER_SIZE,
                "Read more bytes than buffer size"
            );
        }

        // Production safety check and handle
        if bytes_read > BUCKET_BRIGADE_BUFFER_SIZE {
            eprintln!("ERROR: Buffer overflow detected");
            let _ = fs::remove_file(&draft_file_path);
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Buffer overflow in read operation",
            ));
        }

        // Determine if target byte is in this chunk
        let chunk_start_position = total_bytes_processed;
        let chunk_end_position = chunk_start_position + bytes_read;

        // Check if we need to modify a byte in this chunk
        if byte_position_from_start >= chunk_start_position
            && byte_position_from_start < chunk_end_position
        {
            // Calculate position within this chunk
            let position_in_chunk = byte_position_from_start - chunk_start_position;

            // Store original byte for logging
            let original_byte_value = bucket_brigade_buffer[position_in_chunk];

            // Perform the byte replacement
            bucket_brigade_buffer[position_in_chunk] = new_byte_value;
            byte_was_replaced = true;
            #[cfg(debug_assertions)]
            println!(
                "Replaced byte at position {}: 0x{:02X} -> 0x{:02X}",
                byte_position_from_start, original_byte_value, new_byte_value
            );
        }

        // Write chunk to draft file
        let bytes_written = draft_file.write(&bucket_brigade_buffer[..bytes_read])?;

        // =================================================
        // Debug-Assert, Test-Assert, Production-Catch-Handle
        // =================================================

        // Debug build assertion
        debug_assert_eq!(bytes_written, bytes_read, "Not all bytes were written");

        // Test build assertion
        #[cfg(test)]
        {
            assert_eq!(bytes_written, bytes_read, "Not all bytes were written");
        }

        // Production safety check and handle
        if bytes_written != bytes_read {
            eprintln!(
                "ERROR: Write mismatch - expected {} bytes, wrote {} bytes",
                bytes_read, bytes_written
            );
            let _ = fs::remove_file(&draft_file_path);
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Incomplete write operation",
            ));
        }

        total_bytes_processed += bytes_written;

        // Flush to ensure data is written
        draft_file.flush()?;
    }

    // =========================================
    // Verification Phase
    // =========================================
    #[cfg(debug_assertions)]
    println!("\nVerifying operation...");

    // Verify byte was actually replaced
    if !byte_was_replaced {
        eprintln!("ERROR: Target byte position was never reached");
        let _ = fs::remove_file(&draft_file_path);
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Byte replacement did not occur",
        ));
    }

    // Verify file sizes match
    draft_file.flush()?;
    drop(draft_file); // Ensure file is closed
    drop(source_file); // Ensure file is closed

    let draft_metadata = fs::metadata(&draft_file_path)?;
    let draft_size = draft_metadata.len() as usize;

    // =========================================
    // Comprehensive Verification Phase
    // =========================================

    // let mut original_check_file = File::open(&original_file_path)?; // THE ACTUAL ORIGINAL!
    // original_check_file.seek(SeekFrom::Start(byte_position_from_start as u64))?;
    // let mut byte_buffer = [0u8; 1];
    // original_check_file.read_exact(&mut byte_buffer)?;
    // let original_byte_at_position = byte_buffer[0];

    // Read original byte for verification
    /*
    This ensures the file handle is closed before you try to rename.
    The curly braces { } create a new scope. When that scope ends,
    original_check_file is immediately dropped and the file handle is closed.
    */
    let original_byte_at_position = {
        let mut original_check_file = File::open(&original_file_path)?;
        original_check_file.seek(SeekFrom::Start(byte_position_from_start as u64))?;
        let mut byte_buffer = [0u8; 1];
        original_check_file.read_exact(&mut byte_buffer)?;
        byte_buffer[0]
        // original_check_file automatically dropped here
    };

    // Perform all verification checks before replacing the original
    verify_byte_replacement_operation(
        &original_file_path, // The actual original (still unmodified)
        &draft_file_path,    // Modified (draft) file
        byte_position_from_start,
        original_byte_at_position,
        new_byte_value,
    )?;

    // =================================================
    // Debug-Assert, Test-Assert, Production-Catch-Handle
    // =================================================

    // Debug build assertion
    debug_assert_eq!(
        draft_size, original_file_size,
        "Draft file size doesn't match original"
    );

    // Test build assertion
    #[cfg(test)]
    {
        assert_eq!(
            draft_size, original_file_size,
            "Draft file size doesn't match original"
        );
    }

    // Production safety check and handle
    if draft_size != original_file_size {
        eprintln!(
            "ERROR: File size mismatch - original: {} bytes, draft: {} bytes",
            original_file_size, draft_size
        );
        let _ = fs::remove_file(&draft_file_path);
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "File size verification failed",
        ));
    }
    #[cfg(debug_assertions)]
    println!("File size verified: {} bytes", draft_size);

    // =========================================
    // Atomic Replacement Phase
    // =========================================
    #[cfg(debug_assertions)]
    println!("\nReplacing original file with modified version...");

    // Attempt atomic rename (most filesystems support this)
    match fs::rename(&draft_file_path, &original_file_path) {
        Ok(()) => {
            #[cfg(debug_assertions)]
            println!("Original file successfully replaced");
        }
        Err(e) => {
            // DO NOT try to copy over the original!
            // Leave all files as-is for safety
            eprintln!("Cannot atomically replace file: {}", e);
            return Err(e);
        }
    }

    // =========================================
    // Cleanup Phase
    // =========================================
    #[cfg(debug_assertions)]
    println!("\nCleaning up backup file...");

    // Only remove backup after successful replacement
    match fs::remove_file(&backup_file_path) {
        Ok(()) => {
            #[cfg(debug_assertions)]
            println!("Backup file removed")
        }
        Err(e) => {
            // Non-fatal: backup removal failure is not critical
            eprintln!(
                "WARNING: Could not remove backup file: {} ({})",
                backup_file_path.display(),
                e
            );
            #[cfg(debug_assertions)]
            println!("Backup file retained at: {}", backup_file_path.display());
        }
    }

    // =========================================
    // Operation Summary
    // =========================================
    #[cfg(debug_assertions)]
    println!("\n=== Operation Complete ===");
    #[cfg(debug_assertions)]
    println!("File: {}", original_file_path.display());
    #[cfg(debug_assertions)]
    println!("Modified position: {}", byte_position_from_start);
    #[cfg(debug_assertions)]
    println!("New byte value: 0x{:02X}", new_byte_value);
    #[cfg(debug_assertions)]
    println!("Total bytes processed: {}", total_bytes_processed);
    #[cfg(debug_assertions)]
    println!("Total chunks: {}", chunk_number);
    #[cfg(debug_assertions)]
    println!("Status: SUCCESS");

    Ok(())
}

// =========================================
// Test Module
// =========================================

#[cfg(test)]
mod tests {
    use super::*;
    // use std::io::Write;

    #[test]
    fn test_replace_single_byte_basic() {
        // Create test file
        let test_dir = std::env::temp_dir();
        let test_file = test_dir.join("test_byte_replace.bin");

        // Write test data
        let test_data = vec![0x00, 0x11, 0x22, 0x33, 0x44];
        std::fs::write(&test_file, &test_data).expect("Failed to create test file");

        // Replace byte at position 2 (0x22) with 0xFF
        let result = replace_single_byte_in_file(test_file.clone(), 2, 0xFF);

        assert!(result.is_ok(), "Operation should succeed");

        // Verify result
        let modified_data = std::fs::read(&test_file).expect("Failed to read modified file");
        assert_eq!(modified_data, vec![0x00, 0x11, 0xFF, 0x33, 0x44]);

        // Cleanup
        let _ = std::fs::remove_file(&test_file);
    }

    #[test]
    fn test_replace_byte_position_out_of_bounds() {
        let test_dir = std::env::temp_dir();
        let test_file = test_dir.join("test_byte_bounds.bin");

        // Create small file
        std::fs::write(&test_file, vec![0x00, 0x11]).expect("Failed to create test file");

        // Try to replace byte at invalid position
        let result = replace_single_byte_in_file(
            test_file.clone(),
            10, // Position beyond file size
            0xFF,
        );

        assert!(result.is_err(), "Should fail with out of bounds position");

        // Cleanup
        let _ = std::fs::remove_file(&test_file);
    }

    #[test]
    fn test_replace_byte_empty_file() {
        let test_dir = std::env::temp_dir();
        let test_file = test_dir.join("test_empty.bin");

        // Create empty file
        File::create(&test_file).expect("Failed to create empty file");

        // Try to replace byte in empty file
        let result = replace_single_byte_in_file(test_file.clone(), 0, 0xFF);

        assert!(result.is_err(), "Should fail with empty file");

        // Cleanup
        let _ = std::fs::remove_file(&test_file);
    }
}

// =====================
// Remove-Byte Operation
// =====================

/// Performs comprehensive verification of a byte removal operation.
///
/// # Verification Steps
/// 1. **Total byte length check**: Ensures draft is exactly 1 byte smaller than original
/// 2. **Pre-position similarity**: Verifies all bytes before removal position are identical
/// 3. **At-position dissimilarity**: Confirms byte at position has changed (is the next byte)
/// 4. **Post-position similarity with -1 frame-shift**: Verifies remaining bytes match with shift
///
/// # Frame-Shift Verification
/// After removing a byte at position N:
/// - `draft[N] == original[N+1]` (the byte after removed byte shifts into its place)
/// - `draft[N+1] == original[N+2]` (and so on...)
/// - All bytes after position N in draft correspond to position N+1 in original
///
/// # Parameters
/// - `original_path`: Path to the original file
/// - `draft_path`: Path to the draft file with byte removed
/// - `byte_position`: Position where byte was removed
/// - `removed_byte_value`: The byte value that was removed (for logging)
///
/// # Returns
/// - `Ok(())` if all verifications pass
/// - `Err(io::Error)` if any verification fails
fn verify_byte_removal_operation(
    original_path: &Path,
    draft_path: &Path,
    byte_position: usize,
    removed_byte_value: u8,
) -> io::Result<()> {
    #[cfg(debug_assertions)]
    println!("\n=== Comprehensive Verification Phase ===");

    // =========================================
    // Step 1: Total Byte Length Check
    // =========================================
    #[cfg(debug_assertions)]
    println!("1. Verifying total byte length...");

    let original_metadata = fs::metadata(original_path)?;
    let draft_metadata = fs::metadata(draft_path)?;
    let original_size = original_metadata.len() as usize;
    let draft_size = draft_metadata.len() as usize;

    let expected_draft_size = original_size.saturating_sub(1);

    // Debug-Assert, Test-Assert, Production-Catch-Handle
    debug_assert_eq!(
        draft_size, expected_draft_size,
        "Draft file must be exactly 1 byte smaller than original"
    );

    #[cfg(test)]
    {
        assert_eq!(
            draft_size, expected_draft_size,
            "Draft file must be exactly 1 byte smaller than original"
        );
    }

    if draft_size != expected_draft_size {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "File size mismatch: original={}, draft={}, expected={}",
                original_size, draft_size, expected_draft_size
            ),
        ));
    }

    #[cfg(debug_assertions)]
    println!(
        "   ✓ File sizes correct: original={} bytes, draft={} bytes (removed 1 byte)",
        original_size, draft_size
    );

    // Open both files for reading
    let mut original_file = File::open(original_path)?;
    let mut draft_file = File::open(draft_path)?;

    // =========================================
    // Step 2: Pre-Position Similarity Check
    // =========================================
    #[cfg(debug_assertions)]
    println!(
        "2. Verifying pre-position bytes (0 to {})...",
        byte_position.saturating_sub(1)
    );

    if byte_position > 0 {
        const VERIFICATION_BUFFER_SIZE: usize = 64;
        let mut original_buffer = [0u8; VERIFICATION_BUFFER_SIZE];
        let mut draft_buffer = [0u8; VERIFICATION_BUFFER_SIZE];

        let mut pre_position_original_checksum: u64 = 0;
        let mut pre_position_draft_checksum: u64 = 0;
        let mut bytes_verified: usize = 0;

        while bytes_verified < byte_position {
            let bytes_to_read =
                std::cmp::min(VERIFICATION_BUFFER_SIZE, byte_position - bytes_verified);

            let original_bytes_read = original_file.read(&mut original_buffer[..bytes_to_read])?;
            let draft_bytes_read = draft_file.read(&mut draft_buffer[..bytes_to_read])?;

            // Verify same number of bytes read
            if original_bytes_read != draft_bytes_read {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Pre-position read mismatch",
                ));
            }

            // Update checksums
            pre_position_original_checksum = pre_position_original_checksum.wrapping_add(
                compute_simple_checksum(&original_buffer[..original_bytes_read]),
            );
            pre_position_draft_checksum = pre_position_draft_checksum
                .wrapping_add(compute_simple_checksum(&draft_buffer[..draft_bytes_read]));

            // Byte-by-byte comparison for pre-position bytes
            for i in 0..original_bytes_read {
                if original_buffer[i] != draft_buffer[i] {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!(
                            "Pre-position byte mismatch at position {}: original=0x{:02X}, draft=0x{:02X}",
                            bytes_verified + i,
                            original_buffer[i],
                            draft_buffer[i]
                        ),
                    ));
                }
            }

            bytes_verified += original_bytes_read;
        }

        // Verify checksums match
        if pre_position_original_checksum != pre_position_draft_checksum {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "Pre-position checksum mismatch: original={:016X}, draft={:016X}",
                    pre_position_original_checksum, pre_position_draft_checksum
                ),
            ));
        }

        #[cfg(debug_assertions)]
        println!(
            "   ✓ Pre-position bytes match (checksum: {:016X})",
            pre_position_original_checksum
        );
    } else {
        #[cfg(debug_assertions)]
        println!("   ✓ No pre-position bytes to verify (position is 0)");
    }

    // =========================================
    // Step 3: At-Position Verification (Frame-Shift Check)
    // =========================================
    #[cfg(debug_assertions)]
    println!(
        "3. Verifying byte removal and frame-shift at position {}...",
        byte_position
    );

    // Read the byte that was removed from original
    let mut original_removed_byte = [0u8; 1];
    original_file.read_exact(&mut original_removed_byte)?;

    // Part 1: Verify it matches what we expected to remove
    if original_removed_byte[0] != removed_byte_value {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Removed byte mismatch at position {}: expected=0x{:02X}, actual=0x{:02X}",
                byte_position, removed_byte_value, original_removed_byte[0]
            ),
        ));
    }

    // Part 2: Verify the frame-shift occurred correctly
    // The byte now at position N in draft should be the byte that was at position N+1 in original
    let mut draft_current_byte = [0u8; 1];

    // Handle edge case: if we removed the last byte, draft has no more bytes
    let draft_has_more_bytes = draft_file.read(&mut draft_current_byte)? == 1;

    if draft_has_more_bytes {
        // Read the next byte from original (the byte after the removed one)
        let mut original_next_byte = [0u8; 1];
        let original_has_next = original_file.read(&mut original_next_byte)? == 1;

        if !original_has_next {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Draft has more bytes than expected after removal position",
            ));
        }

        // Verify: draft[N] == original[N+1]
        if draft_current_byte[0] != original_next_byte[0] {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "Frame-shift verification failed: draft[{}]=0x{:02X} should equal original[{}]=0x{:02X}",
                    byte_position,
                    draft_current_byte[0],
                    byte_position + 1,
                    original_next_byte[0]
                ),
            ));
        }

        #[cfg(debug_assertions)]
        println!(
            "   ✓ Byte removed: 0x{:02X} | Frame-shift verified: draft[{}]=0x{:02X} == original[{}]=0x{:02X}",
            original_removed_byte[0],
            byte_position,
            draft_current_byte[0],
            byte_position + 1,
            original_next_byte[0]
        );
    } else {
        #[cfg(debug_assertions)]
        println!(
            "   ✓ Byte removed: 0x{:02X} (was last byte in file)",
            original_removed_byte[0]
        );
    }
    // =========================================
    // Step 4: Post-Position Similarity Check with -1 Frame-Shift
    // =========================================
    #[cfg(debug_assertions)]
    println!("4. Verifying post-position bytes with -1 frame-shift...");

    const POST_VERIFICATION_BUFFER_SIZE: usize = 64;
    let mut original_post_buffer = [0u8; POST_VERIFICATION_BUFFER_SIZE];
    let mut draft_post_buffer = [0u8; POST_VERIFICATION_BUFFER_SIZE];

    let mut post_position_original_checksum: u64 = 0;
    let mut post_position_draft_checksum: u64 = 0;
    let mut post_bytes_verified: usize = 0;

    // Note: We already read one byte from each file in Step 3
    // Original file read position: byte_position + 2
    // Draft file read position: byte_position + 1
    // These are already correctly offset by the frame-shift

    loop {
        let original_bytes_read = original_file.read(&mut original_post_buffer)?;
        let draft_bytes_read = draft_file.read(&mut draft_post_buffer)?;

        // Both files should reach EOF at the same time (accounting for the removed byte)
        if original_bytes_read != draft_bytes_read {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "Post-position read size mismatch: original={}, draft={}",
                    original_bytes_read, draft_bytes_read
                ),
            ));
        }

        // Check if we've reached EOF
        if original_bytes_read == 0 {
            break;
        }

        // Update checksums
        post_position_original_checksum = post_position_original_checksum.wrapping_add(
            compute_simple_checksum(&original_post_buffer[..original_bytes_read]),
        );
        post_position_draft_checksum = post_position_draft_checksum.wrapping_add(
            compute_simple_checksum(&draft_post_buffer[..draft_bytes_read]),
        );

        // Byte-by-byte comparison for post-position bytes (with frame-shift already in effect)
        for i in 0..original_bytes_read {
            if original_post_buffer[i] != draft_post_buffer[i] {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!(
                        "Post-position byte mismatch at offset +{}: original=0x{:02X}, draft=0x{:02X}",
                        post_bytes_verified + i,
                        original_post_buffer[i],
                        draft_post_buffer[i]
                    ),
                ));
            }
        }

        post_bytes_verified += original_bytes_read;
    }

    // Verify post-position checksums match
    if post_position_original_checksum != post_position_draft_checksum {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Post-position checksum mismatch: original={:016X}, draft={:016X}",
                post_position_original_checksum, post_position_draft_checksum
            ),
        ));
    }

    if post_bytes_verified > 0 {
        #[cfg(debug_assertions)]
        println!(
            "   ✓ Post-position bytes match with -1 frame-shift ({} bytes, checksum: {:016X})",
            post_bytes_verified, post_position_original_checksum
        );
    } else {
        #[cfg(debug_assertions)]
        println!("   ✓ No post-position bytes (removal was at last byte)");
    }

    // =========================================
    // Final Verification Summary
    // =========================================
    #[cfg(debug_assertions)]
    println!("\n=== Verification Summary ===");
    #[cfg(debug_assertions)]
    println!(
        "✓ Total byte length: VERIFIED (original={}, draft={}, -1 byte)",
        original_size, draft_size
    );
    #[cfg(debug_assertions)]
    println!("✓ Pre-position similarity: VERIFIED");
    #[cfg(debug_assertions)]
    println!("✓ At-position dissimilarity: VERIFIED (byte removed)");
    #[cfg(debug_assertions)]
    println!("✓ Post-position similarity: VERIFIED (with -1 frame-shift)");
    #[cfg(debug_assertions)]
    println!("All verification checks PASSED\n");

    Ok(())
}

/// Performs a byte removal operation on a file using a safe copy-and-replace strategy.
///
/// # Overview
/// This function removes a single byte at a specified position in a file, causing all
/// subsequent bytes to shift backward by one position (frame-shift -1). It uses a defensive
/// "build-new-file" approach rather than modifying the original file directly.
///
/// # Memory Safety
/// - Uses pre-allocated 64-byte buffer (no heap allocation)
/// - Never loads entire file into memory
/// - Processes file chunk-by-chunk using bucket brigade pattern
/// - No dynamic memory allocation
///
/// # File Safety Strategy
/// 1. Creates a backup copy of the original file (.backup extension)
/// 2. Builds a new draft file (.draft extension) with the byte removed
/// 3. Verifies the operation succeeded (including frame-shift verification)
/// 4. Atomically replaces original with draft
/// 5. Removes backup only after successful completion
///
/// # Operation Behavior - Mechanical Steps
/// The draft file is constructed by appending bytes sequentially:
///
/// **Step 1**: Create empty draft file
///
/// **Step 2**: Append pre-position bytes
/// - Read from original: positions 0 to `byte_position - 1`
/// - Append to draft: all these bytes
///
/// **Step 3**: Perform removal AT position
/// - Original file: advance read position by 1 (skip target byte)
/// - Draft file: write nothing (no append action)
/// - Effect: The byte at target position is never written to draft
///
/// **Step 4**: Append post-position bytes
/// - Read from original: positions `byte_position + 1` to EOF
/// - Append to draft: all remaining bytes
/// - Effect: These bytes naturally occupy positions starting at `byte_position` in draft
/// - This creates the -1 frame-shift automatically
///
/// # Frame-Shift Behavior
/// After removing byte at position N:
/// - Bytes 0 to N-1: unchanged positions
/// - Byte at N: removed (does not exist in new file)
/// - Bytes N+1 to EOF: all shift backward by 1 position
/// - File length decreases by exactly 1
///
/// # Parameters
/// - `original_file_path`: Absolute path to the file to modify
/// - `byte_position_from_start`: Zero-indexed position of byte to remove
///
/// # Returns
/// - `Ok(())` on successful byte removal
/// - `Err(io::Error)` if file operations fail or position is invalid
///
/// # Error Conditions
/// - File does not exist
/// - File is empty
/// - Byte position >= file length (out of bounds)
/// - Insufficient permissions
/// - Disk full
/// - I/O errors during read/write
///
/// # Recovery Behavior
/// - If operation fails before replacing original, draft is removed, backup remains
/// - If atomic rename fails, both original and backup are preserved
/// - Orphaned .draft files indicate incomplete operations
/// - Orphaned .backup files indicate failed replacements
///
/// # Edge Cases
/// - Empty file: Returns error (no bytes to remove)
/// - Position >= file length: Returns error (position out of bounds)
/// - Single byte file at position 0: Results in empty file (valid operation)
/// - Remove last byte: File becomes 1 byte shorter, no post-position bytes
/// - Remove first byte: No pre-position bytes, all bytes shift backward
/// - Very large files: Processes in chunks, no memory issues
///
/// # Example
/// ```no_run
/// # use std::io;
/// # use std::path::PathBuf;
/// # fn remove_single_byte_from_file(path: PathBuf, pos: usize) -> io::Result<()> { Ok(()) }
/// // Original file: [0x41, 0x42, 0x43, 0x44, 0x45]
/// let file_path = PathBuf::from("/absolute/path/to/file.dat");
/// let position = 2; // Remove byte at position 2 (0x43)
/// let result = remove_single_byte_from_file(file_path, position);
/// // Resulting file: [0x41, 0x42, 0x44, 0x45]
/// // Note: 0x44 and 0x45 shifted backward by 1 position
/// assert!(result.is_ok());
/// # Ok::<(), io::Error>(())
/// ```
pub fn remove_single_byte_from_file(
    original_file_path: PathBuf,
    byte_position_from_start: usize,
) -> io::Result<()> {
    // =========================================
    // Input Validation Phase
    // =========================================
    #[cfg(debug_assertions)]
    println!("=== Byte Removal Operation ===");
    #[cfg(debug_assertions)]
    println!("Target file: {}", original_file_path.display());
    #[cfg(debug_assertions)]
    println!("Byte position to remove: {}", byte_position_from_start);
    #[cfg(debug_assertions)]
    println!();

    // Verify file exists before any operations
    if !original_file_path.exists() {
        let error_message = format!(
            "Target file does not exist: {}",
            original_file_path.display()
        );
        eprintln!("ERROR: {}", error_message);
        return Err(io::Error::new(io::ErrorKind::NotFound, error_message));
    }

    // Verify file is actually a file, not a directory
    if !original_file_path.is_file() {
        let error_message = format!(
            "Target path is not a file: {}",
            original_file_path.display()
        );
        eprintln!("ERROR: {}", error_message);
        return Err(io::Error::new(io::ErrorKind::InvalidInput, error_message));
    }

    // Get original file metadata for validation
    let original_metadata = fs::metadata(&original_file_path)?;
    let original_file_size = original_metadata.len() as usize;

    // Handle empty file case
    if original_file_size == 0 {
        let error_message = "Cannot remove byte from empty file (file size is 0)";
        eprintln!("ERROR: {}", error_message);
        return Err(io::Error::new(io::ErrorKind::InvalidInput, error_message));
    }

    // Validate byte position is within file bounds
    if byte_position_from_start >= original_file_size {
        let error_message = format!(
            "Byte position {} exceeds file size {} (valid range: 0-{})",
            byte_position_from_start,
            original_file_size,
            original_file_size.saturating_sub(1)
        );
        eprintln!("ERROR: {}", error_message);
        return Err(io::Error::new(io::ErrorKind::InvalidInput, error_message));
    }

    // =========================================
    // Path Construction Phase
    // =========================================

    // Build backup and draft file paths
    let backup_file_path = {
        let mut backup_path = original_file_path.clone();
        let file_name = backup_path
            .file_name()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid file name"))?
            .to_string_lossy();
        let backup_name = format!("{}.backup", file_name);
        backup_path.set_file_name(backup_name);
        backup_path
    };

    let draft_file_path = {
        let mut draft_path = original_file_path.clone();
        let file_name = draft_path
            .file_name()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid file name"))?
            .to_string_lossy();
        let draft_name = format!("{}.draft", file_name);
        draft_path.set_file_name(draft_name);
        draft_path
    };
    #[cfg(debug_assertions)]
    println!("Backup path: {}", backup_file_path.display());
    #[cfg(debug_assertions)]
    println!("Draft path: {}", draft_file_path.display());
    #[cfg(debug_assertions)]
    println!();

    // =========================================
    // Backup Creation Phase
    // =========================================
    #[cfg(debug_assertions)]
    println!("Creating backup copy...");
    fs::copy(&original_file_path, &backup_file_path).map_err(|e| {
        eprintln!("ERROR: Failed to create backup: {}", e);
        e
    })?;
    #[cfg(debug_assertions)]
    println!("Backup created successfully");

    // =========================================
    // Draft File Construction Phase
    // =========================================
    #[cfg(debug_assertions)]
    println!(
        "Building modified draft file (removing byte at position {})...",
        byte_position_from_start
    );

    // Open original for reading
    let mut source_file = File::open(&original_file_path)?;

    // Create draft file for writing
    let mut draft_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&draft_file_path)?;

    // Pre-allocated buffer for bucket brigade operations
    const BUCKET_BRIGADE_BUFFER_SIZE: usize = 64;
    let mut bucket_brigade_buffer = [0u8; BUCKET_BRIGADE_BUFFER_SIZE];

    // =================================================
    // Debug-Assert, Test-Assert, Production-Catch-Handle
    // =================================================

    debug_assert!(
        BUCKET_BRIGADE_BUFFER_SIZE > 0,
        "Bucket brigade buffer must have non-zero size"
    );

    #[cfg(test)]
    {
        assert!(
            BUCKET_BRIGADE_BUFFER_SIZE > 0,
            "Bucket brigade buffer must have non-zero size"
        );
    }

    if BUCKET_BRIGADE_BUFFER_SIZE == 0 {
        let _ = fs::remove_file(&draft_file_path);
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid buffer configuration",
        ));
    }

    // Tracking variables
    let mut total_bytes_read_from_original: usize = 0;
    let mut total_bytes_written_to_draft: usize = 0;
    let mut chunk_number: usize = 0;
    let mut byte_was_removed = false;
    let mut removed_byte_value: u8 = 0;

    // Safety limit to prevent infinite loops
    const MAX_CHUNKS_ALLOWED: usize = 16_777_216;

    // =========================================
    // Main Processing Loop
    // =========================================

    loop {
        // =================================================
        // Debug-Assert, Test-Assert, Production-Catch-Handle
        // =================================================

        debug_assert!(
            chunk_number < MAX_CHUNKS_ALLOWED,
            "Exceeded maximum chunk limit"
        );

        #[cfg(test)]
        {
            assert!(
                chunk_number < MAX_CHUNKS_ALLOWED,
                "Exceeded maximum chunk limit"
            );
        }

        if chunk_number >= MAX_CHUNKS_ALLOWED {
            eprintln!("ERROR: Maximum chunk limit exceeded for safety");
            let _ = fs::remove_file(&draft_file_path);
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "File too large or infinite loop detected",
            ));
        }

        // Clear buffer before reading (prevent data leakage)
        for i in 0..BUCKET_BRIGADE_BUFFER_SIZE {
            bucket_brigade_buffer[i] = 0;
        }

        chunk_number += 1;

        // Read next chunk from source
        let bytes_read = source_file.read(&mut bucket_brigade_buffer)?;

        // EOF detection
        if bytes_read == 0 {
            #[cfg(debug_assertions)]
            println!("Reached end of original file");
            break;
        }

        // =================================================
        // Debug-Assert, Test-Assert, Production-Catch-Handle
        // =================================================

        debug_assert!(
            bytes_read <= BUCKET_BRIGADE_BUFFER_SIZE,
            "Read more bytes than buffer size"
        );

        #[cfg(test)]
        {
            assert!(
                bytes_read <= BUCKET_BRIGADE_BUFFER_SIZE,
                "Read more bytes than buffer size"
            );
        }

        if bytes_read > BUCKET_BRIGADE_BUFFER_SIZE {
            eprintln!("ERROR: Buffer overflow detected");
            let _ = fs::remove_file(&draft_file_path);
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Buffer overflow in read operation",
            ));
        }

        // Determine if target byte is in this chunk
        let chunk_start_position = total_bytes_read_from_original;
        let chunk_end_position = chunk_start_position + bytes_read;

        // Check if we need to skip a byte in this chunk (the removal operation)
        if byte_position_from_start >= chunk_start_position
            && byte_position_from_start < chunk_end_position
        {
            // Calculate position within this chunk
            let position_in_chunk = byte_position_from_start - chunk_start_position;

            // Store the byte being removed for verification
            removed_byte_value = bucket_brigade_buffer[position_in_chunk];
            byte_was_removed = true;
            #[cfg(debug_assertions)]
            println!(
                "Removing byte at position {}: 0x{:02X}",
                byte_position_from_start, removed_byte_value
            );

            // Write bytes BEFORE the removal position in this chunk
            if position_in_chunk > 0 {
                let bytes_before = &bucket_brigade_buffer[..position_in_chunk];
                let bytes_written_before = draft_file.write(bytes_before)?;

                // =================================================
                // Debug-Assert, Test-Assert, Production-Catch-Handle
                // =================================================

                debug_assert_eq!(
                    bytes_written_before, position_in_chunk,
                    "Not all pre-removal bytes were written"
                );

                #[cfg(test)]
                {
                    assert_eq!(
                        bytes_written_before, position_in_chunk,
                        "Not all pre-removal bytes were written"
                    );
                }

                if bytes_written_before != position_in_chunk {
                    eprintln!("ERROR: Incomplete write before removal position");
                    let _ = fs::remove_file(&draft_file_path);
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        "Incomplete write operation",
                    ));
                }

                total_bytes_written_to_draft += bytes_written_before;
            }

            // SKIP the byte at position_in_chunk (this is the removal operation)
            // Do not write bucket_brigade_buffer[position_in_chunk] to draft

            // Write bytes AFTER the removal position in this chunk
            let position_after_removal = position_in_chunk + 1;
            if position_after_removal < bytes_read {
                let bytes_after = &bucket_brigade_buffer[position_after_removal..bytes_read];
                let bytes_written_after = draft_file.write(bytes_after)?;

                let expected_bytes_after = bytes_read - position_after_removal;

                // =================================================
                // Debug-Assert, Test-Assert, Production-Catch-Handle
                // =================================================

                debug_assert_eq!(
                    bytes_written_after, expected_bytes_after,
                    "Not all post-removal bytes were written"
                );

                #[cfg(test)]
                {
                    assert_eq!(
                        bytes_written_after, expected_bytes_after,
                        "Not all post-removal bytes were written"
                    );
                }

                if bytes_written_after != expected_bytes_after {
                    eprintln!("ERROR: Incomplete write after removal position");
                    let _ = fs::remove_file(&draft_file_path);
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        "Incomplete write operation",
                    ));
                }

                total_bytes_written_to_draft += bytes_written_after;
            }
        } else {
            // This chunk does not contain the removal position
            // Write entire chunk to draft file
            let bytes_written = draft_file.write(&bucket_brigade_buffer[..bytes_read])?;

            // =================================================
            // Debug-Assert, Test-Assert, Production-Catch-Handle
            // =================================================

            debug_assert_eq!(bytes_written, bytes_read, "Not all bytes were written");

            #[cfg(test)]
            {
                assert_eq!(bytes_written, bytes_read, "Not all bytes were written");
            }

            if bytes_written != bytes_read {
                eprintln!(
                    "ERROR: Write mismatch - expected {} bytes, wrote {} bytes",
                    bytes_read, bytes_written
                );
                let _ = fs::remove_file(&draft_file_path);
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Incomplete write operation",
                ));
            }

            total_bytes_written_to_draft += bytes_written;
        }

        total_bytes_read_from_original += bytes_read;

        // Flush to ensure data is written
        draft_file.flush()?;
    }

    // =========================================
    // Basic Verification Phase
    // =========================================
    #[cfg(debug_assertions)]
    println!("\nVerifying operation...");

    // Verify byte was actually removed
    if !byte_was_removed {
        eprintln!("ERROR: Target byte position was never reached");
        let _ = fs::remove_file(&draft_file_path);
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Byte removal did not occur",
        ));
    }

    // Verify draft file is exactly 1 byte smaller
    draft_file.flush()?;
    drop(draft_file);
    drop(source_file);

    let draft_metadata = fs::metadata(&draft_file_path)?;
    let draft_size = draft_metadata.len() as usize;
    let expected_draft_size = original_file_size - 1;

    // =================================================
    // Debug-Assert, Test-Assert, Production-Catch-Handle
    // =================================================

    debug_assert_eq!(draft_size, expected_draft_size, "Draft file size incorrect");

    #[cfg(test)]
    {
        assert_eq!(draft_size, expected_draft_size, "Draft file size incorrect");
    }

    if draft_size != expected_draft_size {
        eprintln!(
            "ERROR: File size mismatch - original: {} bytes, draft: {} bytes, expected: {} bytes",
            original_file_size, draft_size, expected_draft_size
        );
        let _ = fs::remove_file(&draft_file_path);
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "File size verification failed",
        ));
    }
    #[cfg(debug_assertions)]
    println!(
        "Basic verification passed: original={} bytes, draft={} bytes (-1 byte)",
        original_file_size, draft_size
    );

    // =========================================
    // Comprehensive Verification Phase
    // =========================================

    // Perform all verification checks before replacing the original
    verify_byte_removal_operation(
        &original_file_path,
        &draft_file_path,
        byte_position_from_start,
        removed_byte_value,
    )?;

    // =========================================
    // Atomic Replacement Phase
    // =========================================
    #[cfg(debug_assertions)]
    println!("\nReplacing original file with modified version...");

    // Attempt atomic rename
    match fs::rename(&draft_file_path, &original_file_path) {
        Ok(()) => {
            #[cfg(debug_assertions)]
            println!("Original file successfully replaced");
        }
        Err(e) => {
            eprintln!("Cannot atomically replace file: {}", e);
            eprintln!("Original and backup files preserved for safety");
            return Err(e);
        }
    }

    // =========================================
    // Cleanup Phase
    // =========================================
    #[cfg(debug_assertions)]
    println!("\nCleaning up backup file...");

    match fs::remove_file(&backup_file_path) {
        Ok(()) => println!("Backup file removed"),
        Err(e) => {
            eprintln!(
                "WARNING: Could not remove backup file: {} ({})",
                backup_file_path.display(),
                e
            );
            #[cfg(debug_assertions)]
            println!("Backup file retained at: {}", backup_file_path.display());
        }
    }

    // =========================================
    // Operation Summary
    // =========================================
    #[cfg(debug_assertions)]
    println!("\n=== Operation Complete ===");
    #[cfg(debug_assertions)]
    println!("File: {}", original_file_path.display());
    #[cfg(debug_assertions)]
    println!("Removed byte at position: {}", byte_position_from_start);
    #[cfg(debug_assertions)]
    println!("Removed byte value: 0x{:02X}", removed_byte_value);
    #[cfg(debug_assertions)]
    println!("Original size: {} bytes", original_file_size);
    #[cfg(debug_assertions)]
    println!("New size: {} bytes", draft_size);
    #[cfg(debug_assertions)]
    println!(
        "Bytes read from original: {}",
        total_bytes_read_from_original
    );
    #[cfg(debug_assertions)]
    println!("Bytes written to draft: {}", total_bytes_written_to_draft);
    #[cfg(debug_assertions)]
    println!("Total chunks: {}", chunk_number);
    #[cfg(debug_assertions)]
    println!("Status: SUCCESS");

    Ok(())
}

// =========================================
// Test Module
// =========================================

#[cfg(test)]
mod removal_tests {
    use super::*;

    #[test]
    fn test_remove_single_byte_basic() {
        let test_dir = std::env::temp_dir();
        let test_file = test_dir.join("test_byte_remove.bin");

        // Create test file: [0x00, 0x11, 0x22, 0x33, 0x44]
        let test_data = vec![0x00, 0x11, 0x22, 0x33, 0x44];
        std::fs::write(&test_file, &test_data).expect("Failed to create test file");

        // Remove byte at position 2 (0x22)
        let result = remove_single_byte_from_file(test_file.clone(), 2);

        assert!(result.is_ok(), "Operation should succeed");

        // Verify result: [0x00, 0x11, 0x33, 0x44]
        let modified_data = std::fs::read(&test_file).expect("Failed to read modified file");
        assert_eq!(modified_data, vec![0x00, 0x11, 0x33, 0x44]);

        // Cleanup
        let _ = std::fs::remove_file(&test_file);
    }

    #[test]
    fn test_remove_first_byte() {
        let test_dir = std::env::temp_dir();
        let test_file = test_dir.join("test_remove_first.bin");

        let test_data = vec![0xAA, 0xBB, 0xCC];
        std::fs::write(&test_file, &test_data).expect("Failed to create test file");

        // Remove first byte
        let result = remove_single_byte_from_file(test_file.clone(), 0);

        assert!(result.is_ok());

        let modified_data = std::fs::read(&test_file).expect("Failed to read modified file");
        assert_eq!(modified_data, vec![0xBB, 0xCC]);

        let _ = std::fs::remove_file(&test_file);
    }

    #[test]
    fn test_remove_last_byte() {
        let test_dir = std::env::temp_dir();
        let test_file = test_dir.join("test_remove_last.bin");

        let test_data = vec![0xAA, 0xBB, 0xCC];
        std::fs::write(&test_file, &test_data).expect("Failed to create test file");

        // Remove last byte
        let result = remove_single_byte_from_file(test_file.clone(), 2);

        assert!(result.is_ok());

        let modified_data = std::fs::read(&test_file).expect("Failed to read modified file");
        assert_eq!(modified_data, vec![0xAA, 0xBB]);

        let _ = std::fs::remove_file(&test_file);
    }

    #[test]
    fn test_remove_from_single_byte_file() {
        let test_dir = std::env::temp_dir();
        let test_file = test_dir.join("test_remove_single.bin");

        std::fs::write(&test_file, vec![0x42]).expect("Failed to create test file");

        let result = remove_single_byte_from_file(test_file.clone(), 0);

        assert!(result.is_ok());

        let modified_data = std::fs::read(&test_file).expect("Failed to read modified file");
        assert_eq!(modified_data, Vec::<u8>::new()); // Empty file

        let _ = std::fs::remove_file(&test_file);
    }

    #[test]
    fn test_remove_byte_out_of_bounds() {
        let test_dir = std::env::temp_dir();
        let test_file = test_dir.join("test_remove_bounds.bin");

        std::fs::write(&test_file, vec![0x00, 0x11]).expect("Failed to create test file");

        let result = remove_single_byte_from_file(test_file.clone(), 10);

        assert!(result.is_err(), "Should fail with out of bounds position");

        let _ = std::fs::remove_file(&test_file);
    }

    #[test]
    fn test_remove_from_empty_file() {
        let test_dir = std::env::temp_dir();
        let test_file = test_dir.join("test_remove_empty.bin");

        File::create(&test_file).expect("Failed to create empty file");

        let result = remove_single_byte_from_file(test_file.clone(), 0);

        assert!(result.is_err(), "Should fail with empty file");

        let _ = std::fs::remove_file(&test_file);
    }
}

// ========
// Add Byte
// ========
/*
Mechanical Steps of Add Byte:
For building the draft file when adding a byte at position N:
- Step 2: Append pre-position bytes (0 to N-1) from original to draft
- Step 3: Append the NEW byte to draft (do NOT advance original read position)
- Step 4: Append remaining bytes (FROM position N to EOF) from original to draft
So the original post-target-position-step position at step 4 is still at N,
causing the byte that WAS(is) at N in the original to now be at N+1 in draft.

Appending at end of file must be allowed.
*/

/// Performs comprehensive verification of a byte addition operation.
///
/// # Verification Steps
/// 1. **Total byte length check**: Ensures draft is exactly 1 byte larger than original
/// 2. **Pre-position similarity**: Verifies all bytes before insertion position are identical
/// 3. **At-position verification**: Confirms the new byte was inserted correctly
/// 4. **Post-position similarity with +1 frame-shift**: Verifies remaining bytes match with shift
///
/// # Frame-Shift Verification
/// After adding a byte at position N:
/// - `draft[N] == new_byte_value` (the inserted byte)
/// - `draft[N+1] == original[N]` (first byte after insertion, shifted forward)
/// - `draft[N+2] == original[N+1]` (second byte after insertion, shifted forward)
/// - All bytes from position N onward in original are shifted +1 in draft
///
/// # Parameters
/// - `original_path`: Path to the original file
/// - `draft_path`: Path to the draft file with byte added
/// - `byte_position`: Position where byte was inserted
/// - `new_byte_value`: The byte value that was inserted
///
/// # Returns
/// - `Ok(())` if all verifications pass
/// - `Err(io::Error)` if any verification fails
fn verify_byte_addition_operation(
    original_path: &Path,
    draft_path: &Path,
    byte_position: usize,
    new_byte_value: u8,
) -> io::Result<()> {
    #[cfg(debug_assertions)]
    println!("\n=== Comprehensive Verification Phase ===");

    // =========================================
    // Step 1: Total Byte Length Check
    // =========================================
    #[cfg(debug_assertions)]
    println!("1. Verifying total byte length...");

    let original_metadata = fs::metadata(original_path)?;
    let draft_metadata = fs::metadata(draft_path)?;
    let original_size = original_metadata.len() as usize;
    let draft_size = draft_metadata.len() as usize;

    let expected_draft_size = original_size + 1;

    // Debug-Assert, Test-Assert, Production-Catch-Handle
    debug_assert_eq!(
        draft_size, expected_draft_size,
        "Draft file must be exactly 1 byte larger than original"
    );

    #[cfg(test)]
    {
        assert_eq!(
            draft_size, expected_draft_size,
            "Draft file must be exactly 1 byte larger than original"
        );
    }

    if draft_size != expected_draft_size {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "File size mismatch: original={}, draft={}, expected={}",
                original_size, draft_size, expected_draft_size
            ),
        ));
    }

    #[cfg(debug_assertions)]
    println!(
        "   ✓ File sizes correct: original={} bytes, draft={} bytes (+1 byte)",
        original_size, draft_size
    );

    // Open both files for reading
    let mut original_file = File::open(original_path)?;
    let mut draft_file = File::open(draft_path)?;

    // =========================================
    // Step 2: Pre-Position Similarity Check
    // =========================================
    #[cfg(debug_assertions)]
    {
        if byte_position > 0 {
            println!(
                "2. Verifying pre-position bytes (0 to {})...",
                byte_position.saturating_sub(1)
            );
        } else {
            println!("2. Verifying pre-position bytes (none - inserting at position 0)...");
        }
    }

    if byte_position > 0 {
        const VERIFICATION_BUFFER_SIZE: usize = 64;
        let mut original_buffer = [0u8; VERIFICATION_BUFFER_SIZE];
        let mut draft_buffer = [0u8; VERIFICATION_BUFFER_SIZE];

        let mut pre_position_original_checksum: u64 = 0;
        let mut pre_position_draft_checksum: u64 = 0;
        let mut bytes_verified: usize = 0;

        while bytes_verified < byte_position {
            let bytes_to_read =
                std::cmp::min(VERIFICATION_BUFFER_SIZE, byte_position - bytes_verified);

            let original_bytes_read = original_file.read(&mut original_buffer[..bytes_to_read])?;
            let draft_bytes_read = draft_file.read(&mut draft_buffer[..bytes_to_read])?;

            // Verify same number of bytes read
            if original_bytes_read != draft_bytes_read {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Pre-position read mismatch",
                ));
            }

            // Update checksums
            pre_position_original_checksum = pre_position_original_checksum.wrapping_add(
                compute_simple_checksum(&original_buffer[..original_bytes_read]),
            );
            pre_position_draft_checksum = pre_position_draft_checksum
                .wrapping_add(compute_simple_checksum(&draft_buffer[..draft_bytes_read]));

            // Byte-by-byte comparison for pre-position bytes
            for i in 0..original_bytes_read {
                if original_buffer[i] != draft_buffer[i] {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!(
                            "Pre-position byte mismatch at position {}: original=0x{:02X}, draft=0x{:02X}",
                            bytes_verified + i,
                            original_buffer[i],
                            draft_buffer[i]
                        ),
                    ));
                }
            }

            bytes_verified += original_bytes_read;
        }

        // Verify checksums match
        if pre_position_original_checksum != pre_position_draft_checksum {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "Pre-position checksum mismatch: original={:016X}, draft={:016X}",
                    pre_position_original_checksum, pre_position_draft_checksum
                ),
            ));
        }

        #[cfg(debug_assertions)]
        println!(
            "   ✓ Pre-position bytes match (checksum: {:016X})",
            pre_position_original_checksum
        );
    } else {
        #[cfg(debug_assertions)]
        println!("   ✓ No pre-position bytes to verify (inserting at position 0)");
    }

    // =========================================
    // Step 3: At-Position Verification
    // =========================================
    #[cfg(debug_assertions)]
    println!(
        "3. Verifying byte insertion at position {}...",
        byte_position
    );

    // Read the byte that should be the newly inserted byte in draft
    let mut draft_inserted_byte = [0u8; 1];
    draft_file.read_exact(&mut draft_inserted_byte)?;

    // Verify it matches the byte we inserted
    if draft_inserted_byte[0] != new_byte_value {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Inserted byte mismatch at position {}: expected=0x{:02X}, actual=0x{:02X}",
                byte_position, new_byte_value, draft_inserted_byte[0]
            ),
        ));
    }

    #[cfg(debug_assertions)]
    println!(
        "   ✓ Byte inserted correctly: draft[{}]=0x{:02X}",
        byte_position, draft_inserted_byte[0]
    );

    // =========================================
    // Step 4: Post-Position Similarity Check with +1 Frame-Shift
    // =========================================
    #[cfg(debug_assertions)]
    {
        if byte_position < original_size {
            println!("4. Verifying post-position bytes with +1 frame-shift...");
        } else {
            println!("4. Verifying post-position bytes (none - inserted at EOF)...");
        }
    }

    const POST_VERIFICATION_BUFFER_SIZE: usize = 64;
    let mut original_post_buffer = [0u8; POST_VERIFICATION_BUFFER_SIZE];
    let mut draft_post_buffer = [0u8; POST_VERIFICATION_BUFFER_SIZE];

    let mut post_position_original_checksum: u64 = 0;
    let mut post_position_draft_checksum: u64 = 0;
    let mut post_bytes_verified: usize = 0;

    // Note: After reading the inserted byte, draft file read position is at byte_position + 1
    // Original file read position is at byte_position
    // These are correctly offset for the +1 frame-shift

    loop {
        let original_bytes_read = original_file.read(&mut original_post_buffer)?;
        let draft_bytes_read = draft_file.read(&mut draft_post_buffer)?;

        // Both files should reach EOF at the same time (accounting for the inserted byte)
        if original_bytes_read != draft_bytes_read {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "Post-position read size mismatch: original={}, draft={}",
                    original_bytes_read, draft_bytes_read
                ),
            ));
        }

        // Check if we've reached EOF
        if original_bytes_read == 0 {
            break;
        }

        // Update checksums
        post_position_original_checksum = post_position_original_checksum.wrapping_add(
            compute_simple_checksum(&original_post_buffer[..original_bytes_read]),
        );
        post_position_draft_checksum = post_position_draft_checksum.wrapping_add(
            compute_simple_checksum(&draft_post_buffer[..draft_bytes_read]),
        );

        // Byte-by-byte comparison for post-position bytes (with +1 frame-shift in effect)
        for i in 0..original_bytes_read {
            if original_post_buffer[i] != draft_post_buffer[i] {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!(
                        "Post-position byte mismatch: original[{}]=0x{:02X}, draft[{}]=0x{:02X}",
                        byte_position + post_bytes_verified + i,
                        original_post_buffer[i],
                        byte_position + 1 + post_bytes_verified + i,
                        draft_post_buffer[i]
                    ),
                ));
            }
        }

        post_bytes_verified += original_bytes_read;
    }

    // Verify post-position checksums match
    if post_position_original_checksum != post_position_draft_checksum {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Post-position checksum mismatch: original={:016X}, draft={:016X}",
                post_position_original_checksum, post_position_draft_checksum
            ),
        ));
    }

    #[cfg(debug_assertions)]
    {
        if post_bytes_verified > 0 {
            println!(
                "   ✓ Post-position bytes match with +1 frame-shift ({} bytes, checksum: {:016X})",
                post_bytes_verified, post_position_original_checksum
            );
        } else {
            println!("   ✓ No post-position bytes (insertion was at EOF)");
        }
    }

    // =========================================
    // Final Verification Summary
    // =========================================
    #[cfg(debug_assertions)]
    {
        println!("\n=== Verification Summary ===");
        println!(
            "✓ Total byte length: VERIFIED (original={}, draft={}, +1 byte)",
            original_size, draft_size
        );
        println!("✓ Pre-position similarity: VERIFIED");
        println!("✓ At-position insertion: VERIFIED");
        println!("✓ Post-position similarity: VERIFIED (with +1 frame-shift)");
        println!("All verification checks PASSED\n");
    }

    Ok(())
}

/// Performs a byte insertion operation on a file using a safe copy-and-replace strategy.
///
/// # Overview
/// This function inserts a single byte at a specified position in a file, causing all
/// subsequent bytes to shift forward by one position (frame-shift +1). It uses a defensive
/// "build-new-file" approach rather than modifying the original file directly.
///
/// # Memory Safety
/// - Uses pre-allocated 64-byte buffer (no heap allocation)
/// - Never loads entire file into memory
/// - Processes file chunk-by-chunk using bucket brigade pattern
/// - No dynamic memory allocation
///
/// # File Safety Strategy
/// 1. Creates a backup copy of the original file (.backup extension)
/// 2. Builds a new draft file (.draft extension) with the byte inserted
/// 3. Verifies the operation succeeded (including frame-shift verification)
/// 4. Atomically replaces original with draft
/// 5. Removes backup only after successful completion
///
/// # Operation Behavior - Mechanical Steps
/// The draft file is constructed by appending bytes sequentially:
///
/// **Step 1**: Create empty draft file
///
/// **Step 2**: Append pre-position bytes
/// - Read from original: positions 0 to `byte_position - 1`
/// - Append to draft: all these bytes
///
/// **Step 3**: Perform insertion AT position
/// - Draft file: append the new byte
/// - Original file: do NOT advance read position (stays at `byte_position`)
/// - Effect: The new byte is written at `byte_position` in draft
///
/// **Step 4**: Append post-position bytes
/// - Read from original: positions `byte_position` to EOF
/// - Append to draft: all remaining bytes
/// - Effect: These bytes naturally occupy positions starting at `byte_position + 1` in draft
/// - This creates the +1 frame-shift automatically
///
/// # Frame-Shift Behavior
/// After inserting byte at position N:
/// - Bytes 0 to N-1: unchanged positions
/// - Byte at N: the newly inserted byte
/// - Bytes N to EOF in original: all shift forward by 1 position (become N+1 to EOF+1 in draft)
/// - File length increases by exactly 1
///
/// # Parameters
/// - `original_file_path`: Absolute path to the file to modify
/// - `byte_position_from_start`: Zero-indexed position where byte will be inserted
/// - `new_byte_value`: The byte value to insert
///
/// # Position Semantics
/// Position represents an insertion point (gap), not an existing byte:
/// - Position 0: Insert before first byte
/// - Position N: Insert between byte N-1 and byte N
/// - Position file_size: Append after last byte (valid operation)
///
/// # Returns
/// - `Ok(())` on successful byte insertion
/// - `Err(io::Error)` if file operations fail or position is invalid
///
/// # Error Conditions
/// - File does not exist
/// - Byte position > file length (out of bounds)
/// - Insufficient permissions
/// - Disk full
/// - I/O errors during read/write
///
/// # Recovery Behavior
/// - If operation fails before replacing original, draft is removed, backup remains
/// - If atomic rename fails, both original and backup are preserved
/// - Orphaned .draft files indicate incomplete operations
/// - Orphaned .backup files indicate failed replacements
///
/// # Edge Cases
/// - Empty file at position 0: Results in single-byte file (valid operation)
/// - Position 0: Inserts before first byte, all bytes shift forward
/// - Position == file_size: Appends to end, no bytes shift (valid operation)
/// - Position > file_size: Returns error (cannot insert beyond EOF)
/// - Very large files: Processes in chunks, no memory issues
///
/// # Example
/// ```no_run
/// # use std::io;
/// # use std::path::PathBuf;
/// # fn add_single_byte_to_file(path: PathBuf, pos: usize, byte: u8) -> io::Result<()> { Ok(()) }
/// // Original file: [0x41, 0x42, 0x43]
/// let file_path = PathBuf::from("/absolute/path/to/file.dat");
/// let position = 1; // Insert between 0x41 and 0x42
/// let new_byte = 0xFF;
/// let result = add_single_byte_to_file(file_path, position, new_byte);
/// // Resulting file: [0x41, 0xFF, 0x42, 0x43]
/// // Note: 0x42 and 0x43 shifted forward by 1 position
/// assert!(result.is_ok());
/// # Ok::<(), io::Error>(())
/// ```
pub fn add_single_byte_to_file(
    original_file_path: PathBuf,
    byte_position_from_start: usize,
    new_byte_value: u8,
) -> io::Result<()> {
    // =========================================
    // Input Validation Phase
    // =========================================

    #[cfg(debug_assertions)]
    {
        println!("=== Byte Insertion Operation ===");
        println!("Target file: {}", original_file_path.display());
        println!("Insert position: {}", byte_position_from_start);
        println!("New byte value: 0x{:02X}", new_byte_value);
        println!();
    }

    // Verify file exists before any operations
    if !original_file_path.exists() {
        let error_message = format!(
            "Target file does not exist: {}",
            original_file_path.display()
        );
        #[cfg(debug_assertions)]
        eprintln!("ERROR: {}", error_message);
        return Err(io::Error::new(io::ErrorKind::NotFound, error_message));
    }

    // Verify file is actually a file, not a directory
    if !original_file_path.is_file() {
        let error_message = format!(
            "Target path is not a file: {}",
            original_file_path.display()
        );
        #[cfg(debug_assertions)]
        eprintln!("ERROR: {}", error_message);
        return Err(io::Error::new(io::ErrorKind::InvalidInput, error_message));
    }

    // Get original file metadata for validation
    let original_metadata = fs::metadata(&original_file_path)?;
    let original_file_size = original_metadata.len() as usize;

    // Validate byte position is within valid insertion range
    // Note: position == file_size is valid (append operation)
    if byte_position_from_start > original_file_size {
        let error_message = format!(
            "Byte position {} exceeds valid insertion range (0-{} for file size {})",
            byte_position_from_start, original_file_size, original_file_size
        );
        #[cfg(debug_assertions)]
        eprintln!("ERROR: {}", error_message);
        return Err(io::Error::new(io::ErrorKind::InvalidInput, error_message));
    }

    // =========================================
    // Path Construction Phase
    // =========================================

    // Build backup and draft file paths
    let backup_file_path = {
        let mut backup_path = original_file_path.clone();
        let file_name = backup_path
            .file_name()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid file name"))?
            .to_string_lossy();
        let backup_name = format!("{}.backup", file_name);
        backup_path.set_file_name(backup_name);
        backup_path
    };

    let draft_file_path = {
        let mut draft_path = original_file_path.clone();
        let file_name = draft_path
            .file_name()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid file name"))?
            .to_string_lossy();
        let draft_name = format!("{}.draft", file_name);
        draft_path.set_file_name(draft_name);
        draft_path
    };

    #[cfg(debug_assertions)]
    {
        println!("Backup path: {}", backup_file_path.display());
        println!("Draft path: {}", draft_file_path.display());
        println!();
    }

    // =========================================
    // Backup Creation Phase
    // =========================================

    #[cfg(debug_assertions)]
    println!("Creating backup copy...");

    fs::copy(&original_file_path, &backup_file_path).map_err(|e| {
        #[cfg(debug_assertions)]
        eprintln!("ERROR: Failed to create backup: {}", e);
        e
    })?;

    #[cfg(debug_assertions)]
    println!("Backup created successfully");

    // =========================================
    // Draft File Construction Phase
    // =========================================

    #[cfg(debug_assertions)]
    println!(
        "Building modified draft file (inserting byte at position {})...",
        byte_position_from_start
    );

    // Open original for reading
    let mut source_file = File::open(&original_file_path)?;

    // Create draft file for writing
    let mut draft_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&draft_file_path)?;

    // Pre-allocated buffer for bucket brigade operations
    const BUCKET_BRIGADE_BUFFER_SIZE: usize = 64;
    let mut bucket_brigade_buffer = [0u8; BUCKET_BRIGADE_BUFFER_SIZE];

    // =================================================
    // Debug-Assert, Test-Assert, Production-Catch-Handle
    // =================================================

    debug_assert!(
        BUCKET_BRIGADE_BUFFER_SIZE > 0,
        "Bucket brigade buffer must have non-zero size"
    );

    #[cfg(test)]
    {
        assert!(
            BUCKET_BRIGADE_BUFFER_SIZE > 0,
            "Bucket brigade buffer must have non-zero size"
        );
    }

    if BUCKET_BRIGADE_BUFFER_SIZE == 0 {
        let _ = fs::remove_file(&draft_file_path);
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid buffer configuration",
        ));
    }

    // Tracking variables
    let mut total_bytes_read_from_original: usize = 0;
    let mut total_bytes_written_to_draft: usize = 0;
    let mut chunk_number: usize = 0;
    let mut byte_was_inserted = false;

    // Safety limit to prevent infinite loops
    const MAX_CHUNKS_ALLOWED: usize = 16_777_216;

    // =========================================
    // Main Processing Loop
    // =========================================

    loop {
        // =================================================
        // Debug-Assert, Test-Assert, Production-Catch-Handle
        // =================================================

        debug_assert!(
            chunk_number < MAX_CHUNKS_ALLOWED,
            "Exceeded maximum chunk limit"
        );

        #[cfg(test)]
        {
            assert!(
                chunk_number < MAX_CHUNKS_ALLOWED,
                "Exceeded maximum chunk limit"
            );
        }

        if chunk_number >= MAX_CHUNKS_ALLOWED {
            #[cfg(debug_assertions)]
            eprintln!("ERROR: Maximum chunk limit exceeded for safety");
            let _ = fs::remove_file(&draft_file_path);
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "File too large or infinite loop detected",
            ));
        }

        // Clear buffer before reading (prevent data leakage)
        for i in 0..BUCKET_BRIGADE_BUFFER_SIZE {
            bucket_brigade_buffer[i] = 0;
        }

        chunk_number += 1;

        // Check if we need to insert the byte before reading next chunk
        if !byte_was_inserted && total_bytes_read_from_original == byte_position_from_start {
            // We've reached the insertion position
            // Insert the new byte BEFORE continuing to copy from original

            #[cfg(debug_assertions)]
            println!(
                "Inserting byte at position {}: 0x{:02X}",
                byte_position_from_start, new_byte_value
            );

            let insert_buffer = [new_byte_value];
            let bytes_written = draft_file.write(&insert_buffer)?;

            // =================================================
            // Debug-Assert, Test-Assert, Production-Catch-Handle
            // =================================================

            debug_assert_eq!(bytes_written, 1, "Failed to write inserted byte");

            #[cfg(test)]
            {
                assert_eq!(bytes_written, 1, "Failed to write inserted byte");
            }

            if bytes_written != 1 {
                #[cfg(debug_assertions)]
                eprintln!("ERROR: Failed to write inserted byte");
                let _ = fs::remove_file(&draft_file_path);
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Failed to write inserted byte",
                ));
            }

            total_bytes_written_to_draft += bytes_written;
            byte_was_inserted = true;
            draft_file.flush()?;

            // Continue to read and copy remaining bytes from original
        }

        // Read next chunk from source
        let bytes_read = source_file.read(&mut bucket_brigade_buffer)?;

        // EOF detection
        if bytes_read == 0 {
            #[cfg(debug_assertions)]
            println!("Reached end of original file");

            // Handle edge case: inserting at EOF (appending)
            if !byte_was_inserted {
                #[cfg(debug_assertions)]
                println!(
                    "Appending byte at EOF (position {}): 0x{:02X}",
                    byte_position_from_start, new_byte_value
                );

                let insert_buffer = [new_byte_value];
                let bytes_written = draft_file.write(&insert_buffer)?;

                if bytes_written != 1 {
                    #[cfg(debug_assertions)]
                    eprintln!("ERROR: Failed to append byte at EOF");
                    let _ = fs::remove_file(&draft_file_path);
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        "Failed to append byte at EOF",
                    ));
                }

                total_bytes_written_to_draft += bytes_written;
                byte_was_inserted = true;
                draft_file.flush()?;
            }

            break;
        }

        // =================================================
        // Debug-Assert, Test-Assert, Production-Catch-Handle
        // =================================================

        debug_assert!(
            bytes_read <= BUCKET_BRIGADE_BUFFER_SIZE,
            "Read more bytes than buffer size"
        );

        #[cfg(test)]
        {
            assert!(
                bytes_read <= BUCKET_BRIGADE_BUFFER_SIZE,
                "Read more bytes than buffer size"
            );
        }

        if bytes_read > BUCKET_BRIGADE_BUFFER_SIZE {
            #[cfg(debug_assertions)]
            eprintln!("ERROR: Buffer overflow detected");
            let _ = fs::remove_file(&draft_file_path);
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Buffer overflow in read operation",
            ));
        }

        // Determine if insertion point is in this chunk
        let chunk_start_position = total_bytes_read_from_original;
        let chunk_end_position = chunk_start_position + bytes_read;

        // Check if we need to insert a byte within this chunk
        if !byte_was_inserted
            && byte_position_from_start >= chunk_start_position
            && byte_position_from_start < chunk_end_position
        {
            // Calculate position within this chunk
            let position_in_chunk = byte_position_from_start - chunk_start_position;

            #[cfg(debug_assertions)]
            println!(
                "Inserting byte at position {}: 0x{:02X}",
                byte_position_from_start, new_byte_value
            );

            // Write bytes BEFORE the insertion position in this chunk
            if position_in_chunk > 0 {
                let bytes_before = &bucket_brigade_buffer[..position_in_chunk];
                let bytes_written_before = draft_file.write(bytes_before)?;

                // =================================================
                // Debug-Assert, Test-Assert, Production-Catch-Handle
                // =================================================

                debug_assert_eq!(
                    bytes_written_before, position_in_chunk,
                    "Not all pre-insertion bytes were written"
                );

                #[cfg(test)]
                {
                    assert_eq!(
                        bytes_written_before, position_in_chunk,
                        "Not all pre-insertion bytes were written"
                    );
                }

                if bytes_written_before != position_in_chunk {
                    #[cfg(debug_assertions)]
                    eprintln!("ERROR: Incomplete write before insertion position");
                    let _ = fs::remove_file(&draft_file_path);
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        "Incomplete write operation",
                    ));
                }

                total_bytes_written_to_draft += bytes_written_before;
            }

            // INSERT the new byte
            let insert_buffer = [new_byte_value];
            let bytes_written_insert = draft_file.write(&insert_buffer)?;

            if bytes_written_insert != 1 {
                #[cfg(debug_assertions)]
                eprintln!("ERROR: Failed to write inserted byte");
                let _ = fs::remove_file(&draft_file_path);
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Failed to write inserted byte",
                ));
            }

            total_bytes_written_to_draft += bytes_written_insert;
            byte_was_inserted = true;

            // Write bytes FROM the insertion position onward (these shift forward by 1)
            let bytes_from_position = &bucket_brigade_buffer[position_in_chunk..bytes_read];
            let bytes_written_after = draft_file.write(bytes_from_position)?;

            let expected_bytes_after = bytes_read - position_in_chunk;

            // =================================================
            // Debug-Assert, Test-Assert, Production-Catch-Handle
            // =================================================

            debug_assert_eq!(
                bytes_written_after, expected_bytes_after,
                "Not all post-insertion bytes were written"
            );

            #[cfg(test)]
            {
                assert_eq!(
                    bytes_written_after, expected_bytes_after,
                    "Not all post-insertion bytes were written"
                );
            }

            if bytes_written_after != expected_bytes_after {
                #[cfg(debug_assertions)]
                eprintln!("ERROR: Incomplete write after insertion position");
                let _ = fs::remove_file(&draft_file_path);
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Incomplete write operation",
                ));
            }

            total_bytes_written_to_draft += bytes_written_after;
        } else {
            // This chunk does not contain the insertion position
            // Write entire chunk to draft file
            let bytes_written = draft_file.write(&bucket_brigade_buffer[..bytes_read])?;

            // =================================================
            // Debug-Assert, Test-Assert, Production-Catch-Handle
            // =================================================

            debug_assert_eq!(bytes_written, bytes_read, "Not all bytes were written");

            #[cfg(test)]
            {
                assert_eq!(bytes_written, bytes_read, "Not all bytes were written");
            }

            if bytes_written != bytes_read {
                #[cfg(debug_assertions)]
                eprintln!(
                    "ERROR: Write mismatch - expected {} bytes, wrote {} bytes",
                    bytes_read, bytes_written
                );
                let _ = fs::remove_file(&draft_file_path);
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Incomplete write operation",
                ));
            }

            total_bytes_written_to_draft += bytes_written;
        }

        total_bytes_read_from_original += bytes_read;

        // Flush to ensure data is written
        draft_file.flush()?;
    }

    // =========================================
    // Basic Verification Phase
    // =========================================

    #[cfg(debug_assertions)]
    println!("\nVerifying operation...");

    // Verify byte was actually inserted
    if !byte_was_inserted {
        #[cfg(debug_assertions)]
        eprintln!("ERROR: Byte insertion did not occur");
        let _ = fs::remove_file(&draft_file_path);
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Byte insertion did not occur",
        ));
    }

    // Verify draft file is exactly 1 byte larger
    draft_file.flush()?;
    drop(draft_file);
    drop(source_file);

    let draft_metadata = fs::metadata(&draft_file_path)?;
    let draft_size = draft_metadata.len() as usize;
    let expected_draft_size = original_file_size + 1;

    // =================================================
    // Debug-Assert, Test-Assert, Production-Catch-Handle
    // =================================================

    debug_assert_eq!(draft_size, expected_draft_size, "Draft file size incorrect");

    #[cfg(test)]
    {
        assert_eq!(draft_size, expected_draft_size, "Draft file size incorrect");
    }

    if draft_size != expected_draft_size {
        #[cfg(debug_assertions)]
        eprintln!(
            "ERROR: File size mismatch - original: {} bytes, draft: {} bytes, expected: {} bytes",
            original_file_size, draft_size, expected_draft_size
        );
        let _ = fs::remove_file(&draft_file_path);
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "File size verification failed",
        ));
    }

    #[cfg(debug_assertions)]
    println!(
        "Basic verification passed: original={} bytes, draft={} bytes (+1 byte)",
        original_file_size, draft_size
    );

    // =========================================
    // Comprehensive Verification Phase
    // =========================================

    // Perform all verification checks before replacing the original
    verify_byte_addition_operation(
        &original_file_path,
        &draft_file_path,
        byte_position_from_start,
        new_byte_value,
    )?;

    // =========================================
    // Atomic Replacement Phase
    // =========================================

    #[cfg(debug_assertions)]
    println!("\nReplacing original file with modified version...");

    // Attempt atomic rename
    match fs::rename(&draft_file_path, &original_file_path) {
        Ok(()) => {
            #[cfg(debug_assertions)]
            println!("Original file successfully replaced");
        }
        Err(e) => {
            #[cfg(debug_assertions)]
            {
                eprintln!("Cannot atomically replace file: {}", e);
                eprintln!("Original and backup files preserved for safety");
            }
            return Err(e);
        }
    }

    // =========================================
    // Cleanup Phase
    // =========================================

    #[cfg(debug_assertions)]
    println!("\nCleaning up backup file...");

    match fs::remove_file(&backup_file_path) {
        Ok(()) => {
            #[cfg(debug_assertions)]
            println!("Backup file removed");
        }
        Err(e) => {
            #[cfg(debug_assertions)]
            {
                eprintln!(
                    "WARNING: Could not remove backup file: {} ({})",
                    backup_file_path.display(),
                    e
                );
                println!("Backup file retained at: {}", backup_file_path.display());
            }
        }
    }

    // =========================================
    // Operation Summary
    // =========================================

    #[cfg(debug_assertions)]
    {
        println!("\n=== Operation Complete ===");
        println!("File: {}", original_file_path.display());
        println!("Inserted byte at position: {}", byte_position_from_start);
        println!("Inserted byte value: 0x{:02X}", new_byte_value);
        println!("Original size: {} bytes", original_file_size);
        println!("New size: {} bytes", draft_size);
        println!(
            "Bytes read from original: {}",
            total_bytes_read_from_original
        );
        println!("Bytes written to draft: {}", total_bytes_written_to_draft);
        println!("Total chunks: {}", chunk_number);
        println!("Status: SUCCESS");
    }

    Ok(())
}

// =========================================
// Test Module
// =========================================

#[cfg(test)]
mod add_byte_tests {
    use super::*;

    #[test]
    fn test_add_single_byte_basic() {
        let test_dir = std::env::temp_dir();
        let test_file = test_dir.join("test_byte_add.bin");

        // Create test file: [0x00, 0x11, 0x22, 0x33]
        let test_data = vec![0x00, 0x11, 0x22, 0x33];
        std::fs::write(&test_file, &test_data).expect("Failed to create test file");

        // Insert byte 0xFF at position 2 (between 0x11 and 0x22)
        let result = add_single_byte_to_file(test_file.clone(), 2, 0xFF);

        assert!(result.is_ok(), "Operation should succeed");

        // Verify result: [0x00, 0x11, 0xFF, 0x22, 0x33]
        let modified_data = std::fs::read(&test_file).expect("Failed to read modified file");
        assert_eq!(modified_data, vec![0x00, 0x11, 0xFF, 0x22, 0x33]);

        // Cleanup
        let _ = std::fs::remove_file(&test_file);
    }

    #[test]
    fn test_add_byte_at_start() {
        let test_dir = std::env::temp_dir();
        let test_file = test_dir.join("test_add_start.bin");

        let test_data = vec![0xAA, 0xBB, 0xCC];
        std::fs::write(&test_file, &test_data).expect("Failed to create test file");

        // Insert at position 0 (before first byte)
        let result = add_single_byte_to_file(test_file.clone(), 0, 0xFF);

        assert!(result.is_ok());

        let modified_data = std::fs::read(&test_file).expect("Failed to read modified file");
        assert_eq!(modified_data, vec![0xFF, 0xAA, 0xBB, 0xCC]);

        let _ = std::fs::remove_file(&test_file);
    }

    #[test]
    fn test_add_byte_at_end() {
        let test_dir = std::env::temp_dir();
        let test_file = test_dir.join("test_add_end.bin");

        let test_data = vec![0xAA, 0xBB, 0xCC];
        std::fs::write(&test_file, &test_data).expect("Failed to create test file");

        // Insert at position 3 (append after last byte)
        let result = add_single_byte_to_file(test_file.clone(), 3, 0xFF);

        assert!(result.is_ok());

        let modified_data = std::fs::read(&test_file).expect("Failed to read modified file");
        assert_eq!(modified_data, vec![0xAA, 0xBB, 0xCC, 0xFF]);

        let _ = std::fs::remove_file(&test_file);
    }

    #[test]
    fn test_add_to_empty_file() {
        let test_dir = std::env::temp_dir();
        let test_file = test_dir.join("test_add_empty.bin");

        // Create empty file
        std::fs::write(&test_file, Vec::<u8>::new()).expect("Failed to create empty file");

        // Insert at position 0
        let result = add_single_byte_to_file(test_file.clone(), 0, 0x42);

        assert!(result.is_ok());

        let modified_data = std::fs::read(&test_file).expect("Failed to read modified file");
        assert_eq!(modified_data, vec![0x42]);

        let _ = std::fs::remove_file(&test_file);
    }

    #[test]
    fn test_add_byte_out_of_bounds() {
        let test_dir = std::env::temp_dir();
        let test_file = test_dir.join("test_add_bounds.bin");

        std::fs::write(&test_file, vec![0x00, 0x11]).expect("Failed to create test file");

        // Try to insert beyond EOF (position 10 when file has only 2 bytes)
        let result = add_single_byte_to_file(test_file.clone(), 10, 0xFF);

        assert!(result.is_err(), "Should fail with out of bounds position");

        let _ = std::fs::remove_file(&test_file);
    }
}

/*
/// Three Tests for basic operations
fn main() -> io::Result<()> {
    // Test 1: Hex-Edit Byte In-Place
    let test_dir_1 = std::env::current_dir()?;
    let original_file_path = test_dir_1.join("pytest_file_1.py");
    let byte_edit_position_from_start: usize = 3; // usize = 3;
    let new_byte_value: u8 = 0x61;

    // Run: In-Place-Edit
    let result_tui = replace_single_byte_in_file(
        original_file_path,
        byte_edit_position_from_start,
        new_byte_value,
    );
    println!("result_tui -> {:?}", result_tui);

    // Test 2: Remove Byte
    let test_dir_2 = std::env::current_dir()?;
    let original_file_path = test_dir_2.join("pytest_file_2.py");
    let byte_remove_position_from_start: usize = 3; // test usize = 3;

    // Run: Remove
    let result_tui =
        remove_single_byte_from_file(original_file_path, byte_remove_position_from_start);
    println!("result_tui -> {:?}", result_tui);

    // Test 3: Add Byte
    let test_dir_3 = std::env::current_dir()?;
    let original_file_path = test_dir_3.join("pytest_file_3.py");
    let byte_add_position_from_start: usize = 10; // test usize = 3;
    let new_add_byte_value: u8 = 0x61;

    // Run: Remove
    let result_tui = add_single_byte_to_file(
        original_file_path,
        byte_add_position_from_start,
        new_add_byte_value,
    );
    println!("result_tui -> {:?}", result_tui);

    println!("main() All Done!");
    Ok(())
}
*/

// ============================================================================
// CORE DATA STRUCTURES (Step 1A - START HERE)
// ============================================================================

/// Edit operation type for changelog entries
///
/// # Format
/// Three-letter lowercase strings for human readability:
/// - "add": Byte was added to file
/// - "rmv": Byte was removed from file
/// - "edt": Byte was replaced in-place (hex edit)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditType {
    /// Add byte operation (causes +1 frame-shift)
    Add,
    /// Remove byte operation (causes -1 frame-shift)
    Rmv,
    /// Edit byte in-place operation (no frame-shift)
    Edt,
}

// Constants
const MAX_UTF8_BYTES: usize = 4;

// ==========================================================
// ERROR SECTION: BUTTON UNDO CHANGELOG ERROR HANDLING SYSTEM
// ==========================================================
/*
# Sample integration

```
fn buttons_handle_user_edit(state: &mut EditorState) -> Result<()> {
    let target_file = state.get_current_file_path()?;
    let log_dir = state.get_changelog_directory()?;

    // Call Button function - error automatically converts to LinesError
    button_make_character_action_changelog(&target_file, Some('a'), 42, EditType::Add, &log_dir)?; // ButtonError converts to LinesError via From trait

    Ok(())
}
```

```
/// Automatic conversion from ButtonError to LinesError
impl From<ButtonError> for LinesError {
    fn from(err: ButtonError) -> Self {
        match err {
            // IO errors map directly
            ButtonError::Io(e) => LinesError::Io(e),

            // Log file issues are invalid input
            ButtonError::MalformedLog { .. } => {
                LinesError::InvalidInput("Malformed changelog file".into())
            }

            // UTF-8 errors map to UTF-8 error category
            ButtonError::InvalidUtf8 { .. } => {
                LinesError::Utf8Error("Invalid UTF-8 in changelog".into())
            }

            // Directory issues are state errors
            ButtonError::LogDirectoryError { .. } => {
                LinesError::StateError("Changelog directory error".into())
            }

            // No logs found is a state error
            ButtonError::NoLogsFound { .. } => {
                LinesError::StateError("No changelog files found".into())
            }

            // Position errors are invalid input
            ButtonError::PositionOutOfBounds { .. } => {
                LinesError::InvalidInput("Changelog position out of bounds".into())
            }

            // Incomplete log sets are state errors
            ButtonError::IncompleteLogSet { .. } => {
                LinesError::StateError("Incomplete changelog set".into())
            }

            // Assertion violations map to our catch-handle error
            ButtonError::AssertionViolation { check } => {
                LinesError::GeneralAssertionCatchViolation(
                    format!("Button system: {}", check).into()
                )
            }
        }
    }
}
```
*/

/// Error types for the Button Undo Changelog system
///
/// # Design Principles
/// - Focused on changelog file operations and UTF-8 character handling
/// - No heap allocation for production error paths (fixed strings)
/// - Maps cleanly to parent error systems (e.g., LinesError)
/// - Never panics - all errors return Result
#[derive(Debug)]
pub enum ButtonError {
    /// File system or I/O operation failed during log operations
    Io(io::Error),

    /// Log file is malformed or cannot be parsed
    /// Examples: missing position, invalid hex byte, wrong format
    MalformedLog {
        log_path: PathBuf,
        reason: &'static str, // Fixed string, no heap
    },

    /// UTF-8 character validation failed
    /// Examples: incomplete multi-byte sequence, invalid UTF-8
    InvalidUtf8 {
        position: u128,
        byte_count: usize,
        reason: &'static str,
    },

    /// Log directory structure issue
    /// Examples: missing directory, wrong naming convention
    LogDirectoryError { path: PathBuf, reason: &'static str },

    /// Cannot find next LIFO log file (empty log directory)
    NoLogsFound { log_dir: PathBuf },

    /// Position out of bounds for target file
    PositionOutOfBounds { position: u128, file_size: u128 },

    /// Multi-byte log set is incomplete or corrupted
    /// Example: Found 10.b and 10 but missing 10.a
    IncompleteLogSet {
        base_number: u128,
        found_logs: &'static str, // e.g., "10.b, 10" (fixed buffer)
    },

    /// For use with Assert-Catch-Handle system
    AssertionViolation { check: &'static str },
}

impl std::fmt::Display for ButtonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ButtonError::Io(e) => write!(f, "IO error: {}", e),

            // Production-safe: no sensitive path details
            #[cfg(not(debug_assertions))]
            ButtonError::MalformedLog { reason, .. } => {
                write!(f, "Log file error: {}", reason)
            }
            #[cfg(debug_assertions)]
            ButtonError::MalformedLog { log_path, reason } => {
                write!(f, "Malformed log {}: {}", log_path.display(), reason)
            }

            #[cfg(not(debug_assertions))]
            ButtonError::InvalidUtf8 { reason, .. } => {
                write!(f, "UTF-8 error: {}", reason)
            }
            #[cfg(debug_assertions)]
            ButtonError::InvalidUtf8 {
                position,
                byte_count,
                reason,
            } => {
                write!(
                    f,
                    "UTF-8 error at position {} ({} bytes): {}",
                    position, byte_count, reason
                )
            }

            #[cfg(not(debug_assertions))]
            ButtonError::LogDirectoryError { reason, .. } => {
                write!(f, "Log directory error: {}", reason)
            }
            #[cfg(debug_assertions)]
            ButtonError::LogDirectoryError { path, reason } => {
                write!(f, "Log directory error {}: {}", path.display(), reason)
            }

            #[cfg(not(debug_assertions))]
            ButtonError::NoLogsFound { .. } => {
                write!(f, "No changelog files found")
            }
            #[cfg(debug_assertions)]
            ButtonError::NoLogsFound { log_dir } => {
                write!(f, "No logs found in {}", log_dir.display())
            }

            ButtonError::PositionOutOfBounds {
                position,
                file_size,
            } => {
                write!(f, "Position {} exceeds file size {}", position, file_size)
            }

            ButtonError::IncompleteLogSet {
                base_number,
                found_logs,
            } => {
                write!(
                    f,
                    "Incomplete log set {}: found {}",
                    base_number, found_logs
                )
            }

            ButtonError::AssertionViolation { check } => {
                write!(f, "Assertion violation: {}", check)
            }
        }
    }
}

impl std::error::Error for ButtonError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ButtonError::Io(e) => Some(e),
            _ => None,
        }
    }
}

/// Automatic conversion from io::Error to ButtonError
impl From<io::Error> for ButtonError {
    fn from(err: io::Error) -> Self {
        ButtonError::Io(err)
    }
}

/// Result type alias for Button changelog operations
pub type ButtonResult<T> = std::result::Result<T, ButtonError>;

// ============================================================================
// ERROR SECTION: BUTTON UNDO CHANGELOG ERROR HANDLING SYSTEM (end)
// ============================================================================

/// Moves a corrupted log file to error log directory
///
/// # Purpose
/// - Remove bad log from active changelog directory
/// - Preserve evidence for debugging
/// - Never crash on failure
///
/// # Arguments
/// * `target_file` - File being edited (for error log naming)
/// * `bad_log_path` - Path to corrupted log file
/// * `reason` - Why the log is being moved (e.g., "malformed_format")
pub fn quarantine_bad_log(target_file: &Path, bad_log_path: &Path, reason: &str) {
    // Build error log directory with timestamp
    let file_stem = match target_file.file_stem() {
        Some(stem) => stem.to_string_lossy(),
        None => {
            #[cfg(debug_assertions)]
            eprintln!("WARNING: Cannot quarantine log - invalid target file");
            return;
        }
    };

    let error_log_dir = match target_file.parent() {
        Some(parent) => parent.join(format!("undoredo_errorlogs_{}", file_stem)),
        None => {
            #[cfg(debug_assertions)]
            eprintln!("WARNING: Cannot determine error log directory");
            return;
        }
    };

    // Get timestamp (NO HEAP)
    let (timestamp_buffer, timestamp_len) = get_timestamp_for_error_log_no_heap();

    // Convert to string slice
    let timestamp_str = match timestamp_buffer_to_str(&timestamp_buffer, timestamp_len) {
        Ok(s) => s,
        Err(_) => {
            #[cfg(debug_assertions)]
            eprintln!("WARNING: Invalid timestamp encoding");
            return;
        }
    };

    let timestamp_dir = error_log_dir.join(timestamp_str);

    // Create error log directory
    if let Err(e) = fs::create_dir_all(&timestamp_dir) {
        #[cfg(debug_assertions)]
        eprintln!("WARNING: Cannot create quarantine directory: {}", e);
        return;
    }

    // Move log file to error directory
    let log_filename = match bad_log_path.file_name() {
        Some(name) => name,
        None => {
            #[cfg(debug_assertions)]
            eprintln!("WARNING: Cannot determine log filename");
            return;
        }
    };

    let destination = timestamp_dir.join(log_filename);

    if let Err(e) = fs::rename(bad_log_path, &destination) {
        #[cfg(debug_assertions)]
        eprintln!("WARNING: Cannot move corrupted log: {}", e);

        // Try to at least log what happened
        log_button_error(
            target_file,
            &format!("Failed to quarantine log: {}", reason),
            Some("quarantine_bad_log"),
        );
    } else {
        #[cfg(debug_assertions)]
        println!("Quarantined log to: {}", destination.display());

        // Log successful quarantine
        log_button_error(
            target_file,
            &format!("Quarantined log: {}", reason),
            Some("quarantine_bad_log"),
        );
    }
}

/// Logs Button changelog errors to dedicated error log directory
///
/// # Purpose
/// - Separate error logs from main Lines editor logs
/// - Never panics or interrupts operation
/// - Uses target file name to organize logs
/// - **NO HEAP ALLOCATION in core logic** (production-safe)
///
/// # Arguments
/// * `target_file` - The file being edited (for log directory naming)
/// * `error_msg` - The error message to log
/// * `context` - Optional context (e.g., "undo_operation", "log_creation")
///
/// # Memory Safety
/// - Fixed stack buffers for timestamp
/// - Minimal heap use only for I/O formatting
/// - Debug builds may use heap for verbose output
pub fn log_button_error(target_file: &Path, error_msg: &str, context: Option<&str>) {
    // Extract filename without extension for directory name
    let file_stem = match target_file.file_stem() {
        Some(stem) => stem.to_string_lossy(),
        None => {
            #[cfg(debug_assertions)]
            eprintln!("WARNING: Cannot determine filename for error log");
            eprintln!("ERROR: {}", error_msg);
            return;
        }
    };

    // Build error log directory path
    let error_log_dir = match target_file.parent() {
        Some(parent) => parent.join(format!("undoredo_errorlogs_{}", file_stem)),
        None => {
            #[cfg(debug_assertions)]
            eprintln!("WARNING: Cannot determine parent directory");
            eprintln!("ERROR: {}", error_msg);
            return;
        }
    };

    // Get timestamp (NO HEAP for timestamp generation)
    let (timestamp_buffer, timestamp_len) = get_timestamp_for_error_log_no_heap();

    // Convert to string slice (validates UTF-8)
    let timestamp_str = match timestamp_buffer_to_str(&timestamp_buffer, timestamp_len) {
        Ok(s) => s,
        Err(_) => {
            #[cfg(debug_assertions)]
            eprintln!("WARNING: Invalid timestamp encoding");
            return;
        }
    };

    // Create timestamped subdirectory
    let timestamp_dir = error_log_dir.join(timestamp_str);

    if let Err(e) = fs::create_dir_all(&timestamp_dir) {
        #[cfg(debug_assertions)]
        eprintln!("WARNING: Cannot create error log directory: {}", e);
        eprintln!("ERROR: {}", error_msg);
        return;
    }

    // Build error log file path
    let error_log_file = timestamp_dir.join("error.log");

    // Format log entry (minimal heap use for I/O buffer only)
    let log_entry = if let Some(ctx) = context {
        format!("[{}] [{}] {}\n", timestamp_str, ctx, error_msg)
    } else {
        format!("[{}] {}\n", timestamp_str, error_msg)
    };

    // Attempt to write
    match OpenOptions::new()
        .create(true)
        .append(true)
        .open(&error_log_file)
    {
        Ok(mut file) => {
            if let Err(e) = file.write_all(log_entry.as_bytes()) {
                #[cfg(debug_assertions)]
                eprintln!("WARNING: Cannot write to error log: {}", e);
                eprintln!("ERROR: {}", error_msg);
            }
            let _ = file.flush();
        }
        Err(e) => {
            #[cfg(debug_assertions)]
            eprintln!("WARNING: Cannot open error log: {}", e);
            eprintln!("ERROR: {}", error_msg);
        }
    }
}

/// Gets timestamp string for error logging (NO HEAP)
///
/// # Memory Safety
/// - Fixed 32-byte stack buffer
/// - No heap allocation
/// - Production-safe
///
/// # Format
/// Unix epoch seconds as decimal string
/// Example: "1704067200" (fits in 10 chars for years 1970-2286)
///
/// # Returns
/// * `([u8; 32], usize)` - Fixed buffer and length of valid data
fn get_timestamp_for_error_log_no_heap() -> ([u8; 32], usize) {
    use std::time::{SystemTime, UNIX_EPOCH};

    let secs = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_secs(),
        Err(_) => 0, // Fallback for time before epoch
    };

    // Convert u64 to decimal string on stack
    let mut buffer = [0u8; 32];
    let mut temp = secs;
    let mut len = 0;

    // Handle zero case
    if temp == 0 {
        buffer[0] = b'0';
        return (buffer, 1);
    }

    // Extract digits in reverse (least significant first)
    let mut digits = [0u8; 20]; // Max digits for u64
    let mut digit_count = 0;

    // Bounded loop: max 20 iterations (u64 max is ~19 digits)
    while temp > 0 && digit_count < 20 {
        digits[digit_count] = (temp % 10) as u8 + b'0';
        temp /= 10;
        digit_count += 1;
    }

    // Reverse into buffer (most significant first)
    // Bounded loop: max 20 iterations
    for i in 0..digit_count {
        buffer[i] = digits[digit_count - 1 - i];
        len += 1;
    }

    (buffer, len)
}

/// Helper to convert fixed timestamp buffer to &str
///
/// # Safety
/// Only returns the valid portion of the buffer
///
/// # Arguments
/// * `buffer` - Fixed 32-byte buffer containing ASCII digits
/// * `len` - Length of valid data in buffer
///
/// # Returns
/// * `Result<&str, std::str::Utf8Error>` - String slice or encoding error
fn timestamp_buffer_to_str(buffer: &[u8; 32], len: usize) -> Result<&str, std::str::Utf8Error> {
    std::str::from_utf8(&buffer[..len])
}

// ============================================================================
// CORE DATA STRUCTURES: LogEntry and Helper Functions
// ============================================================================

// ============================================================================
// CORE DATA STRUCTURES (Step 1A - CONTINUED)
// ============================================================================

/// Represents a single changelog entry for one byte operation
///
/// # Purpose
/// Stores the information needed to UNDO a single byte-level edit.
/// This is the INVERSE of what the user did.
///
/// # Memory Layout
/// - Fixed size: 1 byte (EditType) + 16 bytes (u128) + 1 byte (Option<u8>) = ~18 bytes
/// - No heap allocation
/// - Stack-only storage
///
/// # Changelog Logic Examples
///
/// **User adds byte 0x48 ('H') at position 100:**
/// - User action: Add 0x48
/// - LogEntry stores: `Rmv` at position 100 (no byte needed)
/// - Undo operation: Remove the byte that was added
///
/// **User removes byte 0x48 ('H') from position 100:**
/// - User action: Remove 0x48
/// - LogEntry stores: `Add` 0x48 at position 100
/// - Undo operation: Add back the byte that was removed
///
/// **User hex-edits position 100 from 0xFF to 0x61:**
/// - User action: Edit 0xFF → 0x61
/// - LogEntry stores: `Edt` 0xFF at position 100
/// - Undo operation: Edit back to original value 0xFF
///
/// # File Format
/// Serialized as 2-3 lines:
/// ```text
/// add      ← Edit type (3 letters)
/// 100      ← Position (decimal u128)
/// 48       ← Byte value (2-char hex, omitted for Rmv)
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LogEntry {
    /// Type of edit operation to perform for undo
    /// - Add: Insert this byte (undoes a user remove)
    /// - Rmv: Delete this byte (undoes a user add)
    /// - Edt: Replace with this byte (undoes a user hex-edit)
    edit_type: EditType,

    /// Byte position in target file (0-indexed)
    /// Uses u128 to support very large files
    position: u128,

    /// The byte value for undo operation
    /// - Some(byte): For Add and Edt operations
    /// - None: For Rmv operations (no byte needed to delete)
    byte_value: Option<u8>,
}

impl LogEntry {
    /// Creates a new log entry
    ///
    /// # Arguments
    /// * `edit_type` - Type of undo operation
    /// * `position` - File position for operation
    /// * `byte_value` - Byte value (Some for Add/Edt, None for Rmv)
    ///
    /// # Returns
    /// * `Result<LogEntry, &'static str>` - New log entry or error message
    ///
    /// # Validation
    /// - Rmv must have None for byte_value
    /// - Add and Edt must have Some for byte_value
    ///
    /// # Examples
    /// ```
    /// // Create log to undo user's addition of 'H' at position 42
    /// let log = LogEntry::new(EditType::Rmv, 42, None)?;
    ///
    /// // Create log to undo user's removal of 'H' at position 42
    /// let log = LogEntry::new(EditType::Add, 42, Some(0x48))?;
    ///
    /// // Create log to undo user's hex-edit (0xFF→0x61) at position 42
    /// let log = LogEntry::new(EditType::Edt, 42, Some(0xFF))?;
    /// ```
    pub fn new(
        edit_type: EditType,
        position: u128,
        byte_value: Option<u8>,
    ) -> Result<Self, &'static str> {
        // =================================================
        // Debug-Assert, Test-Assert, Production-Catch-Handle
        // =================================================

        // Validation: Rmv must not have a byte value
        debug_assert!(
            !(edit_type == EditType::Rmv && byte_value.is_some()),
            "Rmv operation must not have byte_value"
        );

        #[cfg(test)]
        assert!(
            !(edit_type == EditType::Rmv && byte_value.is_some()),
            "Rmv operation must not have byte_value"
        );

        if edit_type == EditType::Rmv && byte_value.is_some() {
            return Err("Rmv operation must not have byte_value");
        }

        // Validation: Add and Edt must have a byte value
        debug_assert!(
            !(matches!(edit_type, EditType::Add | EditType::Edt) && byte_value.is_none()),
            "Add/Edt operations must have byte_value"
        );

        #[cfg(test)]
        assert!(
            !(matches!(edit_type, EditType::Add | EditType::Edt) && byte_value.is_none()),
            "Add/Edt operations must have byte_value"
        );

        if matches!(edit_type, EditType::Add | EditType::Edt) && byte_value.is_none() {
            return Err("Add/Edt operations must have byte_value");
        }

        Ok(LogEntry {
            edit_type,
            position,
            byte_value,
        })
    }

    /// Gets the edit type for this log entry
    pub fn edit_type(&self) -> EditType {
        self.edit_type
    }

    /// Gets the file position for this operation
    pub fn position(&self) -> u128 {
        self.position
    }

    /// Gets the byte value (if present)
    pub fn byte_value(&self) -> Option<u8> {
        self.byte_value
    }
}

// ============================================================================
// EDIT TYPE SERIALIZATION/DESERIALIZATION
// ============================================================================

impl EditType {
    /// Converts EditType to 3-letter string for log files
    ///
    /// # Returns
    /// * `&'static str` - Fixed string, no heap allocation
    ///
    /// # Format
    /// - Add → "add"
    /// - Rmv → "rmv"
    /// - Edt → "edt"
    pub fn as_str(self) -> &'static str {
        match self {
            EditType::Add => "add",
            EditType::Rmv => "rmv",
            EditType::Edt => "edt",
        }
    }

    /// Parses 3-letter string into EditType
    ///
    /// # Arguments
    /// * `s` - String slice to parse (should be 3 characters)
    ///
    /// # Returns
    /// * `Result<EditType, &'static str>` - Parsed type or error message
    ///
    /// # Accepted Input
    /// - "add" → EditType::Add
    /// - "rmv" → EditType::Rmv
    /// - "edt" → EditType::Edt
    /// - Case-sensitive (must be lowercase)
    ///
    /// # Errors
    /// - Returns error for any other input
    pub fn from_str(s: &str) -> Result<Self, &'static str> {
        match s {
            "add" => Ok(EditType::Add),
            "rmv" => Ok(EditType::Rmv),
            "edt" => Ok(EditType::Edt),
            _ => Err("Invalid edit type string (must be 'add', 'rmv', or 'edt')"),
        }
    }
}

// ============================================================================
// LOG ENTRY SERIALIZATION/DESERIALIZATION
// ============================================================================

impl LogEntry {
    /// Serializes log entry to file format
    ///
    /// # Format
    /// ```text
    /// add      ← Line 1: edit type (3 letters)
    /// 12345    ← Line 2: position (decimal)
    /// FF       ← Line 3: byte hex (only for add/edt)
    /// ```
    ///
    /// # Returns
    /// * `String` - Serialized log entry (uses heap for flexibility)
    ///
    /// # Note on Heap Usage
    /// This uses String (heap) for simplicity in writing to files.
    /// The heap usage is minimal (< 50 bytes) and only during I/O.
    ///
    /// # Examples
    /// ```
    /// let log = LogEntry::new(EditType::Add, 42, Some(0x48))?;
    /// let serialized = log.to_file_format();
    /// // Result: "add\n42\n48\n"
    /// ```
    pub fn to_file_format(&self) -> String {
        let mut result = String::with_capacity(32); // Pre-allocate reasonable size

        // Line 1: Edit type
        result.push_str(self.edit_type.as_str());
        result.push('\n');

        // Line 2: Position (decimal)
        result.push_str(&self.position.to_string());
        result.push('\n');

        // Line 3: Byte value (hex, only for add/edt)
        if let Some(byte) = self.byte_value {
            result.push_str(&format!("{:02X}", byte));
            result.push('\n');
        }

        result
    }

    /// Deserializes log entry from file format
    ///
    /// # Arguments
    /// * `content` - File content as string
    ///
    /// # Returns
    /// * `Result<LogEntry, &'static str>` - Parsed log entry or error
    ///
    /// # Expected Format
    /// 2-3 lines:
    /// 1. Edit type: "add", "rmv", or "edt"
    /// 2. Position: decimal number (e.g., "12345")
    /// 3. Byte hex: two hex digits (e.g., "FF") - only for add/edt
    ///
    /// # Errors
    /// - Missing lines
    /// - Invalid edit type
    /// - Invalid position (not a number)
    /// - Invalid hex byte (not 2 hex digits)
    /// - Missing byte for add/edt
    /// - Unexpected byte for rmv
    ///
    /// # Examples
    /// ```
    /// let content = "add\n42\n48\n";
    /// let log = LogEntry::from_file_format(content)?;
    /// assert_eq!(log.edit_type(), EditType::Add);
    /// assert_eq!(log.position(), 42);
    /// assert_eq!(log.byte_value(), Some(0x48));
    /// ```
    pub fn from_file_format(content: &str) -> Result<Self, &'static str> {
        // Split into lines
        let lines: Vec<&str> = content.lines().collect();

        // Validation: must have at least 2 lines
        if lines.len() < 2 {
            return Err("Log file must have at least 2 lines (type and position)");
        }

        // Parse line 1: Edit type
        let edit_type = EditType::from_str(lines[0].trim())?;

        // Parse line 2: Position
        let position = lines[1]
            .trim()
            .parse::<u128>()
            .map_err(|_| "Invalid position: must be a decimal number")?;

        // Parse line 3 (if present): Byte value
        let byte_value = if lines.len() >= 3 {
            let hex_str = lines[2].trim();

            // Validation: must be exactly 2 hex digits
            if hex_str.len() != 2 {
                return Err("Byte value must be exactly 2 hex digits");
            }

            let byte =
                u8::from_str_radix(hex_str, 16).map_err(|_| "Invalid hex byte: must be 00-FF")?;

            Some(byte)
        } else {
            None
        };

        // Validation: Check consistency
        match edit_type {
            EditType::Rmv => {
                if byte_value.is_some() {
                    return Err("Rmv operation must not have byte value");
                }
            }
            EditType::Add | EditType::Edt => {
                if byte_value.is_none() {
                    return Err("Add/Edt operations must have byte value");
                }
            }
        }

        // Use validated constructor
        LogEntry::new(edit_type, position, byte_value)
    }
}

// ============================================================================
// CONSTANTS FOR LOG FILE NAMING
// ============================================================================

/// Maximum number of bytes in a UTF-8 character
// pub const MAX_UTF8_BYTES: usize = 4;

/// Letters used for multi-byte log file naming (a-z)
/// Used to create sequences like: 10.c, 10.b, 10.a, 10
pub const LOG_LETTER_SEQUENCE: [char; 26] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z',
];

/// Log directory name prefix
/// Full name format: "changelog_{filename_without_extension}"
pub const LOG_DIR_PREFIX: &str = "changelog_";

/// Redo log directory name prefix
/// Full name format: "changelog_redo_{filename_without_extension}"
pub const REDO_LOG_DIR_PREFIX: &str = "changelog_redo_";

/// Error log directory name prefix
/// Full name format: "undoredo_errorlogs_{filename_without_extension}"
// pub const ERROR_LOG_DIR_PREFIX: &str = "undoredo_errorlogs_";

/// Gets the letter suffix for a multi-byte log file
///
/// # Purpose
/// For multi-byte UTF-8 characters, we need to create a sequence of log files
/// with letter suffixes to maintain LIFO ordering.
///
/// # Arguments
/// * `byte_index` - Index of byte in character (0 = first, 3 = last)
/// * `total_bytes` - Total number of bytes in character (1-4)
///
/// # Returns
/// * `Option<char>` - Letter suffix, or None for the last byte (no extension)
///
/// # LIFO Stack Logic ("Cheap Trick" Button Approach)
/// For a 3-byte character at position 20:
/// - Byte 0 (first):  File "20"   (no letter, last in stack, first out)
/// - Byte 1 (middle): File "20.a" (letter 'a')
/// - Byte 2 (last):   File "20.b" (letter 'b', first in stack, last out)
///
/// The LAST byte gets the HIGHEST letter (goes in stack first).
/// The FIRST byte gets NO letter (goes in stack last, comes out first).
///
/// # Examples
/// ```
/// // 3-byte character: E9 98 BF
/// assert_eq!(get_log_file_letter_suffix(0, 3), None);      // First byte: "20"
/// assert_eq!(get_log_file_letter_suffix(1, 3), Some('a')); // Second byte: "20.a"
/// assert_eq!(get_log_file_letter_suffix(2, 3), Some('b')); // Third byte: "20.b"
/// ```
pub fn get_log_file_letter_suffix(byte_index: usize, total_bytes: usize) -> Option<char> {
    // =================================================
    // Debug-Assert, Test-Assert, Production-Catch-Handle
    // =================================================

    debug_assert!(
        total_bytes >= 1 && total_bytes <= MAX_UTF8_BYTES,
        "total_bytes must be 1-4"
    );

    #[cfg(test)]
    assert!(
        total_bytes >= 1 && total_bytes <= MAX_UTF8_BYTES,
        "total_bytes must be 1-4"
    );

    if total_bytes < 1 || total_bytes > MAX_UTF8_BYTES {
        // Production: return None as safe fallback
        return None;
    }

    debug_assert!(
        byte_index < total_bytes,
        "byte_index must be less than total_bytes"
    );

    #[cfg(test)]
    assert!(
        byte_index < total_bytes,
        "byte_index must be less than total_bytes"
    );

    if byte_index >= total_bytes {
        // Production: return None as safe fallback
        return None;
    }

    // Single-byte character: no letter suffix
    if total_bytes == 1 {
        return None;
    }

    // First byte (index 0): no letter (last in stack, first out)
    if byte_index == 0 {
        return None;
    }

    // Other bytes: assign letters starting from 'a'
    // byte_index 1 → 'a', byte_index 2 → 'b', byte_index 3 → 'c'
    let letter_index = byte_index - 1;
    Some(LOG_LETTER_SEQUENCE[letter_index])
}

// ============================================================================
// UNIT TESTS
// ============================================================================

#[cfg(test)]
mod log_entry_tests {
    use super::*;

    #[test]
    fn test_edit_type_serialization() {
        assert_eq!(EditType::Add.as_str(), "add");
        assert_eq!(EditType::Rmv.as_str(), "rmv");
        assert_eq!(EditType::Edt.as_str(), "edt");
    }

    #[test]
    fn test_edit_type_deserialization() {
        assert_eq!(EditType::from_str("add").unwrap(), EditType::Add);
        assert_eq!(EditType::from_str("rmv").unwrap(), EditType::Rmv);
        assert_eq!(EditType::from_str("edt").unwrap(), EditType::Edt);

        assert!(EditType::from_str("invalid").is_err());
        assert!(EditType::from_str("ADD").is_err()); // Case-sensitive
    }

    #[test]
    fn test_log_entry_creation_valid() {
        // Valid Rmv (no byte)
        let rmv_log = LogEntry::new(EditType::Rmv, 42, None);
        assert!(rmv_log.is_ok());

        // Valid Add (with byte)
        let add_log = LogEntry::new(EditType::Add, 100, Some(0x48));
        assert!(add_log.is_ok());

        // Valid Edt (with byte)
        let edt_log = LogEntry::new(EditType::Edt, 200, Some(0xFF));
        assert!(edt_log.is_ok());
    }

    // // TODO fix test, conflicts with assert?
    // #[test]
    // fn test_log_entry_creation_invalid() {
    //     // Invalid: Rmv with byte
    //     let invalid_rmv = LogEntry::new(EditType::Rmv, 42, Some(0x48));
    //     assert!(invalid_rmv.is_err());

    //     // Invalid: Add without byte
    //     let invalid_add = LogEntry::new(EditType::Add, 100, None);
    //     assert!(invalid_add.is_err());

    //     // Invalid: Edt without byte
    //     let invalid_edt = LogEntry::new(EditType::Edt, 200, None);
    //     assert!(invalid_edt.is_err());
    // }

    #[test]
    fn test_log_entry_serialization() {
        // Test Add
        let add_log = LogEntry::new(EditType::Add, 42, Some(0x48)).unwrap();
        let serialized = add_log.to_file_format();
        assert_eq!(serialized, "add\n42\n48\n");

        // Test Rmv (no byte line)
        let rmv_log = LogEntry::new(EditType::Rmv, 100, None).unwrap();
        let serialized = rmv_log.to_file_format();
        assert_eq!(serialized, "rmv\n100\n");

        // Test Edt
        let edt_log = LogEntry::new(EditType::Edt, 200, Some(0xFF)).unwrap();
        let serialized = edt_log.to_file_format();
        assert_eq!(serialized, "edt\n200\nFF\n");
    }

    #[test]
    fn test_log_entry_deserialization() {
        // Test Add
        let content = "add\n42\n48\n";
        let log = LogEntry::from_file_format(content).unwrap();
        assert_eq!(log.edit_type(), EditType::Add);
        assert_eq!(log.position(), 42);
        assert_eq!(log.byte_value(), Some(0x48));

        // Test Rmv
        let content = "rmv\n100\n";
        let log = LogEntry::from_file_format(content).unwrap();
        assert_eq!(log.edit_type(), EditType::Rmv);
        assert_eq!(log.position(), 100);
        assert_eq!(log.byte_value(), None);

        // Test Edt
        let content = "edt\n200\nFF\n";
        let log = LogEntry::from_file_format(content).unwrap();
        assert_eq!(log.edit_type(), EditType::Edt);
        assert_eq!(log.position(), 200);
        assert_eq!(log.byte_value(), Some(0xFF));
    }

    #[test]
    fn test_log_entry_roundtrip() {
        let original = LogEntry::new(EditType::Add, 12345, Some(0xAB)).unwrap();
        let serialized = original.to_file_format();
        let deserialized = LogEntry::from_file_format(&serialized).unwrap();

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_get_log_file_letter_suffix() {
        // Single-byte: no letter
        assert_eq!(get_log_file_letter_suffix(0, 1), None);

        // 2-byte: first=none, second='a'
        assert_eq!(get_log_file_letter_suffix(0, 2), None);
        assert_eq!(get_log_file_letter_suffix(1, 2), Some('a'));

        // 3-byte: first=none, second='a', third='b'
        assert_eq!(get_log_file_letter_suffix(0, 3), None);
        assert_eq!(get_log_file_letter_suffix(1, 3), Some('a'));
        assert_eq!(get_log_file_letter_suffix(2, 3), Some('b'));

        // 4-byte: first=none, second='a', third='b', fourth='c'
        assert_eq!(get_log_file_letter_suffix(0, 4), None);
        assert_eq!(get_log_file_letter_suffix(1, 4), Some('a'));
        assert_eq!(get_log_file_letter_suffix(2, 4), Some('b'));
        assert_eq!(get_log_file_letter_suffix(3, 4), Some('c'));
    }
}

// ============================================================================
// LOG FILE OPERATIONS - SINGLE-BYTE LOG CREATION
// ============================================================================

/// Gets the next available log file number in a directory
///
/// # Purpose
/// Finds the highest-numbered log file and returns the next number for LIFO ordering.
/// Scans directory for files matching pattern: digits with optional letter suffix.
///
/// # Arguments
/// * `log_dir` - Directory to scan for existing log files
///
/// # Returns
/// * `ButtonResult<u128>` - Next available log number (0 if directory is empty)
///
/// # Behavior
/// - Returns 0 if directory doesn't exist (will be created)
/// - Returns 0 if directory is empty
/// - Returns highest_number + 1 if logs exist
/// - Ignores non-log files (must start with digits)
///
/// # Examples
/// ```
/// // Directory contains: 0, 1, 2, 2.a, 3
/// // Returns: 4
/// let next = get_next_log_number(&log_dir)?;
/// assert_eq!(next, 4);
/// ```
fn get_next_log_number(log_dir: &Path) -> ButtonResult<u128> {
    // If directory doesn't exist, start at 0
    if !log_dir.exists() {
        return Ok(0);
    }

    // =================================================
    // Debug-Assert, Test-Assert, Production-Catch-Handle
    // =================================================

    debug_assert!(log_dir.is_dir(), "log_dir must be a directory");

    #[cfg(test)]
    assert!(log_dir.is_dir(), "log_dir must be a directory");

    if !log_dir.is_dir() {
        return Err(ButtonError::LogDirectoryError {
            path: log_dir.to_path_buf(),
            reason: "Path exists but is not a directory",
        });
    }

    let mut max_number: u128 = 0;
    let mut found_any_log = false;

    // Read directory entries
    let entries = fs::read_dir(log_dir).map_err(|e| ButtonError::Io(e))?;

    // Bounded loop: iterate through directory entries
    // Upper bound: reasonable filesystem limits (millions of files)
    const MAX_DIR_ENTRIES: usize = 10_000_000;
    let mut entry_count: usize = 0;

    for entry_result in entries {
        // =================================================
        // Debug-Assert, Test-Assert, Production-Catch-Handle
        // =================================================

        debug_assert!(
            entry_count < MAX_DIR_ENTRIES,
            "Directory entry count exceeded safety limit"
        );

        #[cfg(test)]
        assert!(
            entry_count < MAX_DIR_ENTRIES,
            "Directory entry count exceeded safety limit"
        );

        if entry_count >= MAX_DIR_ENTRIES {
            return Err(ButtonError::LogDirectoryError {
                path: log_dir.to_path_buf(),
                reason: "Too many directory entries (safety limit)",
            });
        }

        entry_count += 1;

        let entry = entry_result.map_err(|e| ButtonError::Io(e))?;
        let filename = entry.file_name();
        let filename_str = filename.to_string_lossy();

        // Parse filename: should be number or number.letter
        // Extract the numeric part before any '.'
        let numeric_part = if let Some(dot_pos) = filename_str.find('.') {
            &filename_str[..dot_pos]
        } else {
            &filename_str[..]
        };

        // Try to parse as u128
        if let Ok(number) = numeric_part.parse::<u128>() {
            found_any_log = true;
            if number > max_number {
                max_number = number;
            }
        }
        // Ignore files that don't match our naming pattern
    }

    // Return next number (0 if no logs found, max+1 otherwise)
    if found_any_log {
        Ok(max_number.saturating_add(1))
    } else {
        Ok(0)
    }
}

/// Creates a single-byte log file in the specified directory
///
/// # Purpose
/// Internal helper function that writes a LogEntry to a numbered file.
/// Handles directory creation and file writing.
///
/// # Arguments
/// * `target_file` - File being edited (for error logging)
/// * `log_dir` - Directory to write log file
/// * `log_entry` - The log entry to write
///
/// # Returns
/// * `ButtonResult<()>` - Success or error
///
/// # Behavior
/// - Creates log directory if it doesn't exist
/// - Gets next available log number
/// - Writes log entry to file "{number}"
/// - Uses absolute paths for safety
///
/// # File Format
/// Creates file like "0", "1", "2", etc. containing:
/// ```text
/// add
/// 12345
/// FF
/// ```
fn write_log_entry_to_file(
    target_file: &Path,
    log_dir: &Path,
    log_entry: &LogEntry,
) -> ButtonResult<()> {
    // =================================================
    // Debug-Assert, Test-Assert, Production-Catch-Handle
    // =================================================

    debug_assert!(
        target_file.is_absolute(),
        "target_file must be absolute path"
    );

    #[cfg(test)]
    assert!(
        target_file.is_absolute(),
        "target_file must be absolute path"
    );

    if !target_file.is_absolute() {
        return Err(ButtonError::LogDirectoryError {
            path: target_file.to_path_buf(),
            reason: "Target file path must be absolute",
        });
    }

    debug_assert!(log_dir.is_absolute(), "log_dir must be absolute path");

    #[cfg(test)]
    assert!(log_dir.is_absolute(), "log_dir must be absolute path");

    if !log_dir.is_absolute() {
        return Err(ButtonError::LogDirectoryError {
            path: log_dir.to_path_buf(),
            reason: "Log directory path must be absolute",
        });
    }

    // Create log directory if it doesn't exist
    if !log_dir.exists() {
        fs::create_dir_all(log_dir).map_err(|e| ButtonError::Io(e))?;
    }

    // Get next log number
    let log_number = get_next_log_number(log_dir)?;

    // Build log file path: "{log_dir}/{number}"
    let log_file_path = log_dir.join(log_number.to_string());

    // Serialize log entry
    let log_content = log_entry.to_file_format();

    // Write to file
    fs::write(&log_file_path, log_content).map_err(|e| {
        // Log error before returning
        log_button_error(
            target_file,
            &format!("Failed to write log file: {}", e),
            Some("write_log_entry_to_file"),
        );
        ButtonError::Io(e)
    })?;

    #[cfg(debug_assertions)]
    println!(
        "Created log file: {} for {:?} at position {}",
        log_file_path.display(),
        log_entry.edit_type(),
        log_entry.position()
    );

    Ok(())
}

/// Creates changelog entry when user ADDS a byte
///
/// # Purpose
/// When user adds a byte to the file, this creates a log entry that says "remove"
/// so that undo will remove the added byte.
///
/// # Inverse Changelog Logic
/// - User action: ADD byte at position
/// - Log entry: RMV at position (undo removes the added byte)
///
/// # Arguments
/// * `target_file` - File being edited (absolute path)
/// * `edit_file_position` - Position where user added byte (0-indexed)
/// * `log_directory_path` - Directory to write log file (absolute path)
///
/// # Returns
/// * `ButtonResult<()>` - Success or error
///
/// # Examples
/// ```
/// // User added 'H' (0x48) at position 42 in file.txt
/// // Create log that says "remove at position 42"
/// button_remove_byte_make_log_file(
///     &Path::new("/absolute/path/to/file.txt"),
///     42,
///     &Path::new("/absolute/path/to/changelog_file")
/// )?;
/// ```
pub fn button_remove_byte_make_log_file(
    target_file: &Path,
    edit_file_position: u128,
    log_directory_path: &Path,
) -> ButtonResult<()> {
    // Create log entry: Rmv at position (no byte value needed)
    let log_entry = LogEntry::new(EditType::Rmv, edit_file_position, None)
        .map_err(|e| ButtonError::AssertionViolation { check: e })?;

    // Write to log directory
    write_log_entry_to_file(target_file, log_directory_path, &log_entry)?;

    Ok(())
}

/// Creates changelog entry when user REMOVES a byte
///
/// # Purpose
/// When user removes a byte from the file, this creates a log entry that says "add"
/// so that undo will add back the removed byte.
///
/// # Inverse Changelog Logic
/// - User action: REMOVE byte (value was 0x48) at position
/// - Log entry: ADD 0x48 at position (undo restores the removed byte)
///
/// # Arguments
/// * `target_file` - File being edited (absolute path)
/// * `edit_file_position` - Position where user removed byte (0-indexed)
/// * `byte_value` - The byte value that was removed
/// * `log_directory_path` - Directory to write log file (absolute path)
///
/// # Returns
/// * `ButtonResult<()>` - Success or error
///
/// # Examples
/// ```
/// // User removed 'H' (0x48) at position 42 from file.txt
/// // Create log that says "add 0x48 at position 42"
/// button_add_byte_make_log_file(
///     &Path::new("/absolute/path/to/file.txt"),
///     42,
///     0x48,
///     &Path::new("/absolute/path/to/changelog_file")
/// )?;
/// ```
pub fn button_add_byte_make_log_file(
    target_file: &Path,
    edit_file_position: u128,
    byte_value: u8,
    log_directory_path: &Path,
) -> ButtonResult<()> {
    // Create log entry: Add byte at position
    let log_entry = LogEntry::new(EditType::Add, edit_file_position, Some(byte_value))
        .map_err(|e| ButtonError::AssertionViolation { check: e })?;

    // Write to log directory
    write_log_entry_to_file(target_file, log_directory_path, &log_entry)?;

    Ok(())
}

/// Creates changelog entry when user HEX-EDITS a byte in place
///
/// # Purpose
/// When user changes a byte value without changing file length (hex edit),
/// this creates a log entry that says "edit back to original value"
/// so that undo will restore the original byte.
///
/// # Inverse Changelog Logic
/// - User action: EDIT byte at position (0xFF → 0x61)
/// - Log entry: EDT 0xFF at position (undo restores original 0xFF)
///
/// # Arguments
/// * `target_file` - File being edited (absolute path)
/// * `edit_file_position` - Position where user edited byte (0-indexed)
/// * `original_byte_value` - The ORIGINAL byte value before user's edit
/// * `log_directory_path` - Directory to write log file (absolute path)
///
/// # Returns
/// * `ButtonResult<()>` - Success or error
///
/// # Examples
/// ```
/// // User hex-edited position 42: changed 0xFF to 0x61
/// // Create log that says "edit back to 0xFF at position 42"
/// button_hexeditinplace_byte_make_log_file(
///     &Path::new("/absolute/path/to/file.txt"),
///     42,
///     0xFF,  // Original value before user's edit
///     &Path::new("/absolute/path/to/changelog_file")
/// )?;
/// ```
pub fn button_hexeditinplace_byte_make_log_file(
    target_file: &Path,
    edit_file_position: u128,
    original_byte_value: u8,
    log_directory_path: &Path,
) -> ButtonResult<()> {
    // Create log entry: Edit byte at position back to original value
    let log_entry = LogEntry::new(EditType::Edt, edit_file_position, Some(original_byte_value))
        .map_err(|e| ButtonError::AssertionViolation { check: e })?;

    // Write to log directory
    write_log_entry_to_file(target_file, log_directory_path, &log_entry)?;

    Ok(())
}

// ============================================================================
// UNIT TESTS FOR LOG FILE CREATION
// ============================================================================

#[cfg(test)]
mod log_creation_tests {
    use super::*;
    use std::env;

    #[test]
    fn test_get_next_log_number_empty_dir() {
        let test_dir = env::temp_dir().join("button_test_empty");
        let _ = fs::remove_dir_all(&test_dir); // Clean up if exists
        fs::create_dir_all(&test_dir).unwrap();

        let next_num = get_next_log_number(&test_dir).unwrap();
        assert_eq!(next_num, 0, "Empty directory should return 0");

        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_get_next_log_number_with_logs() {
        let test_dir = env::temp_dir().join("button_test_with_logs");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();

        // Create some log files
        fs::write(test_dir.join("0"), "test").unwrap();
        fs::write(test_dir.join("1"), "test").unwrap();
        fs::write(test_dir.join("2"), "test").unwrap();

        let next_num = get_next_log_number(&test_dir).unwrap();
        assert_eq!(next_num, 3, "Should return 3 after 0,1,2");

        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_get_next_log_number_with_multibyte_logs() {
        let test_dir = env::temp_dir().join("button_test_multibyte");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();

        // Create multibyte log files (10, 10.a, 10.b)
        fs::write(test_dir.join("10"), "test").unwrap();
        fs::write(test_dir.join("10.a"), "test").unwrap();
        fs::write(test_dir.join("10.b"), "test").unwrap();

        let next_num = get_next_log_number(&test_dir).unwrap();
        assert_eq!(next_num, 11, "Should return 11 after 10.x series");

        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_button_remove_byte_make_log_file() {
        let test_dir = env::temp_dir().join("button_test_remove");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();

        let target_file = test_dir.join("target.txt");
        fs::write(&target_file, b"test").unwrap();

        // User ADDED byte at position 42
        // Log should say: REMOVE at position 42
        let result = button_remove_byte_make_log_file(
            &target_file.canonicalize().unwrap(),
            42,
            &test_dir.canonicalize().unwrap(),
        );

        assert!(result.is_ok(), "Log creation should succeed");

        // Verify log file was created
        let log_file = test_dir.join("0");
        assert!(log_file.exists(), "Log file should exist");

        // Verify log content
        let content = fs::read_to_string(&log_file).unwrap();
        assert!(
            content.starts_with("rmv\n42\n"),
            "Log should contain rmv and position"
        );

        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_button_add_byte_make_log_file() {
        let test_dir = env::temp_dir().join("button_test_add");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();

        let target_file = test_dir.join("target.txt");
        fs::write(&target_file, b"test").unwrap();

        // User REMOVED byte 0x48 at position 100
        // Log should say: ADD 0x48 at position 100
        let result = button_add_byte_make_log_file(
            &target_file.canonicalize().unwrap(),
            100,
            0x48,
            &test_dir.canonicalize().unwrap(),
        );

        assert!(result.is_ok(), "Log creation should succeed");

        // Verify log file
        let log_file = test_dir.join("0");
        assert!(log_file.exists(), "Log file should exist");

        let content = fs::read_to_string(&log_file).unwrap();
        assert!(content.contains("add"), "Log should contain add");
        assert!(content.contains("100"), "Log should contain position");
        assert!(content.contains("48"), "Log should contain byte value");

        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_button_hexeditinplace_byte_make_log_file() {
        let test_dir = env::temp_dir().join("button_test_hexedit");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();

        let target_file = test_dir.join("target.txt");
        fs::write(&target_file, b"test").unwrap();

        // User HEX-EDITED position 200: 0xFF → 0x61
        // Log should say: EDT 0xFF at position 200
        let result = button_hexeditinplace_byte_make_log_file(
            &target_file.canonicalize().unwrap(),
            200,
            0xFF, // Original value
            &test_dir.canonicalize().unwrap(),
        );

        assert!(result.is_ok(), "Log creation should succeed");

        // Verify log file
        let log_file = test_dir.join("0");
        assert!(log_file.exists(), "Log file should exist");

        let content = fs::read_to_string(&log_file).unwrap();
        assert!(content.contains("edt"), "Log should contain edt");
        assert!(content.contains("200"), "Log should contain position");
        assert!(content.contains("FF"), "Log should contain original byte");

        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_sequential_log_numbering() {
        let test_dir = env::temp_dir().join("button_test_sequential");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();

        let target_file = test_dir.join("target.txt");
        fs::write(&target_file, b"test").unwrap();
        let target_abs = target_file.canonicalize().unwrap();
        let dir_abs = test_dir.canonicalize().unwrap();

        // Create three logs
        button_remove_byte_make_log_file(&target_abs, 10, &dir_abs).unwrap();
        button_add_byte_make_log_file(&target_abs, 20, 0xAA, &dir_abs).unwrap();
        button_hexeditinplace_byte_make_log_file(&target_abs, 30, 0xBB, &dir_abs).unwrap();

        // Verify files 0, 1, 2 exist
        assert!(test_dir.join("0").exists());
        assert!(test_dir.join("1").exists());
        assert!(test_dir.join("2").exists());

        let _ = fs::remove_dir_all(&test_dir);
    }
}

// ============================================================================
// LOG FILE OPERATIONS: Single Byte
// ============================================================================

// ============================================================================
// LOG FILE OPERATIONS - PHASE 2B: SINGLE-BYTE UNDO EXECUTION
// ============================================================================

/// Reads and parses a log file into a LogEntry
///
/// # Purpose
/// Reads a log file from disk and deserializes it into a LogEntry struct.
/// Validates the log file format and content.
///
/// # Arguments
/// * `log_file_path` - Path to the log file to read
///
/// # Returns
/// * `ButtonResult<LogEntry>` - Parsed log entry or error
///
/// # Errors
/// - File doesn't exist
/// - File cannot be read
/// - Log file is malformed (invalid format)
/// - Log file has invalid content (bad hex, invalid position, etc.)
///
/// # Examples
/// ```
/// let log_entry = read_log_file(&Path::new("/path/to/changelog/0"))?;
/// assert_eq!(log_entry.edit_type(), EditType::Add);
/// ```
fn read_log_file(log_file_path: &Path) -> ButtonResult<LogEntry> {
    // =================================================
    // Debug-Assert, Test-Assert, Production-Catch-Handle
    // =================================================

    debug_assert!(log_file_path.exists(), "Log file must exist before reading");

    #[cfg(test)]
    assert!(log_file_path.exists(), "Log file must exist before reading");

    if !log_file_path.exists() {
        return Err(ButtonError::MalformedLog {
            log_path: log_file_path.to_path_buf(),
            reason: "Log file does not exist",
        });
    }

    // Read file content
    let content = fs::read_to_string(log_file_path).map_err(|e| {
        #[cfg(debug_assertions)]
        eprintln!("Failed to read log file {}: {}", log_file_path.display(), e);

        ButtonError::MalformedLog {
            log_path: log_file_path.to_path_buf(),
            reason: "Cannot read log file",
        }
    })?;

    // Parse into LogEntry
    let log_entry = LogEntry::from_file_format(&content).map_err(|reason| {
        #[cfg(debug_assertions)]
        eprintln!(
            "Failed to parse log file {}: {}",
            log_file_path.display(),
            reason
        );

        ButtonError::MalformedLog {
            log_path: log_file_path.to_path_buf(),
            reason,
        }
    })?;

    Ok(log_entry)
}

/// Executes a single log entry by calling the appropriate file operation
///
/// # Purpose
/// Takes a parsed LogEntry and executes the undo operation on the target file
/// by dispatching to the correct function from basic_file_byte_operations.
///
/// # Dispatch Logic
/// - `EditType::Add` → calls `add_single_byte_to_file()` (restore removed byte)
/// - `EditType::Rmv` → calls `remove_single_byte_from_file()` (remove added byte)
/// - `EditType::Edt` → calls `replace_single_byte_in_file()` (restore original byte)
///
/// # Arguments
/// * `target_file` - File to perform undo operation on (absolute path)
/// * `log_entry` - The log entry describing what to undo
///
/// # Returns
/// * `ButtonResult<()>` - Success or error
///
/// # Note on basic_file_byte_operations Integration
/// This function assumes the following functions are available:
/// - `add_single_byte_to_file(path, position, byte) -> io::Result<()>`
/// - `remove_single_byte_from_file(path, position) -> io::Result<()>`
/// - `replace_single_byte_in_file(path, position, byte) -> io::Result<()>`
///
/// These functions come from the basic_file_byte_operations module.
fn execute_log_entry(target_file: &Path, log_entry: &LogEntry) -> ButtonResult<()> {
    // =================================================
    // Debug-Assert, Test-Assert, Production-Catch-Handle
    // =================================================

    debug_assert!(
        target_file.is_absolute(),
        "Target file must be absolute path"
    );

    #[cfg(test)]
    assert!(
        target_file.is_absolute(),
        "Target file must be absolute path"
    );

    if !target_file.is_absolute() {
        return Err(ButtonError::AssertionViolation {
            check: "Target file path must be absolute",
        });
    }

    debug_assert!(
        target_file.exists(),
        "Target file must exist before undo operation"
    );

    #[cfg(test)]
    assert!(
        target_file.exists(),
        "Target file must exist before undo operation"
    );

    if !target_file.exists() {
        return Err(ButtonError::Io(io::Error::new(
            io::ErrorKind::NotFound,
            "Target file does not exist",
        )));
    }

    // Get file size for bounds checking
    let file_metadata = fs::metadata(target_file).map_err(|e| ButtonError::Io(e))?;
    let file_size = file_metadata.len() as u128;

    let position = log_entry.position();

    // Dispatch based on edit type
    match log_entry.edit_type() {
        EditType::Add => {
            // Log says "add" - user had removed, so restore the byte
            let byte_value = log_entry
                .byte_value()
                .ok_or_else(|| ButtonError::MalformedLog {
                    log_path: PathBuf::from("unknown"),
                    reason: "Add operation missing byte value",
                })?;

            #[cfg(debug_assertions)]
            println!(
                "Undo: Adding byte 0x{:02X} at position {} (user had removed)",
                byte_value, position
            );

            // Validate position for add (can be at EOF)
            if position > file_size {
                return Err(ButtonError::PositionOutOfBounds {
                    position,
                    file_size,
                });
            }

            // Call basic_file_byte_operations::add_single_byte_to_file
            add_single_byte_to_file(target_file.to_path_buf(), position as usize, byte_value)
                .map_err(|e| ButtonError::Io(e))?;
        }

        EditType::Rmv => {
            // Log says "rmv" - user had added, so remove the byte
            #[cfg(debug_assertions)]
            println!(
                "Undo: Removing byte at position {} (user had added)",
                position
            );

            // Validate position for remove (must be within file)
            if position >= file_size {
                return Err(ButtonError::PositionOutOfBounds {
                    position,
                    file_size,
                });
            }

            // Call basic_file_byte_operations::remove_single_byte_from_file
            remove_single_byte_from_file(target_file.to_path_buf(), position as usize)
                .map_err(|e| ButtonError::Io(e))?;
        }

        EditType::Edt => {
            // Log says "edt" - user had hex-edited, so restore original byte
            let byte_value = log_entry
                .byte_value()
                .ok_or_else(|| ButtonError::MalformedLog {
                    log_path: PathBuf::from("unknown"),
                    reason: "Edit operation missing byte value",
                })?;

            #[cfg(debug_assertions)]
            println!(
                "Undo: Replacing byte at position {} with 0x{:02X} (user had hex-edited)",
                position, byte_value
            );

            // Validate position for edit (must be within file)
            if position >= file_size {
                return Err(ButtonError::PositionOutOfBounds {
                    position,
                    file_size,
                });
            }

            // Call basic_file_byte_operations::replace_single_byte_in_file
            replace_single_byte_in_file(target_file.to_path_buf(), position as usize, byte_value)
                .map_err(|e| ButtonError::Io(e))?;
        }
    }

    Ok(())
}

/// Finds the next log file to undo in LIFO order
///
/// # Purpose
/// Scans the log directory to find the highest-numbered log file,
/// which is the most recent change (Last In, First Out).
///
/// # Arguments
/// * `log_dir` - Directory containing changelog files
///
/// # Returns
/// * `ButtonResult<PathBuf>` - Path to the next log file to undo
///
/// # LIFO Logic
/// - Looks for highest number: if directory has 0,1,2,3 → returns 3
/// - Ignores letter suffixes for now (handles single-byte only)
/// - Returns error if directory is empty (no logs to undo)
///
/// # Examples
/// ```
/// // Directory contains: 0, 1, 2, 3
/// let next_log = find_next_lifo_log_file(&log_dir)?;
/// assert_eq!(next_log.file_name().unwrap(), "3");
/// ```
fn find_next_lifo_log_file(log_dir: &Path) -> ButtonResult<PathBuf> {
    // =================================================
    // Debug-Assert, Test-Assert, Production-Catch-Handle
    // =================================================

    debug_assert!(log_dir.exists(), "Log directory must exist");

    #[cfg(test)]
    assert!(log_dir.exists(), "Log directory must exist");

    if !log_dir.exists() {
        return Err(ButtonError::NoLogsFound {
            log_dir: log_dir.to_path_buf(),
        });
    }

    debug_assert!(log_dir.is_dir(), "Log path must be a directory");

    #[cfg(test)]
    assert!(log_dir.is_dir(), "Log path must be a directory");

    if !log_dir.is_dir() {
        return Err(ButtonError::LogDirectoryError {
            path: log_dir.to_path_buf(),
            reason: "Path exists but is not a directory",
        });
    }

    let mut max_number: Option<u128> = None;
    let mut max_log_path: Option<PathBuf> = None;

    // Read directory entries
    let entries = fs::read_dir(log_dir).map_err(|e| ButtonError::Io(e))?;

    // Bounded loop: iterate through directory entries
    const MAX_DIR_ENTRIES: usize = 10_000_000;
    let mut entry_count: usize = 0;

    for entry_result in entries {
        // =================================================
        // Debug-Assert, Test-Assert, Production-Catch-Handle
        // =================================================

        debug_assert!(
            entry_count < MAX_DIR_ENTRIES,
            "Directory entry count exceeded safety limit"
        );

        #[cfg(test)]
        assert!(
            entry_count < MAX_DIR_ENTRIES,
            "Directory entry count exceeded safety limit"
        );

        if entry_count >= MAX_DIR_ENTRIES {
            return Err(ButtonError::LogDirectoryError {
                path: log_dir.to_path_buf(),
                reason: "Too many directory entries (safety limit)",
            });
        }

        entry_count += 1;

        let entry = entry_result.map_err(|e| ButtonError::Io(e))?;
        let entry_path = entry.path();

        // Skip if not a file
        if !entry_path.is_file() {
            continue;
        }

        let filename = entry.file_name();
        let filename_str = filename.to_string_lossy();

        // For single-byte logs: Parse filename as bare number (ignore .letter for now)
        // Extract the numeric part before any '.'
        let numeric_part = if let Some(dot_pos) = filename_str.find('.') {
            &filename_str[..dot_pos]
        } else {
            &filename_str[..]
        };

        // Try to parse as u128
        if let Ok(number) = numeric_part.parse::<u128>() {
            // For LIFO: we want the highest number WITHOUT a letter suffix
            // (single-byte logs have no letter)
            let has_letter_suffix = filename_str.contains('.');

            if !has_letter_suffix {
                // This is a bare number (single-byte log or last in multi-byte set)
                match max_number {
                    None => {
                        max_number = Some(number);
                        max_log_path = Some(entry_path);
                    }
                    Some(current_max) => {
                        if number > current_max {
                            max_number = Some(number);
                            max_log_path = Some(entry_path);
                        }
                    }
                }
            }
        }
    }

    // Return the path with highest number
    match max_log_path {
        Some(path) => Ok(path),
        None => Err(ButtonError::NoLogsFound {
            log_dir: log_dir.to_path_buf(),
        }),
    }
}

// ============================================================================
// UNIT TESTS FOR UNDO OPERATIONS
// ============================================================================

#[cfg(test)]
mod undo_tests {
    use super::*;
    use std::env;

    #[test]
    fn test_read_log_file_valid() {
        let test_dir = env::temp_dir().join("button_test_read_log");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();

        // Create a valid log file
        let log_file = test_dir.join("0");
        fs::write(&log_file, "add\n42\n48\n").unwrap();

        let log_entry = read_log_file(&log_file).unwrap();
        assert_eq!(log_entry.edit_type(), EditType::Add);
        assert_eq!(log_entry.position(), 42);
        assert_eq!(log_entry.byte_value(), Some(0x48));

        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_read_log_file_malformed() {
        let test_dir = env::temp_dir().join("button_test_read_bad_log");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();

        // Create a malformed log file (missing position)
        let log_file = test_dir.join("0");
        fs::write(&log_file, "add\n").unwrap();

        let result = read_log_file(&log_file);
        assert!(result.is_err(), "Should fail on malformed log");

        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_find_next_lifo_log_file() {
        let test_dir = env::temp_dir().join("button_test_find_lifo");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();

        // Create log files 0, 1, 2, 3
        fs::write(test_dir.join("0"), "test").unwrap();
        fs::write(test_dir.join("1"), "test").unwrap();
        fs::write(test_dir.join("2"), "test").unwrap();
        fs::write(test_dir.join("3"), "test").unwrap();

        let next_log = find_next_lifo_log_file(&test_dir).unwrap();
        assert_eq!(
            next_log.file_name().unwrap().to_string_lossy(),
            "3",
            "Should find highest numbered log"
        );

        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_find_next_lifo_empty_dir() {
        let test_dir = env::temp_dir().join("button_test_find_lifo_empty");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();

        let result = find_next_lifo_log_file(&test_dir);
        assert!(result.is_err(), "Should fail on empty directory");

        match result {
            Err(ButtonError::NoLogsFound { .. }) => {} // Expected
            _ => panic!("Should return NoLogsFound error"),
        }

        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_full_undo_cycle_add() {
        // Test full cycle: user removes byte -> log created -> undo restores byte
        let test_dir = env::temp_dir().join("button_test_undo_add");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();

        // Create target file with content
        let target_file = test_dir.join("target.txt");
        fs::write(&target_file, b"ABCD").unwrap();
        let target_abs = target_file.canonicalize().unwrap();

        // Create log directory
        let log_dir = test_dir.join("logs");
        fs::create_dir_all(&log_dir).unwrap();
        let log_dir_abs = log_dir.canonicalize().unwrap();

        // Simulate: User removed byte 'X' (0x58) at position 2
        // Log should say: ADD 0x58 at position 2
        button_add_byte_make_log_file(&target_abs, 2, 0x58, &log_dir_abs).unwrap();

        // Manually remove byte to simulate user action
        // File was "ABCD", user removes at position 2, file becomes "ABCD" -> we'll manually edit
        // Actually, let's simulate by starting with correct state
        fs::write(&target_file, b"ABCD").unwrap(); // Position 2 needs 'X' added

        // Perform undo (should add 'X' at position 2)
        button_undo_single_byte_with_redo_support(&target_abs, &log_dir_abs, false, None).unwrap();

        // Verify: Byte was added at position 2
        let content = fs::read(&target_file).unwrap();
        assert_eq!(content[2], 0x58, "Byte should be restored at position 2");
        assert_eq!(content.len(), 5, "File should be 5 bytes");

        // Verify: Log file was removed
        assert!(
            !log_dir.join("0").exists(),
            "Log file should be removed after undo"
        );

        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_full_undo_cycle_remove() {
        // Test full cycle: user adds byte -> log created -> undo removes byte
        let test_dir = env::temp_dir().join("button_test_undo_remove");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();

        let target_file = test_dir.join("target.txt");
        fs::write(&target_file, b"ABXCD").unwrap(); // File with extra 'X' that user added
        let target_abs = target_file.canonicalize().unwrap();

        let log_dir = test_dir.join("logs");
        fs::create_dir_all(&log_dir).unwrap();
        let log_dir_abs = log_dir.canonicalize().unwrap();

        // Simulate: User added byte 'X' at position 2
        // Log should say: RMV at position 2
        button_remove_byte_make_log_file(&target_abs, 2, &log_dir_abs).unwrap();

        // Perform undo (should remove byte at position 2)
        button_undo_single_byte_with_redo_support(&target_abs, &log_dir_abs, false, None).unwrap();

        // Verify: Byte was removed from position 2
        let content = fs::read(&target_file).unwrap();
        assert_eq!(
            content, b"ABCD",
            "Byte should be removed, restoring original"
        );

        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_full_undo_cycle_edit() {
        // Test full cycle: user edits byte -> log created -> undo restores original
        let test_dir = env::temp_dir().join("button_test_undo_edit");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();

        let target_file = test_dir.join("target.txt");
        fs::write(&target_file, b"ABZD").unwrap(); // User changed 'C' (0x43) to 'Z' (0x5A)
        let target_abs = target_file.canonicalize().unwrap();

        let log_dir = test_dir.join("logs");
        fs::create_dir_all(&log_dir).unwrap();
        let log_dir_abs = log_dir.canonicalize().unwrap();

        // Simulate: User hex-edited position 2: 'C' (0x43) -> 'Z' (0x5A)
        // Log should say: EDT 0x43 at position 2 (restore original 'C')
        button_hexeditinplace_byte_make_log_file(&target_abs, 2, 0x43, &log_dir_abs).unwrap();

        // Perform undo (should restore 'C' at position 2)
        button_undo_single_byte_with_redo_support(&target_abs, &log_dir_abs, false, None).unwrap();

        // Verify: Original byte was restored
        let content = fs::read(&target_file).unwrap();
        assert_eq!(content, b"ABCD", "Original byte should be restored");

        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_multiple_undo_lifo_order() {
        // Test that multiple undos happen in LIFO order
        let test_dir = env::temp_dir().join("button_test_multiple_undo");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();

        let target_file = test_dir.join("target.txt");
        fs::write(&target_file, b"ABXYZCD").unwrap(); // User added X, Y, Z in sequence
        let target_abs = target_file.canonicalize().unwrap();

        let log_dir = test_dir.join("logs");
        fs::create_dir_all(&log_dir).unwrap();
        let log_dir_abs = log_dir.canonicalize().unwrap();

        // User added X at position 2, then Y at position 3, then Z at position 4
        // Logs say: remove at 2, remove at 3, remove at 4
        button_remove_byte_make_log_file(&target_abs, 2, &log_dir_abs).unwrap(); // Log 0
        button_remove_byte_make_log_file(&target_abs, 3, &log_dir_abs).unwrap(); // Log 1
        button_remove_byte_make_log_file(&target_abs, 4, &log_dir_abs).unwrap(); // Log 2

        // Undo first (should undo log 2: remove at position 4, removing 'Z')
        button_undo_single_byte_with_redo_support(&target_abs, &log_dir_abs, false, None).unwrap();
        let content = fs::read(&target_file).unwrap();
        assert_eq!(content, b"ABXYCD", "First undo should remove Z");

        // Undo second (should undo log 1: remove at position 3, removing 'Y')
        button_undo_single_byte_with_redo_support(&target_abs, &log_dir_abs, false, None).unwrap();
        let content = fs::read(&target_file).unwrap();
        assert_eq!(content, b"ABXCD", "Second undo should remove Y");

        // Undo third (should undo log 0: remove at position 2, removing 'X')
        button_undo_single_byte_with_redo_support(&target_abs, &log_dir_abs, false, None).unwrap();
        let content = fs::read(&target_file).unwrap();
        assert_eq!(content, b"ABCD", "Third undo should remove X");

        // Verify all logs consumed
        let result = find_next_lifo_log_file(&log_dir_abs);
        assert!(result.is_err(), "Should have no logs remaining");

        let _ = fs::remove_dir_all(&test_dir);
    }
}

// ============================================================================
// MULTI-BYTE UTF-8 OPERATIONS
// ============================================================================

// ============================================================================
// MULTI-BYTE UTF-8 OPERATIONS - PHASE 3: CHARACTER DETECTION & LOG CREATION
// ============================================================================

/// Detects the number of bytes in a UTF-8 character by examining the first byte
///
/// # Purpose
/// UTF-8 encoding uses the leading byte to indicate how many bytes follow:
/// - 0xxxxxxx: 1-byte character (ASCII)
/// - 110xxxxx: 2-byte character
/// - 1110xxxx: 3-byte character
/// - 11110xxx: 4-byte character
///
/// # Arguments
/// * `first_byte` - The first byte of a potential UTF-8 character
///
/// # Returns
/// * `Result<usize, &'static str>` - Number of bytes (1-4) or error
///
/// # UTF-8 Encoding Rules
/// ```text
/// 1-byte: 0xxxxxxx                (0x00-0x7F)
/// 2-byte: 110xxxxx 10xxxxxx       (0xC0-0xDF)
/// 3-byte: 1110xxxx 10xxxxxx 10xxxxxx (0xE0-0xEF)
/// 4-byte: 11110xxx 10xxxxxx 10xxxxxx 10xxxxxx (0xF0-0xF7)
/// ```
///
/// # Examples
/// ```
/// assert_eq!(detect_utf8_byte_count(0x41), Ok(1)); // 'A' - ASCII
/// assert_eq!(detect_utf8_byte_count(0xC3), Ok(2)); // Start of 2-byte char
/// assert_eq!(detect_utf8_byte_count(0xE9), Ok(3)); // Start of 3-byte char
/// assert_eq!(detect_utf8_byte_count(0xF0), Ok(4)); // Start of 4-byte char
/// ```
fn detect_utf8_byte_count(first_byte: u8) -> Result<usize, &'static str> {
    // Check bit patterns using bit masking
    if first_byte & 0b1000_0000 == 0 {
        // Pattern: 0xxxxxxx - ASCII (1 byte)
        Ok(1)
    } else if first_byte & 0b1110_0000 == 0b1100_0000 {
        // Pattern: 110xxxxx - 2-byte sequence
        Ok(2)
    } else if first_byte & 0b1111_0000 == 0b1110_0000 {
        // Pattern: 1110xxxx - 3-byte sequence
        Ok(3)
    } else if first_byte & 0b1111_1000 == 0b1111_0000 {
        // Pattern: 11110xxx - 4-byte sequence
        Ok(4)
    } else {
        // Invalid UTF-8 start byte
        Err("Invalid UTF-8 start byte")
    }
}

/// Reads a character's bytes from a file at a specific position
///
/// # Purpose
/// Reads the bytes that make up a complete UTF-8 character from a file.
/// Validates that the sequence forms a valid UTF-8 character.
///
/// # Arguments
/// * `file_path` - File to read from (absolute path)
/// * `position` - Starting position of the character (0-indexed)
///
/// # Returns
/// * `ButtonResult<Vec<u8>>` - The character's bytes (1-4 bytes)
///
/// # Behavior
/// - Reads first byte to detect character length
/// - Reads remaining bytes
/// - Validates the complete sequence as valid UTF-8
/// - Returns error if not a valid character
///
/// # Examples
/// ```
/// // Read character at position 10 (might be 'A' or '阿' or '𝕏')
/// let char_bytes = read_character_bytes_from_file(&file_path, 10)?;
/// assert!(char_bytes.len() >= 1 && char_bytes.len() <= 4);
/// ```
fn read_character_bytes_from_file(file_path: &Path, position: u128) -> ButtonResult<Vec<u8>> {
    // =================================================
    // Debug-Assert, Test-Assert, Production-Catch-Handle
    // =================================================

    debug_assert!(
        file_path.exists(),
        "File must exist before reading character"
    );

    #[cfg(test)]
    assert!(
        file_path.exists(),
        "File must exist before reading character"
    );

    if !file_path.exists() {
        return Err(ButtonError::Io(io::Error::new(
            io::ErrorKind::NotFound,
            "File does not exist",
        )));
    }

    // Open file for reading
    let mut file = File::open(file_path).map_err(|e| ButtonError::Io(e))?;

    // Get file size
    let file_metadata = file.metadata().map_err(|e| ButtonError::Io(e))?;
    let file_size = file_metadata.len() as u128;

    // Validate position
    if position >= file_size {
        return Err(ButtonError::PositionOutOfBounds {
            position,
            file_size,
        });
    }

    // Seek to position
    file.seek(SeekFrom::Start(position as u64))
        .map_err(|e| ButtonError::Io(e))?;

    // Read first byte
    let mut first_byte_buffer = [0u8; 1];
    file.read_exact(&mut first_byte_buffer)
        .map_err(|e| ButtonError::Io(e))?;
    let first_byte = first_byte_buffer[0];

    // Detect character byte count
    let byte_count = detect_utf8_byte_count(first_byte).map_err(|e| ButtonError::InvalidUtf8 {
        position,
        byte_count: 0,
        reason: e,
    })?;

    // =================================================
    // Debug-Assert, Test-Assert, Production-Catch-Handle
    // =================================================

    debug_assert!(
        byte_count >= 1 && byte_count <= MAX_UTF8_BYTES,
        "Byte count must be 1-4"
    );

    #[cfg(test)]
    assert!(
        byte_count >= 1 && byte_count <= MAX_UTF8_BYTES,
        "Byte count must be 1-4"
    );

    if byte_count < 1 || byte_count > MAX_UTF8_BYTES {
        return Err(ButtonError::InvalidUtf8 {
            position,
            byte_count,
            reason: "Byte count out of valid range (1-4)",
        });
    }

    // Check if enough bytes remain in file
    if position + (byte_count as u128) > file_size {
        return Err(ButtonError::InvalidUtf8 {
            position,
            byte_count,
            reason: "Incomplete UTF-8 sequence (file too short)",
        });
    }

    // Allocate buffer for full character
    let mut char_bytes = vec![0u8; byte_count];
    char_bytes[0] = first_byte;

    // Read remaining bytes (if multi-byte character)
    if byte_count > 1 {
        file.read_exact(&mut char_bytes[1..byte_count])
            .map_err(|e| ButtonError::Io(e))?;
    }

    // Validate as UTF-8
    match std::str::from_utf8(&char_bytes) {
        Ok(_) => Ok(char_bytes),
        Err(_) => Err(ButtonError::InvalidUtf8 {
            position,
            byte_count,
            reason: "Invalid UTF-8 sequence",
        }),
    }
}

/// Creates multiple log files for a multi-byte character removal (user ADDED)
///
/// # Purpose
/// When user adds a multi-byte character, create multiple log files that say "remove"
/// to undo the addition. Uses the "cheap trick" button-stack approach where all
/// removes happen at the same position (the first byte position).
///
/// # Inverse Changelog Logic
/// - User action: ADD multi-byte character (e.g., '阿' = E9 98 BF) at position 20
/// - Log entries: RMV at position 20 (three times)
/// - Log files created:
///   * "10.b": rmv at 20 (last byte, highest letter, first in stack)
///   * "10.a": rmv at 20 (middle byte)
///   * "10": rmv at 20 (first byte, no letter, last in stack, first out)
///
/// # "Cheap Trick" Button Stack
/// All removals use the SAME position (position of first byte).
/// When undoing, each remove operation naturally shifts remaining bytes.
///
/// # Arguments
/// * `target_file` - File being edited (absolute path)
/// * `edit_file_position` - Position where user added character (0-indexed)
/// * `character_byte_count` - Number of bytes in the character (1-4)
/// * `log_directory_path` - Directory to write log files (absolute path)
///
/// # Returns
/// * `ButtonResult<()>` - Success or error
///
/// # Examples
/// ```
/// // User added '阿' (3 bytes: E9 98 BF) at position 20
/// // Create logs: 10.b, 10.a, 10 (all say "rmv at 20")
/// button_remove_multibyte_make_log_files(
///     &Path::new("/absolute/path/to/file.txt"),
///     20,
///     3,
///     &Path::new("/absolute/path/to/changelog_file")
/// )?;
/// ```
pub fn button_remove_multibyte_make_log_files(
    target_file: &Path,
    edit_file_position: u128,
    character_byte_count: usize,
    log_directory_path: &Path,
) -> ButtonResult<()> {
    // =================================================
    // Debug-Assert, Test-Assert, Production-Catch-Handle
    // =================================================

    debug_assert!(
        character_byte_count >= 1 && character_byte_count <= MAX_UTF8_BYTES,
        "Character byte count must be 1-4"
    );

    #[cfg(test)]
    assert!(
        character_byte_count >= 1 && character_byte_count <= MAX_UTF8_BYTES,
        "Character byte count must be 1-4"
    );

    if character_byte_count < 1 || character_byte_count > MAX_UTF8_BYTES {
        return Err(ButtonError::InvalidUtf8 {
            position: edit_file_position,
            byte_count: character_byte_count,
            reason: "Character byte count must be 1-4",
        });
    }

    // Create log directory if needed
    if !log_directory_path.exists() {
        fs::create_dir_all(log_directory_path).map_err(|e| ButtonError::Io(e))?;
    }

    // Get base log number for this character
    let base_log_number = get_next_log_number(log_directory_path)?;

    #[cfg(debug_assertions)]
    println!(
        "Creating {} remove log files starting at number {}",
        character_byte_count, base_log_number
    );

    // Create log files for each byte
    // Bounded loop: max 4 iterations (MAX_UTF8_BYTES)
    for byte_index in 0..character_byte_count {
        // =================================================
        // Debug-Assert, Test-Assert, Production-Catch-Handle
        // =================================================

        debug_assert!(
            byte_index < MAX_UTF8_BYTES,
            "Byte index exceeded max UTF-8 bytes"
        );

        #[cfg(test)]
        assert!(
            byte_index < MAX_UTF8_BYTES,
            "Byte index exceeded max UTF-8 bytes"
        );

        if byte_index >= MAX_UTF8_BYTES {
            return Err(ButtonError::AssertionViolation {
                check: "Byte index exceeded maximum",
            });
        }

        // Create log entry: Rmv at position (no byte value for remove)
        let log_entry = LogEntry::new(EditType::Rmv, edit_file_position, None)
            .map_err(|e| ButtonError::AssertionViolation { check: e })?;

        // Get letter suffix for this byte (or None for last byte)
        let letter_suffix = get_log_file_letter_suffix(byte_index, character_byte_count);

        // Build filename: "{number}" or "{number}.{letter}"
        let filename = match letter_suffix {
            Some(letter) => format!("{}.{}", base_log_number, letter),
            None => base_log_number.to_string(),
        };

        let log_file_path = log_directory_path.join(&filename);

        // Serialize and write
        let log_content = log_entry.to_file_format();
        fs::write(&log_file_path, log_content).map_err(|e| {
            log_button_error(
                target_file,
                &format!("Failed to write multi-byte log file {}: {}", filename, e),
                Some("button_remove_multibyte_make_log_files"),
            );
            ButtonError::Io(e)
        })?;

        #[cfg(debug_assertions)]
        println!("  Created log file: {}", filename);
    }

    Ok(())
}

/// Creates multiple log files for a multi-byte character addition (user REMOVED)
///
/// # Purpose
/// When user removes a multi-byte character, create multiple log files that say "add"
/// with the original bytes to restore the character. Uses button-stack approach where
/// all adds happen at the same position.
///
/// # Inverse Changelog Logic
/// - User action: REMOVE multi-byte character (e.g., '阿' = E9 98 BF) at position 20
/// - Log entries: ADD with each byte at position 20
/// - Log files created:
///   * "10.b": add BF at 20 (last byte, first in stack)
///   * "10.a": add 98 at 20 (middle byte)
///   * "10": add E9 at 20 (first byte, last in stack, first out)
///
/// # "Cheap Trick" Button Stack
/// All additions use the SAME position. When undoing (reading 10.b, 10.a, 10):
/// - First add BF at 20
/// - Then add 98 at 20 (pushes BF to position 21)
/// - Then add E9 at 20 (pushes 98 to 21, BF to 22)
/// - Result: E9 98 BF at positions 20-21-22 ✓
///
/// # Arguments
/// * `target_file` - File being edited (absolute path)
/// * `edit_file_position` - Position where user removed character (0-indexed)
/// * `character_bytes` - The bytes of the removed character (1-4 bytes)
/// * `log_directory_path` - Directory to write log files (absolute path)
///
/// # Returns
/// * `ButtonResult<()>` - Success or error
///
/// # Examples
/// ```
/// // User removed '阿' (E9 98 BF) at position 20
/// // Create logs: 10.b (add BF), 10.a (add 98), 10 (add E9)
/// button_add_multibyte_make_log_files(
///     &Path::new("/absolute/path/to/file.txt"),
///     20,
///     &[0xE9, 0x98, 0xBF],
///     &Path::new("/absolute/path/to/changelog_file")
/// )?;
/// ```
pub fn button_add_multibyte_make_log_files(
    target_file: &Path,
    edit_file_position: u128,
    character_bytes: &[u8],
    log_directory_path: &Path,
) -> ButtonResult<()> {
    let character_byte_count = character_bytes.len();

    // =================================================
    // Debug-Assert, Test-Assert, Production-Catch-Handle
    // =================================================

    debug_assert!(
        character_byte_count >= 1 && character_byte_count <= MAX_UTF8_BYTES,
        "Character byte count must be 1-4"
    );

    #[cfg(test)]
    assert!(
        character_byte_count >= 1 && character_byte_count <= MAX_UTF8_BYTES,
        "Character byte count must be 1-4"
    );

    if character_byte_count < 1 || character_byte_count > MAX_UTF8_BYTES {
        return Err(ButtonError::InvalidUtf8 {
            position: edit_file_position,
            byte_count: character_byte_count,
            reason: "Character byte count must be 1-4",
        });
    }

    // Validate UTF-8
    if std::str::from_utf8(character_bytes).is_err() {
        return Err(ButtonError::InvalidUtf8 {
            position: edit_file_position,
            byte_count: character_byte_count,
            reason: "Invalid UTF-8 byte sequence",
        });
    }

    // Create log directory if needed
    if !log_directory_path.exists() {
        fs::create_dir_all(log_directory_path).map_err(|e| ButtonError::Io(e))?;
    }

    // Get base log number
    let base_log_number = get_next_log_number(log_directory_path)?;

    #[cfg(debug_assertions)]
    println!(
        "Creating {} add log files starting at number {}",
        character_byte_count, base_log_number
    );

    // Create log files for each byte
    // Bounded loop: max 4 iterations
    for byte_index in 0..character_byte_count {
        // =================================================
        // Debug-Assert, Test-Assert, Production-Catch-Handle
        // =================================================

        debug_assert!(
            byte_index < MAX_UTF8_BYTES,
            "Byte index exceeded max UTF-8 bytes"
        );

        #[cfg(test)]
        assert!(
            byte_index < MAX_UTF8_BYTES,
            "Byte index exceeded max UTF-8 bytes"
        );

        if byte_index >= MAX_UTF8_BYTES {
            return Err(ButtonError::AssertionViolation {
                check: "Byte index exceeded maximum",
            });
        }

        let byte_value = character_bytes[byte_index];

        // Create log entry: Add byte at position
        let log_entry = LogEntry::new(EditType::Add, edit_file_position, Some(byte_value))
            .map_err(|e| ButtonError::AssertionViolation { check: e })?;

        // Get letter suffix
        let letter_suffix = get_log_file_letter_suffix(byte_index, character_byte_count);

        // Build filename
        let filename = match letter_suffix {
            Some(letter) => format!("{}.{}", base_log_number, letter),
            None => base_log_number.to_string(),
        };

        let log_file_path = log_directory_path.join(&filename);

        // Serialize and write
        let log_content = log_entry.to_file_format();
        fs::write(&log_file_path, log_content).map_err(|e| {
            log_button_error(
                target_file,
                &format!("Failed to write multi-byte log file {}: {}", filename, e),
                Some("button_add_multibyte_make_log_files"),
            );
            ButtonError::Io(e)
        })?;

        #[cfg(debug_assertions)]
        println!(
            "  Created log file: {} (byte 0x{:02X})",
            filename, byte_value
        );
    }

    Ok(())
}

// ============================================================================
// MULTI-BYTE UTF-8 OPERATIONS - PHASE 3B: UNDO EXECUTION
// ============================================================================

/// Finds all log files in a multi-byte log set
///
/// # Purpose
/// For a given base number, finds all associated log files including letter suffixes.
/// Returns them in LIFO order (highest letter first, bare number last).
///
/// # Arguments
/// * `log_dir` - Directory containing log files
/// * `base_number` - The base number for the log set
///
/// # Returns
/// * `ButtonResult<Vec<PathBuf>>` - Paths in LIFO order, or error if incomplete
///
/// # Expected Patterns
/// - 1-byte: just "10"
/// - 2-byte: "10.a", "10"
/// - 3-byte: "10.b", "10.a", "10"
/// - 4-byte: "10.c", "10.b", "10.a", "10"
///
/// # LIFO Order
/// Returns highest letter first: [10.c, 10.b, 10.a, 10]
///
/// # Validation
/// - Must have bare number file (no letter)
/// - Letters must be sequential from 'a' with no gaps
/// - No orphaned letters (e.g., having 'b' without 'a')
/// - Returns error if incomplete set detected
fn find_multibyte_log_set(log_dir: &Path, base_number: u128) -> ButtonResult<Vec<PathBuf>> {
    let mut log_files = Vec::with_capacity(MAX_UTF8_BYTES);

    // Check for bare number (required)
    let bare_path = log_dir.join(base_number.to_string());
    if !bare_path.exists() {
        return Err(ButtonError::IncompleteLogSet {
            base_number,
            found_logs: "missing base file",
        });
    }

    // FIXED: Scan ALL possible letter files first (don't break early)
    let mut found_letters = Vec::new();
    for i in 0..(MAX_UTF8_BYTES - 1) {
        let letter = LOG_LETTER_SEQUENCE[i];
        let letter_path = log_dir.join(format!("{}.{}", base_number, letter));

        if letter_path.exists() {
            found_letters.push((i, letter, letter_path));
        }
    }

    // If no letters found, this is a single-byte log (valid)
    if found_letters.is_empty() {
        log_files.push(bare_path);
        return Ok(log_files);
    }

    // FIXED: Validate that letters are sequential with NO GAPS
    // Check that we have indices 0, 1, 2... with no missing values
    for expected_index in 0..found_letters.len() {
        let (actual_index, letter, _) = &found_letters[expected_index];

        if *actual_index != expected_index {
            // We have a gap! For example: found 'b' (index 1) but missing 'a' (index 0)
            #[cfg(debug_assertions)]
            eprintln!(
                "Incomplete log set {}: found letter '{}' but missing earlier letters",
                base_number, letter
            );

            return Err(ButtonError::IncompleteLogSet {
                base_number,
                found_logs: "non-sequential letters (gap detected)",
            });
        }
    }

    // Build result in LIFO order: highest letter first, bare number last
    // Reverse the found letters
    for (_index, _letter, path) in found_letters.iter().rev() {
        log_files.push(path.clone());
    }

    // Add bare number last (comes out first in LIFO)
    log_files.push(bare_path);

    Ok(log_files)
}

/// Finds the next multi-byte log set to undo in LIFO order
///
/// # Purpose
/// Finds the highest-numbered bare log file (no letter suffix) and returns
/// the complete set of log files for that multi-byte character.
///
/// # Arguments
/// * `log_dir` - Directory containing log files
///
/// # Returns
/// * `ButtonResult<Vec<PathBuf>>` - Log files in LIFO order
///
/// # Behavior
/// - Scans for highest bare number (no '.letter' suffix)
/// - Finds all associated letter files
/// - Returns complete set in LIFO order
/// - Returns error if no logs found or set is incomplete
fn find_next_multibyte_lifo_log_set(log_dir: &Path) -> ButtonResult<Vec<PathBuf>> {
    // Find highest bare number (reuse existing function logic)
    let next_bare_log = find_next_lifo_log_file(log_dir)?;

    // Extract number from filename
    let filename = next_bare_log
        .file_name()
        .ok_or_else(|| ButtonError::LogDirectoryError {
            path: next_bare_log.clone(),
            reason: "Invalid log filename",
        })?
        .to_string_lossy();

    let base_number = filename
        .parse::<u128>()
        .map_err(|_| ButtonError::MalformedLog {
            log_path: next_bare_log.clone(),
            reason: "Cannot parse log number",
        })?;

    // Find complete set
    find_multibyte_log_set(log_dir, base_number)
}

// ============================================================================
// UNIT TESTS FOR MULTI-BYTE OPERATIONS
// ============================================================================

#[cfg(test)]
mod multibyte_tests {
    use super::*;
    use std::env;

    #[test]
    fn test_detect_utf8_byte_count() {
        // 1-byte (ASCII)
        assert_eq!(detect_utf8_byte_count(0x41), Ok(1)); // 'A'
        assert_eq!(detect_utf8_byte_count(0x7F), Ok(1)); // DEL

        // 2-byte
        assert_eq!(detect_utf8_byte_count(0xC3), Ok(2)); // Latin supplement
        assert_eq!(detect_utf8_byte_count(0xDF), Ok(2)); // Latin supplement

        // 3-byte
        assert_eq!(detect_utf8_byte_count(0xE9), Ok(3)); // CJK
        assert_eq!(detect_utf8_byte_count(0xEF), Ok(3)); // CJK

        // 4-byte
        assert_eq!(detect_utf8_byte_count(0xF0), Ok(4)); // Emoji/supplementary
        assert_eq!(detect_utf8_byte_count(0xF4), Ok(4)); // Emoji/supplementary

        // Invalid
        assert!(detect_utf8_byte_count(0x80).is_err()); // Continuation byte
        assert!(detect_utf8_byte_count(0xF8).is_err()); // Invalid start
    }

    #[test]
    fn test_button_remove_multibyte_make_log_files() {
        let test_dir = env::temp_dir().join("button_test_multibyte_remove");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();

        let target_file = test_dir.join("target.txt");
        fs::write(&target_file, b"test").unwrap();
        let target_abs = target_file.canonicalize().unwrap();

        let log_dir = test_dir.join("logs");
        fs::create_dir_all(&log_dir).unwrap();
        let log_dir_abs = log_dir.canonicalize().unwrap();

        // User added 3-byte character at position 10
        // Create logs: 0.b, 0.a, 0 (all say "rmv at 10")
        button_remove_multibyte_make_log_files(&target_abs, 10, 3, &log_dir_abs).unwrap();

        // Verify files exist
        assert!(log_dir.join("0.b").exists(), "Should create 0.b");
        assert!(log_dir.join("0.a").exists(), "Should create 0.a");
        assert!(log_dir.join("0").exists(), "Should create 0");

        // Verify content
        let content_b = fs::read_to_string(log_dir.join("0.b")).unwrap();
        assert!(content_b.contains("rmv"));
        assert!(content_b.contains("10"));

        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_button_add_multibyte_make_log_files() {
        let test_dir = env::temp_dir().join("button_test_multibyte_add");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();

        let target_file = test_dir.join("target.txt");
        fs::write(&target_file, b"test").unwrap();
        let target_abs = target_file.canonicalize().unwrap();

        let log_dir = test_dir.join("logs");
        fs::create_dir_all(&log_dir).unwrap();
        let log_dir_abs = log_dir.canonicalize().unwrap();

        // User removed 3-byte character '阿' (E9 98 BF) at position 10
        // Create logs: 0.b (add BF), 0.a (add 98), 0 (add E9)
        let char_bytes = vec![0xE9, 0x98, 0xBF];
        button_add_multibyte_make_log_files(&target_abs, 10, &char_bytes, &log_dir_abs).unwrap();

        // Verify files exist
        assert!(log_dir.join("0.b").exists());
        assert!(log_dir.join("0.a").exists());
        assert!(log_dir.join("0").exists());

        // Verify content of 0.b (should have byte BF)
        let content_b = fs::read_to_string(log_dir.join("0.b")).unwrap();
        assert!(content_b.contains("add"));
        assert!(content_b.contains("10"));
        assert!(content_b.contains("BF"));

        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_find_multibyte_log_set() {
        let test_dir = env::temp_dir().join("button_test_find_set");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();

        // Create 3-byte log set
        fs::write(test_dir.join("5.b"), "test").unwrap();
        fs::write(test_dir.join("5.a"), "test").unwrap();
        fs::write(test_dir.join("5"), "test").unwrap();

        let log_set = find_multibyte_log_set(&test_dir, 5).unwrap();

        // Should be in LIFO order: 5.b, 5.a, 5
        assert_eq!(log_set.len(), 3);
        assert!(log_set[0].to_string_lossy().contains("5.b"));
        assert!(log_set[1].to_string_lossy().contains("5.a"));
        assert!(log_set[2].to_string_lossy().contains("5"));

        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_full_multibyte_undo_cycle() {
        // Test: user adds 3-byte character -> creates remove logs -> undo removes it
        let test_dir = env::temp_dir().join("button_test_multibyte_undo");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();

        let target_file = test_dir.join("target.txt");
        // File starts as "AB阿CD" where 阿 is at positions 2,3,4
        fs::write(&target_file, b"AB\xE9\x98\xBFCD").unwrap();
        let target_abs = target_file.canonicalize().unwrap();

        let log_dir = test_dir.join("logs");
        fs::create_dir_all(&log_dir).unwrap();
        let log_dir_abs = log_dir.canonicalize().unwrap();

        // User added '阿' at position 2, create remove logs
        button_remove_multibyte_make_log_files(&target_abs, 2, 3, &log_dir_abs).unwrap();

        // Perform undo (should remove 3 bytes at position 2)
        button_undo_multibyte_with_redo_support(&target_abs, &log_dir_abs, false, None).unwrap();

        // Verify: 阿 was removed, file is now "ABCD"
        let content = fs::read(&target_file).unwrap();
        assert_eq!(content, b"ABCD");

        // Verify: All log files were removed
        assert!(!log_dir.join("0.b").exists());
        assert!(!log_dir.join("0.a").exists());
        assert!(!log_dir.join("0").exists());

        let _ = fs::remove_dir_all(&test_dir);
    }
}

// ============================================================================
// PUBLIC API "Router" functions, that route user actions
// - button_make_character_action_changelog(etc)
// - button_undo_redo_next_inverse_changelog_pop_lifo(etc)
// ============================================================================

// ============================================================================
// PUBLIC API - PHASE 4: ROUTER FUNCTIONS
// ============================================================================

/// Creates a changelog entry for a character-level action (high-level API)
///
/// # Purpose
/// Main entry point for creating changelog entries. Automatically handles:
/// - Single-byte vs multi-byte characters
/// - User add vs remove vs hex-edit operations
/// - Proper inverse logging (log opposite of user action)
/// - Directory creation and absolute path handling
///
/// # Arguments
/// * `target_file` - File being edited (will be converted to absolute path)
/// * `character` - Character involved in action:
///   - Some(char): For user remove (log will restore it)
///   - Some(char): For user hex-edit (not used, see note below)
///   - None: For user add (no need to know what was added)
/// * `position` - Position in file where action occurred (0-indexed)
/// * `edit_type` - Type of user action (Add/Rmv/Edt)
/// * `log_directory_path` - Directory to write changelog files
///
/// # Returns
/// * `ButtonResult<()>` - Success or error
///
/// # Edit Type Logic
/// The edit_type describes what the USER did (not what the log will do):
/// - `EditType::Add`: User added a character → Log will say "remove"
/// - `EditType::Rmv`: User removed a character → Log will say "add" (with character bytes)
/// - `EditType::Edt`: User hex-edited → Log will say "edit" (with original byte)
///
/// # Character Parameter Usage
/// - For `Add`: character is None (don't need to know what user added)
/// - For `Rmv`: character is Some (need bytes to restore)
/// - For `Edt`: Not recommended to use this function (see `button_make_hexedit_changelog` instead)
///
/// # Multi-byte Handling
/// Automatically detects UTF-8 character length and creates multiple log files
/// with proper letter suffixes if needed.
///
/// # Examples
/// ```
/// // User added character 'A' at position 10
/// button_make_character_action_changelog(
///     Path::new("file.txt"),
///     None,  // Don't need to know what was added
///     10,
///     EditType::Add,
///     Path::new("./changelog_file")
/// )?;
///
/// // User removed character '阿' at position 20
/// button_make_character_action_changelog(
///     Path::new("file.txt"),
///     Some('阿'),  // Need character bytes to restore
///     20,
///     EditType::Rmv,
///     Path::new("./changelog_file")
/// )?;
/// ```
pub fn button_make_character_action_changelog(
    target_file: &Path,
    character: Option<char>,
    position: u128,
    edit_type: EditType,
    log_directory_path: &Path,
) -> ButtonResult<()> {
    // Convert paths to absolute
    let target_file_abs = fs::canonicalize(target_file).map_err(|e| {
        ButtonError::Io(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Cannot resolve target file path: {}", e),
        ))
    })?;

    let log_dir_abs = if log_directory_path.exists() {
        fs::canonicalize(log_directory_path).map_err(|e| ButtonError::Io(e))?
    } else {
        // Create directory and then canonicalize
        fs::create_dir_all(log_directory_path).map_err(|e| ButtonError::Io(e))?;
        fs::canonicalize(log_directory_path).map_err(|e| ButtonError::Io(e))?
    };

    #[cfg(debug_assertions)]
    println!(
        "Creating changelog for {:?} action at position {} (char: {:?})",
        edit_type, position, character
    );

    // Route based on user action type
    match edit_type {
        EditType::Add => {
            // User ADDED a character
            // Read the character from file to determine byte count
            let char_bytes = read_character_bytes_from_file(&target_file_abs, position)?;
            let byte_count = char_bytes.len();

            #[cfg(debug_assertions)]
            println!("  User added {}-byte character", byte_count);

            if byte_count == 1 {
                // Single-byte: create one "remove" log
                button_remove_byte_make_log_file(&target_file_abs, position, &log_dir_abs)?;
            } else {
                // Multi-byte: create multiple "remove" logs
                button_remove_multibyte_make_log_files(
                    &target_file_abs,
                    position,
                    byte_count,
                    &log_dir_abs,
                )?;
            }
        }

        EditType::Rmv => {
            // User REMOVED a character
            // Need the character to know what bytes to restore
            let ch = character.ok_or_else(|| ButtonError::InvalidUtf8 {
                position,
                byte_count: 0,
                reason: "Character required for remove operation",
            })?;

            // Convert character to UTF-8 bytes
            let mut char_bytes = [0u8; 4];
            let char_str = ch.encode_utf8(&mut char_bytes);
            let char_bytes_slice = char_str.as_bytes();
            let byte_count = char_bytes_slice.len();

            #[cfg(debug_assertions)]
            println!("  User removed {}-byte character '{}'", byte_count, ch);

            if byte_count == 1 {
                // Single-byte: create one "add" log
                button_add_byte_make_log_file(
                    &target_file_abs,
                    position,
                    char_bytes_slice[0],
                    &log_dir_abs,
                )?;
            } else {
                // Multi-byte: create multiple "add" logs
                button_add_multibyte_make_log_files(
                    &target_file_abs,
                    position,
                    char_bytes_slice,
                    &log_dir_abs,
                )?;
            }
        }

        EditType::Edt => {
            // Hex-edit: Not recommended to use this function
            // User should call button_make_hexedit_changelog directly
            return Err(ButtonError::InvalidUtf8 {
                position,
                byte_count: 1,
                reason: "Use button_make_hexedit_changelog for hex edits",
            });
        }
    }

    Ok(())
}

/// Creates a changelog entry for a hex-edit action
///
/// # Purpose
/// Specialized function for hex-edit operations (in-place byte replacement).
/// Unlike character add/remove, hex-edits don't change file length.
///
/// # Arguments
/// * `target_file` - File being edited (will be converted to absolute path)
/// * `position` - Position in file where hex-edit occurred (0-indexed)
/// * `original_byte` - The ORIGINAL byte value before user's edit
/// * `log_directory_path` - Directory to write changelog file
///
/// # Returns
/// * `ButtonResult<()>` - Success or error
///
/// # Inverse Changelog Logic
/// - User action: HEX-EDIT byte at position (original → new value)
/// - Log entry: EDT {original} at position (undo restores original)
///
/// # Note
/// This always creates a single log file (hex-edits are always single-byte).
///
/// # Examples
/// ```
/// // User hex-edited position 42: changed 0xFF to 0x61
/// button_make_hexedit_changelog(
///     Path::new("file.txt"),
///     42,
///     0xFF,  // Original value before edit
///     Path::new("./changelog_file")
/// )?;
/// ```
pub fn button_make_hexedit_changelog(
    target_file: &Path,
    position: u128,
    original_byte: u8,
    log_directory_path: &Path,
) -> ButtonResult<()> {
    // Convert paths to absolute
    let target_file_abs = fs::canonicalize(target_file).map_err(|e| {
        ButtonError::Io(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Cannot resolve target file path: {}", e),
        ))
    })?;

    let log_dir_abs = if log_directory_path.exists() {
        fs::canonicalize(log_directory_path).map_err(|e| ButtonError::Io(e))?
    } else {
        // Create directory and then canonicalize
        fs::create_dir_all(log_directory_path).map_err(|e| ButtonError::Io(e))?;
        fs::canonicalize(log_directory_path).map_err(|e| ButtonError::Io(e))?
    };

    #[cfg(debug_assertions)]
    println!(
        "Creating hex-edit changelog at position {} (original: 0x{:02X})",
        position, original_byte
    );

    // Hex-edits are always single-byte
    button_hexeditinplace_byte_make_log_file(
        &target_file_abs,
        position,
        original_byte,
        &log_dir_abs,
    )
}

// ============================================================================
// REDO SUPPORT - HELPER FUNCTIONS
// ============================================================================

/// Checks if a log directory is a redo directory
///
/// # Purpose
/// Determines whether we're processing undo logs or redo logs based on
/// the directory name. Used to prevent redo operations from creating
/// more redo logs (avoiding infinite redo chains).
///
/// # Arguments
/// * `log_directory_path` - Directory to check
///
/// # Returns
/// * `ButtonResult<bool>` - True if this is a redo directory, false if undo
///
/// # Detection Logic
/// Checks if directory name starts with "changelog_redo_"
/// - "changelog_file/" → false (undo directory)
/// - "changelog_redo_file/" → true (redo directory)
///
/// # Examples
/// ```
/// let is_redo = is_redo_directory(Path::new("./changelog_redo_myfile"))?;
/// assert_eq!(is_redo, true);
/// ```
fn is_redo_directory(log_directory_path: &Path) -> ButtonResult<bool> {
    // =================================================
    // Debug-Assert, Test-Assert, Production-Catch-Handle
    // =================================================

    debug_assert!(
        log_directory_path.is_absolute(),
        "Log directory must be absolute path"
    );

    #[cfg(test)]
    assert!(
        log_directory_path.is_absolute(),
        "Log directory must be absolute path"
    );

    if !log_directory_path.is_absolute() {
        return Err(ButtonError::AssertionViolation {
            check: "Log directory path must be absolute",
        });
    }

    // Extract directory name (last path segment)
    let dir_name = log_directory_path
        .file_name()
        .ok_or_else(|| ButtonError::LogDirectoryError {
            path: log_directory_path.to_path_buf(),
            reason: "Invalid directory path - no filename component",
        })?
        .to_string_lossy();

    // Check if it starts with redo prefix
    Ok(dir_name.starts_with(REDO_LOG_DIR_PREFIX))
}

/// Reads a single byte from file at specified position
///
/// # Purpose
/// Captures a byte value before it gets destroyed by an undo operation.
/// Used for creating inverse redo logs.
///
/// # Arguments
/// * `file_path` - File to read from (absolute path)
/// * `position` - Position of byte to read (0-indexed)
///
/// # Returns
/// * `ButtonResult<u8>` - The byte value at that position
///
/// # Use Case
/// When undoing a "remove" or "hex-edit" operation, we need to know
/// what byte is currently at the position before we modify it, so we
/// can create a redo log to restore it later.
///
/// # Examples
/// ```
/// // Before removing byte at position 10, capture it for redo log
/// let current_byte = read_single_byte_from_file(&file_path, 10)?;
/// // Now we can create redo log: "add {current_byte} at 10"
/// ```
pub fn read_single_byte_from_file(file_path: &Path, position: u128) -> ButtonResult<u8> {
    // =================================================
    // Debug-Assert, Test-Assert, Production-Catch-Handle
    // =================================================

    debug_assert!(file_path.exists(), "File must exist before reading");

    #[cfg(test)]
    assert!(file_path.exists(), "File must exist before reading");

    if !file_path.exists() {
        return Err(ButtonError::Io(io::Error::new(
            io::ErrorKind::NotFound,
            "File does not exist",
        )));
    }

    // Open file for reading
    let mut file = File::open(file_path).map_err(|e| ButtonError::Io(e))?;

    // Get file size for bounds checking
    let file_metadata = file.metadata().map_err(|e| ButtonError::Io(e))?;
    let file_size = file_metadata.len() as u128;

    // Validate position
    if position >= file_size {
        return Err(ButtonError::PositionOutOfBounds {
            position,
            file_size,
        });
    }

    // Seek to position
    file.seek(SeekFrom::Start(position as u64))
        .map_err(|e| ButtonError::Io(e))?;

    // Read single byte
    let mut byte_buffer = [0u8; 1];
    file.read_exact(&mut byte_buffer)
        .map_err(|e| ButtonError::Io(e))?;

    Ok(byte_buffer[0])
}

// ============================================================================
// MODIFIED ROUTER FUNCTION WITH REDO SUPPORT
// ============================================================================

/// Undoes the next changelog entry in LIFO order (high-level API)
///
/// # Purpose
/// Main entry point for undo/redo operations. Automatically detects whether
/// the next log is single-byte or multi-byte and calls the appropriate
/// undo function. **Now supports redo by creating inverse logs.**
///
/// # Arguments
/// * `target_file` - File to perform undo on (will be converted to absolute path)
/// * `log_directory_path` - Directory containing changelog files
///
/// # Returns
/// * `ButtonResult<()>` - Success or error
///
/// # Detection Logic
/// 1. **Undo vs Redo**: Checks if directory name starts with "changelog_redo_"
///    - If not → UNDO operation (creates redo logs)
///    - If yes → REDO operation (no redo log creation)
///
/// 2. **Single vs Multi-byte**: Finds the highest-numbered bare log file, then:
///    - If no letter-suffix files exist → single-byte undo
///    - If letter-suffix files exist (e.g., 10.a, 10.b) → multi-byte undo
///
/// # LIFO Behavior
/// Always processes the most recent change first (highest number).
///
/// # Redo Log Creation (Only for Undo Operations)
/// When undoing (not redoing), creates inverse logs in redo directory:
/// - Undo log says "rmv at P" → Captures byte at P → Redo log: "add {byte} at P"
/// - Undo log says "add X at P" → Redo log: "rmv at P"
/// - Undo log says "edt X at P" → Captures current byte → Redo log: "edt {current} at P"
///
/// # Error Handling
/// - No logs found → returns NoLogsFound error
/// - Malformed logs → quarantines and returns error
/// - File operation fails → leaves logs in place, returns error
/// - Success → removes processed log file(s), creates redo logs if applicable
///
/// # Examples
/// ```
/// // Undo the most recent change (creates redo log)
/// button_undo_redo_next_inverse_changelog_pop_lifo(
///     Path::new("file.txt"),
///     Path::new("./changelog_file")  // Undo directory
/// )?;
///
/// // Redo the most recent undo (no new redo logs created)
/// button_undo_redo_next_inverse_changelog_pop_lifo(
///     Path::new("file.txt"),
///     Path::new("./changelog_redo_file")  // Redo directory
/// )?;
/// ```
pub fn button_undo_redo_next_inverse_changelog_pop_lifo(
    target_file: &Path,
    log_directory_path: &Path,
) -> ButtonResult<()> {
    // Convert paths to absolute
    let target_file_abs = fs::canonicalize(target_file).map_err(|e| {
        ButtonError::Io(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Cannot resolve target file path: {}", e),
        ))
    })?;

    let log_dir_abs = fs::canonicalize(log_directory_path).map_err(|e| {
        ButtonError::Io(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Cannot resolve log directory path: {}", e),
        ))
    })?;

    // =========================================
    // REDO DETECTION: Check if this is undo or redo
    // =========================================
    let is_undo_operation = !is_redo_directory(&log_dir_abs)?;

    #[cfg(debug_assertions)]
    {
        if is_undo_operation {
            println!("This is an UNDO operation (will create redo logs)");
        } else {
            println!("This is a REDO operation (no redo logs will be created)");
        }
    }

    // Get redo directory path (only needed for undo operations)
    let redo_dir = if is_undo_operation {
        let redo_path = get_redo_changelog_directory_path(&target_file_abs)?;
        // Create redo directory if it doesn't exist
        if !redo_path.exists() {
            fs::create_dir_all(&redo_path).map_err(|e| ButtonError::Io(e))?;
        }
        Some(redo_path)
    } else {
        None
    };

    #[cfg(debug_assertions)]
    println!("Finding next changelog to undo...");

    // Find the next bare log file (highest number without letter suffix)
    let next_bare_log = find_next_lifo_log_file(&log_dir_abs)?;

    // Extract number from filename
    let filename = next_bare_log
        .file_name()
        .ok_or_else(|| ButtonError::LogDirectoryError {
            path: next_bare_log.clone(),
            reason: "Invalid log filename",
        })?
        .to_string_lossy();

    let base_number = filename
        .parse::<u128>()
        .map_err(|_| ButtonError::MalformedLog {
            log_path: next_bare_log.clone(),
            reason: "Cannot parse log number",
        })?;

    #[cfg(debug_assertions)]
    println!("  Found base log number: {}", base_number);

    // Check for letter-suffix files to determine if multi-byte
    let mut has_letter_files = false;

    // Bounded loop: check for letters a, b, c (max 3)
    for i in 0..(MAX_UTF8_BYTES - 1) {
        let letter = LOG_LETTER_SEQUENCE[i];
        let letter_path = log_dir_abs.join(format!("{}.{}", base_number, letter));

        if letter_path.exists() {
            has_letter_files = true;
            #[cfg(debug_assertions)]
            println!("  Found letter file: {}.{}", base_number, letter);
            break;
        }
    }

    // =========================================
    // ROUTE TO SINGLE-BYTE OR MULTI-BYTE HANDLER
    // =========================================
    if has_letter_files {
        #[cfg(debug_assertions)]
        println!("  Routing to multi-byte undo with redo support");

        button_undo_multibyte_with_redo_support(
            &target_file_abs,
            &log_dir_abs,
            is_undo_operation,
            redo_dir.as_deref(),
        )
    } else {
        #[cfg(debug_assertions)]
        println!("  Routing to single-byte undo with redo support");

        button_undo_single_byte_with_redo_support(
            &target_file_abs,
            &log_dir_abs,
            is_undo_operation,
            redo_dir.as_deref(),
        )
    }
}

// ============================================================================
// SINGLE-BYTE UNDO WITH REDO SUPPORT
// ============================================================================

/// Performs undo operation for single-byte changelog with redo support
///
/// # Purpose
/// Internal function that handles single-byte undo operations and optionally
/// creates inverse redo logs.
///
/// # Arguments
/// * `target_file` - File to perform undo on (absolute path)
/// * `log_dir` - Directory containing undo logs (absolute path)
/// * `is_undo_operation` - True if this is undo (not redo)
/// * `redo_dir` - Optional redo directory (Some for undo, None for redo)
///
/// # Returns
/// * `ButtonResult<()>` - Success or error
fn button_undo_single_byte_with_redo_support(
    target_file: &Path,
    log_dir: &Path,
    is_undo_operation: bool,
    redo_dir: Option<&Path>,
) -> ButtonResult<()> {
    // Step 1: Find next log file
    let log_file_path = find_next_lifo_log_file(log_dir)?;

    #[cfg(debug_assertions)]
    println!("Undoing log file: {}", log_file_path.display());

    // Step 2: Read and parse log file
    let log_entry = match read_log_file(&log_file_path) {
        Ok(entry) => entry,
        Err(e) => {
            // Log is malformed - quarantine it
            quarantine_bad_log(target_file, &log_file_path, "Failed to parse log file");
            return Err(e);
        }
    };

    // =========================================
    // REDO CAPTURE: Read data before destruction (if undo operation)
    // =========================================
    let captured_byte_for_redo = if is_undo_operation {
        match log_entry.edit_type() {
            EditType::Rmv => {
                // We're about to REMOVE a byte - capture it for redo
                let position = log_entry.position();
                match read_single_byte_from_file(target_file, position) {
                    Ok(byte) => {
                        #[cfg(debug_assertions)]
                        println!(
                            "  Captured byte 0x{:02X} at position {} for redo",
                            byte, position
                        );
                        Some(byte)
                    }
                    Err(e) => {
                        #[cfg(debug_assertions)]
                        eprintln!("  Warning: Could not capture byte for redo: {}", e);
                        None // Continue with undo, but redo log won't be created
                    }
                }
            }
            EditType::Edt => {
                // We're about to EDIT a byte - capture current value for redo
                let position = log_entry.position();
                match read_single_byte_from_file(target_file, position) {
                    Ok(byte) => {
                        #[cfg(debug_assertions)]
                        println!(
                            "  Captured current byte 0x{:02X} at position {} for redo",
                            byte, position
                        );
                        Some(byte)
                    }
                    Err(e) => {
                        #[cfg(debug_assertions)]
                        eprintln!("  Warning: Could not capture byte for redo: {}", e);
                        None
                    }
                }
            }
            EditType::Add => {
                // We're about to ADD a byte - nothing to capture (insertion doesn't destroy data)
                None
            }
        }
    } else {
        None // This is a redo operation - don't capture
    };

    // Step 3: Execute undo operation
    match execute_log_entry(target_file, &log_entry) {
        Ok(()) => {
            #[cfg(debug_assertions)]
            println!("Undo operation successful");

            // =========================================
            // REDO LOG CREATION: Create inverse log (if undo operation)
            // =========================================
            if is_undo_operation {
                if let Some(redo_directory) = redo_dir {
                    let redo_result = create_inverse_redo_log(
                        target_file,
                        redo_directory,
                        &log_entry,
                        captured_byte_for_redo,
                    );

                    if let Err(e) = redo_result {
                        // Non-fatal: redo log creation failed, but undo succeeded
                        #[cfg(debug_assertions)]
                        eprintln!("Warning: Could not create redo log: {}", e);

                        log_button_error(
                            target_file,
                            &format!("Could not create redo log: {}", e),
                            Some("button_undo_single_byte_with_redo_support"),
                        );
                    }
                }
            }

            // Step 4: Remove log file after successful undo
            if let Err(e) = fs::remove_file(&log_file_path) {
                #[cfg(debug_assertions)]
                eprintln!("Warning: Could not remove log file after undo: {}", e);

                log_button_error(
                    target_file,
                    &format!("Could not remove log file after successful undo: {}", e),
                    Some("button_undo_single_byte_with_redo_support"),
                );
            }

            Ok(())
        }
        Err(e) => {
            // Undo operation failed - leave log file in place
            #[cfg(debug_assertions)]
            eprintln!("Undo operation failed: {}", e);

            log_button_error(
                target_file,
                &format!("Undo operation failed: {}", e),
                Some("button_undo_single_byte_with_redo_support"),
            );

            Err(e)
        }
    }
}

// ============================================================================
// MULTI-BYTE UNDO WITH REDO SUPPORT
// ============================================================================

/// Performs undo operation for multi-byte changelog with redo support
///
/// # Purpose
/// Internal function that handles multi-byte undo operations and optionally
/// creates inverse redo logs.
///
/// # Critical Context: "Cheap Trick" Button Stack
/// Multi-byte log files use the "cheap trick" for WRITING operations:
/// - All log entries record the SAME position (position of first byte)
/// - When undoing: writes happen at position 0 repeatedly
/// - Each write pushes previous bytes forward automatically
/// - Example: Writing E9, 98, BF at position 0 → E9 pushes to 1, 98 pushes to 2
///
/// **However**, for READING (redo capture), we must read from ACTUAL positions:
/// - The bytes are at sequential positions 0, 1, 2 in the file
/// - NOT all at position 0 (that's just how we write them back)
/// - We must calculate: actual_position = base_position + byte_index
///
/// # Arguments
/// * `target_file` - File to perform undo on (absolute path)
/// * `log_dir` - Directory containing undo logs (absolute path)
/// * `is_undo_operation` - True if this is undo (not redo)
/// * `redo_dir` - Optional redo directory (Some for undo, None for redo)
///
/// # Returns
/// * `ButtonResult<()>` - Success or error
///
/// # Operation Flow
/// 1. Find and parse multi-byte log set (e.g., 10.b, 10.a, 10)
/// 2. **If undo**: Capture bytes from SEQUENTIAL positions (0,1,2) before destruction
/// 3. Execute undo operations (writes use "cheap trick" position)
/// 4. **If undo**: Create inverse redo logs with captured bytes
/// 5. Remove processed undo logs
///
/// # Why This Distinction Matters
/// **Writing (Cheap Trick)**: All logs say "position 0" for simplicity
/// - First add at 0 → places byte at 0
/// - Second add at 0 → pushes first byte to 1, places new byte at 0
/// - Result: Bytes naturally end up at 0, 1, 2
///
/// **Reading (Redo Capture)**: Must use ACTUAL file positions
/// - Byte 0 is at position 0 in file
/// - Byte 1 is at position 1 in file
/// - Byte 2 is at position 2 in file
/// - If we read position 0 three times, we get the same byte three times (BUG!)
fn button_undo_multibyte_with_redo_support(
    target_file: &Path,
    log_dir: &Path,
    is_undo_operation: bool,
    redo_dir: Option<&Path>,
) -> ButtonResult<()> {
    // =========================================
    // STEP 1: Find and Parse Log Files
    // =========================================

    // Find next multi-byte log set (e.g., "10.b", "10.a", "10")
    let log_files = find_next_multibyte_lifo_log_set(log_dir)?;

    #[cfg(debug_assertions)]
    {
        println!("Undoing multi-byte log set ({} files):", log_files.len());
        for log_file in &log_files {
            println!("  - {}", log_file.display());
        }
    }

    // Parse all log files into LogEntry structs
    let mut log_entries = Vec::with_capacity(log_files.len());

    for log_file_path in &log_files {
        match read_log_file(log_file_path) {
            Ok(entry) => log_entries.push(entry),
            Err(e) => {
                // Log is malformed - quarantine entire set
                for bad_log in &log_files {
                    quarantine_bad_log(
                        target_file,
                        bad_log,
                        "Part of malformed multi-byte log set",
                    );
                }
                return Err(e);
            }
        }
    }

    // =========================================
    // STEP 2: REDO CAPTURE (If Undo Operation)
    // =========================================
    // **CRITICAL**: Must read from ACTUAL file positions, not log positions!
    // Log positions all say 0 (cheap trick), but bytes are at 0, 1, 2...

    let mut captured_bytes_for_redo = Vec::new();

    if is_undo_operation {
        // Get base position from first log entry (all entries have same position due to cheap trick)
        let base_position = log_entries[0].position();
        let byte_count = log_entries.len();

        #[cfg(debug_assertions)]
        println!(
            "  Capturing {} bytes from ACTUAL positions {} to {} (not log position {})",
            byte_count,
            base_position,
            base_position + byte_count as u128 - 1,
            base_position
        );

        // Bounded loop: max 4 iterations (MAX_UTF8_BYTES)
        for byte_index in 0..byte_count {
            // =================================================
            // Debug-Assert, Test-Assert, Production-Catch-Handle
            // =================================================

            debug_assert!(
                byte_index < MAX_UTF8_BYTES,
                "Byte index exceeded max UTF-8 bytes"
            );

            #[cfg(test)]
            assert!(
                byte_index < MAX_UTF8_BYTES,
                "Byte index exceeded max UTF-8 bytes"
            );

            if byte_index >= MAX_UTF8_BYTES {
                return Err(ButtonError::AssertionViolation {
                    check: "Too many log entries in set",
                });
            }

            let log_entry = &log_entries[byte_index];

            // **KEY CALCULATION**: Actual position in file
            // - base_position: what all logs say (e.g., 0)
            // - byte_index: which byte in the sequence (0, 1, 2)
            // - actual_position: where byte really is in file (0, 1, 2)
            let actual_file_position = base_position + byte_index as u128;

            let captured_byte = match log_entry.edit_type() {
                EditType::Rmv => {
                    // About to REMOVE byte - capture it from ACTUAL position
                    match read_single_byte_from_file(target_file, actual_file_position) {
                        Ok(byte) => {
                            #[cfg(debug_assertions)]
                            println!(
                                "    Captured byte 0x{:02X} from ACTUAL position {} (log says {}, byte {}/{})",
                                byte,
                                actual_file_position,
                                base_position,
                                byte_index + 1,
                                byte_count
                            );
                            Some(byte)
                        }
                        Err(e) => {
                            #[cfg(debug_assertions)]
                            eprintln!(
                                "    Warning: Could not capture byte at position {}: {}",
                                actual_file_position, e
                            );
                            None
                        }
                    }
                }
                EditType::Edt => {
                    // About to EDIT byte - capture current value from ACTUAL position
                    match read_single_byte_from_file(target_file, actual_file_position) {
                        Ok(byte) => {
                            #[cfg(debug_assertions)]
                            println!(
                                "    Captured byte 0x{:02X} from ACTUAL position {} for hex-edit redo",
                                byte, actual_file_position
                            );
                            Some(byte)
                        }
                        Err(e) => {
                            #[cfg(debug_assertions)]
                            eprintln!(
                                "    Warning: Could not capture byte at position {}: {}",
                                actual_file_position, e
                            );
                            None
                        }
                    }
                }
                EditType::Add => {
                    // Insertion doesn't destroy data - nothing to capture
                    None
                }
            };

            captured_bytes_for_redo.push(captured_byte);
        }

        #[cfg(debug_assertions)]
        println!(
            "  Captured {} bytes for redo: {:?}",
            captured_bytes_for_redo.len(),
            captured_bytes_for_redo
                .iter()
                .map(|opt| match opt {
                    Some(b) => format!("0x{:02X}", b),
                    None => "None".to_string(),
                })
                .collect::<Vec<_>>()
        );
    }

    // =========================================
    // STEP 3: Execute Undo Operations
    // =========================================
    // Operations use log positions (cheap trick - all at position 0)

    // Bounded loop: max 4 iterations (MAX_UTF8_BYTES)
    for (i, log_entry) in log_entries.iter().enumerate() {
        // =================================================
        // Debug-Assert, Test-Assert, Production-Catch-Handle
        // =================================================

        debug_assert!(
            i < MAX_UTF8_BYTES,
            "Log entry index exceeded max UTF-8 bytes"
        );

        #[cfg(test)]
        assert!(
            i < MAX_UTF8_BYTES,
            "Log entry index exceeded max UTF-8 bytes"
        );

        if i >= MAX_UTF8_BYTES {
            return Err(ButtonError::AssertionViolation {
                check: "Too many log entries in set",
            });
        }

        // Execute operation using position from log (cheap trick position)
        match execute_log_entry(target_file, log_entry) {
            Ok(()) => {
                #[cfg(debug_assertions)]
                println!("  Executed log entry {}/{}", i + 1, log_entries.len());
            }
            Err(e) => {
                // Operation failed - leave all logs in place
                #[cfg(debug_assertions)]
                eprintln!(
                    "  Failed at log entry {}/{}: {}",
                    i + 1,
                    log_entries.len(),
                    e
                );

                log_button_error(
                    target_file,
                    &format!("Multi-byte undo failed at entry {}: {}", i + 1, e),
                    Some("button_undo_multibyte_with_redo_support"),
                );

                return Err(e);
            }
        }
    }

    // =========================================
    // STEP 4: Create Redo Logs (If Undo Operation)
    // =========================================
    // Use captured bytes to create inverse redo logs

    if is_undo_operation {
        if let Some(redo_directory) = redo_dir {
            let redo_result = create_inverse_redo_logs_multibyte(
                target_file,
                redo_directory,
                &log_entries,
                &captured_bytes_for_redo,
            );

            if let Err(e) = redo_result {
                // Non-fatal: redo log creation failed, but undo succeeded
                #[cfg(debug_assertions)]
                eprintln!("Warning: Could not create redo logs: {}", e);

                log_button_error(
                    target_file,
                    &format!("Could not create redo logs: {}", e),
                    Some("button_undo_multibyte_with_redo_support"),
                );
            }
        }
    }

    // =========================================
    // STEP 5: Cleanup - Remove Processed Logs
    // =========================================

    for log_file_path in &log_files {
        if let Err(e) = fs::remove_file(log_file_path) {
            #[cfg(debug_assertions)]
            eprintln!(
                "Warning: Could not remove log file {}: {}",
                log_file_path.display(),
                e
            );

            log_button_error(
                target_file,
                &format!("Could not remove log file after undo: {}", e),
                Some("button_undo_multibyte_with_redo_support"),
            );
        }
    }

    #[cfg(debug_assertions)]
    println!("Multi-byte undo completed successfully");

    Ok(())
}

// ============================================================================
// REDO LOG CREATION HELPERS
// ============================================================================

/// Creates inverse redo log for a single-byte operation
///
/// # Purpose
/// After successfully undoing an operation, create the inverse log entry
/// in the redo directory so the undo can be redone later.
///
/// # Arguments
/// * `target_file` - Target file (for error logging)
/// * `redo_dir` - Redo directory to write log to
/// * `undo_log_entry` - The log entry we just executed
/// * `captured_byte` - Byte captured before destruction (for Rmv/Edt)
///
/// # Returns
/// * `ButtonResult<()>` - Success or error
///
/// # Inverse Logic
/// | Undo Log Was | We Executed | Redo Log Should Be |
/// |--------------|-------------|-------------------|
/// | rmv at P | Removed byte X | add X at P |
/// | add X at P | Added byte X | rmv at P |
/// | edt Y at P | Edited to Y | edt X at P |
fn create_inverse_redo_log(
    target_file: &Path,
    redo_dir: &Path,
    undo_log_entry: &LogEntry,
    captured_byte: Option<u8>,
) -> ButtonResult<()> {
    #[cfg(debug_assertions)]
    println!("Creating inverse redo log...");

    let position = undo_log_entry.position();

    // Build inverse log entry
    let inverse_log_entry = match undo_log_entry.edit_type() {
        EditType::Rmv => {
            // Undo log said "rmv" - we removed a byte
            // Redo log should say "add {captured_byte}"
            let byte = captured_byte.ok_or_else(|| ButtonError::InvalidUtf8 {
                position,
                byte_count: 1,
                reason: "Cannot create redo log: no byte was captured",
            })?;

            #[cfg(debug_assertions)]
            println!("  Inverse: rmv -> add 0x{:02X} at {}", byte, position);

            LogEntry::new(EditType::Add, position, Some(byte))
                .map_err(|e| ButtonError::AssertionViolation { check: e })?
        }

        EditType::Add => {
            // Undo log said "add X" - we added a byte
            // Redo log should say "rmv"
            #[cfg(debug_assertions)]
            println!("  Inverse: add -> rmv at {}", position);

            LogEntry::new(EditType::Rmv, position, None)
                .map_err(|e| ButtonError::AssertionViolation { check: e })?
        }

        EditType::Edt => {
            // Undo log said "edt Y" - we edited to Y
            // Redo log should say "edt {captured_current_byte}"
            let byte = captured_byte.ok_or_else(|| ButtonError::InvalidUtf8 {
                position,
                byte_count: 1,
                reason: "Cannot create redo log: no byte was captured",
            })?;

            #[cfg(debug_assertions)]
            println!("  Inverse: edt -> edt 0x{:02X} at {}", byte, position);

            LogEntry::new(EditType::Edt, position, Some(byte))
                .map_err(|e| ButtonError::AssertionViolation { check: e })?
        }
    };

    // Write to redo directory
    write_log_entry_to_file(target_file, redo_dir, &inverse_log_entry)?;

    #[cfg(debug_assertions)]
    println!("  Redo log created successfully");

    Ok(())
}

/// Creates inverse redo logs for a multi-byte operation
///
/// # Purpose
/// After successfully undoing a multi-byte operation, create the inverse log entries
/// in the redo directory.
///
/// # Arguments
/// * `target_file` - Target file (for error logging only - not modified)
/// * `redo_dir` - Redo directory to write logs to
/// * `undo_log_entries` - The log entries we just executed
/// * `captured_bytes` - Bytes captured before destruction (for Rmv/Edt)
///
/// # Error Logging
/// - **Debug builds**: Verbose console output with full paths and details
/// - **Test builds**: Assertions that panic on invalid state
/// - **Production builds**: Terse error logs via `log_button_error()`, no panic
///
/// # Returns
/// * `ButtonResult<()>` - Success or error
fn create_inverse_redo_logs_multibyte(
    target_file: &Path,
    redo_dir: &Path,
    undo_log_entries: &[LogEntry],
    captured_bytes: &[Option<u8>],
) -> ButtonResult<()> {
    // =================================================
    // Debug-Assert, Test-Assert, Production-Catch-Handle
    // =================================================

    // Debug build: verbose output
    #[cfg(debug_assertions)]
    println!("Creating inverse redo logs for multi-byte operation...");

    // Test build: strict validation
    #[cfg(test)]
    {
        assert!(
            !undo_log_entries.is_empty(),
            "Must have at least one log entry"
        );
        assert_eq!(
            undo_log_entries.len(),
            captured_bytes.len(),
            "Captured bytes count must match log entries count"
        );
    }

    // Production build: safe validation without panic
    if undo_log_entries.is_empty() {
        log_button_error(
            target_file,
            "Cannot create redo logs: no undo log entries provided",
            Some("create_inverse_redo_logs_multibyte"),
        );
        return Err(ButtonError::AssertionViolation {
            check: "Empty log entries array",
        });
    }

    if undo_log_entries.len() != captured_bytes.len() {
        log_button_error(
            target_file,
            "Cannot create redo logs: captured bytes count mismatch",
            Some("create_inverse_redo_logs_multibyte"),
        );
        return Err(ButtonError::AssertionViolation {
            check: "Captured bytes count mismatch",
        });
    }

    // Get base log number for redo logs
    let base_log_number = match get_next_log_number(redo_dir) {
        Ok(num) => num,
        Err(e) => {
            // Debug: verbose error
            #[cfg(debug_assertions)]
            eprintln!("Failed to get next log number: {}", e);

            // Production: log error
            log_button_error(
                target_file,
                &format!("Failed to get next redo log number: {}", e),
                Some("create_inverse_redo_logs_multibyte"),
            );
            return Err(e);
        }
    };

    let byte_count = undo_log_entries.len();

    // Bounded loop: max 4 iterations
    for (byte_index, undo_log_entry) in undo_log_entries.iter().enumerate() {
        // =================================================
        // Debug-Assert, Test-Assert, Production-Catch-Handle
        // =================================================

        debug_assert!(
            byte_index < MAX_UTF8_BYTES,
            "Byte index exceeded max UTF-8 bytes"
        );

        #[cfg(test)]
        assert!(
            byte_index < MAX_UTF8_BYTES,
            "Byte index exceeded max UTF-8 bytes"
        );

        if byte_index >= MAX_UTF8_BYTES {
            log_button_error(
                target_file,
                "Too many log entries in redo set",
                Some("create_inverse_redo_logs_multibyte"),
            );
            return Err(ButtonError::AssertionViolation {
                check: "Too many log entries",
            });
        }

        let position = undo_log_entry.position();
        let captured_byte = captured_bytes.get(byte_index).and_then(|b| *b);

        // Build inverse log entry
        let inverse_log_entry = match undo_log_entry.edit_type() {
            EditType::Rmv => {
                // Undo removed a byte - redo should add it back
                let byte = captured_byte.ok_or_else(|| {
                    // Debug: verbose error
                    #[cfg(debug_assertions)]
                    eprintln!(
                        "Cannot create redo log: no byte captured at index {}",
                        byte_index
                    );

                    // Production: log error
                    log_button_error(
                        target_file,
                        &format!(
                            "Cannot create redo log: no byte captured at index {}",
                            byte_index
                        ),
                        Some("create_inverse_redo_logs_multibyte"),
                    );

                    ButtonError::InvalidUtf8 {
                        position,
                        byte_count: byte_index + 1,
                        reason: "No byte captured for redo",
                    }
                })?;

                LogEntry::new(EditType::Add, position, Some(byte))
                    .map_err(|e| ButtonError::AssertionViolation { check: e })?
            }

            EditType::Add => {
                // Undo added a byte - redo should remove it
                LogEntry::new(EditType::Rmv, position, None)
                    .map_err(|e| ButtonError::AssertionViolation { check: e })?
            }

            EditType::Edt => {
                // Undo edited a byte - redo should edit back
                let byte = captured_byte.ok_or_else(|| {
                    #[cfg(debug_assertions)]
                    eprintln!(
                        "Cannot create redo log: no byte captured for hex-edit at index {}",
                        byte_index
                    );

                    log_button_error(
                        target_file,
                        &format!(
                            "Cannot create redo log: no byte captured at index {}",
                            byte_index
                        ),
                        Some("create_inverse_redo_logs_multibyte"),
                    );

                    ButtonError::InvalidUtf8 {
                        position,
                        byte_count: byte_index + 1,
                        reason: "No byte captured for hex-edit redo",
                    }
                })?;

                LogEntry::new(EditType::Edt, position, Some(byte))
                    .map_err(|e| ButtonError::AssertionViolation { check: e })?
            }
        };

        // Get letter suffix
        let letter_suffix = get_log_file_letter_suffix(byte_index, byte_count);

        // Build filename
        let filename = match letter_suffix {
            Some(letter) => format!("{}.{}", base_log_number, letter),
            None => base_log_number.to_string(),
        };

        let log_file_path = redo_dir.join(&filename);

        // Serialize and write
        let log_content = inverse_log_entry.to_file_format();

        if let Err(e) = fs::write(&log_file_path, log_content) {
            // Debug: verbose error
            #[cfg(debug_assertions)]
            eprintln!("Failed to write redo log file {}: {}", filename, e);

            // Production: log error
            log_button_error(
                target_file,
                &format!("Failed to write redo log file {}: {}", filename, e),
                Some("create_inverse_redo_logs_multibyte"),
            );

            return Err(ButtonError::Io(e));
        }

        // Debug: success message
        #[cfg(debug_assertions)]
        println!("  Created redo log file: {}", filename);
    }

    Ok(())
}

/// Helper function to build changelog directory path from target file
///
/// # Purpose
/// Constructs the standard changelog directory path for a target file.
/// Format: `{parent_dir}/changelog_{filename_without_extension}/`
///
/// # Arguments
/// * `target_file` - The file being edited
///
/// # Returns
/// * `ButtonResult<PathBuf>` - Path to changelog directory
///
/// # Examples
/// ```
/// // File: /home/user/documents/myfile.txt
/// // Returns: /home/user/documents/changelog_myfile/
/// let log_dir = get_undo_changelog_directory_path(Path::new("/home/user/documents/myfile.txt"))?;
/// ```
pub fn get_undo_changelog_directory_path(target_file: &Path) -> ButtonResult<PathBuf> {
    // Get parent directory
    let parent_dir = target_file
        .parent()
        .ok_or_else(|| ButtonError::LogDirectoryError {
            path: target_file.to_path_buf(),
            reason: "Cannot determine parent directory",
        })?;

    // Get filename WITHOUT the period (remove all dots)
    let file_name = target_file
        .file_name()
        .ok_or_else(|| ButtonError::LogDirectoryError {
            path: target_file.to_path_buf(),
            reason: "Cannot determine filename",
        })?
        .to_string_lossy();

    // Remove ALL periods from filename
    let file_name_no_dots = file_name.replace('.', "");

    // Build changelog directory name
    let log_dir_name = format!("{}{}", LOG_DIR_PREFIX, file_name_no_dots);
    let log_dir_path = parent_dir.join(log_dir_name);

    Ok(log_dir_path)
}

/// Helper function to build redo changelog directory path from target file
///
/// # Purpose
/// Constructs the standard redo changelog directory path for a target file.
/// Format: `{parent_dir}/changelog_redo_{filename_without_extension}/`
///
/// # Arguments
/// * `target_file` - The file being edited
///
/// # Returns
/// * `ButtonResult<PathBuf>` - Path to redo changelog directory
///
/// # Examples
/// ```
/// // File: /home/user/documents/myfile.txt
/// // Returns: /home/user/documents/changelog_redo_myfile/
/// let redo_dir = get_redo_changelog_directory_path(Path::new("/home/user/documents/myfile.txt"))?;
/// ```
pub fn get_redo_changelog_directory_path(target_file: &Path) -> ButtonResult<PathBuf> {
    // Get parent directory
    let parent_dir = target_file
        .parent()
        .ok_or_else(|| ButtonError::LogDirectoryError {
            path: target_file.to_path_buf(),
            reason: "Cannot determine parent directory",
        })?;

    // Get filename WITHOUT the period (remove all dots)
    let file_name = target_file
        .file_name()
        .ok_or_else(|| ButtonError::LogDirectoryError {
            path: target_file.to_path_buf(),
            reason: "Cannot determine filename",
        })?
        .to_string_lossy();

    // Remove ALL periods from filename
    let file_name_no_dots = file_name.replace('.', "");

    // Build redo changelog directory name
    let redo_dir_name = format!("{}{}", REDO_LOG_DIR_PREFIX, file_name_no_dots);
    let redo_dir_path = parent_dir.join(redo_dir_name);

    Ok(redo_dir_path)
}

/// Clears all redo changelog files for a target file
///
/// # Purpose
/// When a normal edit action occurs (not an undo), all redo logs should be cleared
/// because the redo history is no longer valid.
///
/// # Arguments
/// * `target_file` - The file being edited
///
/// # Returns
/// * `ButtonResult<()>` - Success or error
///
/// # Behavior
/// - Finds or creates redo directory path
/// - Removes all files in redo directory
/// - Leaves directory structure intact (empty directory)
/// - Non-fatal: if directory doesn't exist, returns Ok
///
/// # Examples
/// ```
/// // User makes a normal edit - clear redo history
/// button_clear_all_redo_logs(Path::new("file.txt"))?;
/// ```
pub fn button_clear_all_redo_logs(target_file: &Path) -> ButtonResult<()> {
    let redo_dir = get_redo_changelog_directory_path(target_file)?;

    // If directory doesn't exist, nothing to clear
    if !redo_dir.exists() {
        return Ok(());
    }

    #[cfg(debug_assertions)]
    println!("Clearing redo logs in: {}", redo_dir.display());

    // Read and remove all files in directory
    let entries = fs::read_dir(&redo_dir).map_err(|e| ButtonError::Io(e))?;

    // Bounded loop: iterate through directory entries
    const MAX_REDO_FILES: usize = 10_000_000;
    let mut file_count: usize = 0;

    for entry_result in entries {
        // =================================================
        // Debug-Assert, Test-Assert, Production-Catch-Handle
        // =================================================

        debug_assert!(
            file_count < MAX_REDO_FILES,
            "Redo file count exceeded safety limit"
        );

        #[cfg(test)]
        assert!(
            file_count < MAX_REDO_FILES,
            "Redo file count exceeded safety limit"
        );

        if file_count >= MAX_REDO_FILES {
            return Err(ButtonError::LogDirectoryError {
                path: redo_dir.clone(),
                reason: "Too many redo files (safety limit)",
            });
        }

        file_count += 1;

        let entry = entry_result.map_err(|e| ButtonError::Io(e))?;
        let entry_path = entry.path();

        // Only remove files (not subdirectories)
        if entry_path.is_file() {
            if let Err(e) = fs::remove_file(&entry_path) {
                #[cfg(debug_assertions)]
                eprintln!(
                    "Warning: Could not remove redo log {}: {}",
                    entry_path.display(),
                    e
                );

                // Non-fatal: continue clearing other files
                log_button_error(
                    target_file,
                    &format!("Could not remove redo log: {}", e),
                    Some("button_clear_all_redo_logs"),
                );
            }
        }
    }

    #[cfg(debug_assertions)]
    println!("  Cleared {} redo log file(s)", file_count);

    Ok(())
}

// ============================================================================
// UNIT TESTS FOR ROUTER FUNCTIONS
// ============================================================================

#[cfg(test)]
mod router_tests {
    use super::*;
    use std::env;

    #[test]
    fn test_button_make_character_action_changelog_add_single_byte() {
        let test_dir = env::temp_dir().join("button_test_router_add_single");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();

        let target_file = test_dir.join("target.txt");
        fs::write(&target_file, b"ABXCD").unwrap(); // User added 'X' at position 2

        let log_dir = test_dir.join("logs");

        // User added single-byte character at position 2
        button_make_character_action_changelog(
            &target_file,
            None, // Don't need to know what was added
            2,
            EditType::Add,
            &log_dir,
        )
        .unwrap();

        // Should create one "remove" log
        assert!(log_dir.join("0").exists());

        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_button_make_character_action_changelog_remove_single_byte() {
        let test_dir = env::temp_dir().join("button_test_router_remove_single");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();

        let target_file = test_dir.join("target.txt");
        fs::write(&target_file, b"ABCD").unwrap();

        let log_dir = test_dir.join("logs");

        // User removed 'X' (0x58) at position 2
        button_make_character_action_changelog(
            &target_file,
            Some('X'), // Need character to restore
            2,
            EditType::Rmv,
            &log_dir,
        )
        .unwrap();

        // Should create one "add" log
        assert!(log_dir.join("0").exists());

        let content = fs::read_to_string(log_dir.join("0")).unwrap();
        assert!(content.contains("add"));
        assert!(content.contains("58")); // Hex for 'X'

        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_button_make_character_action_changelog_add_multibyte() {
        let test_dir = env::temp_dir().join("button_test_router_add_multi");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();

        let target_file = test_dir.join("target.txt");
        // User added '阿' at position 2
        fs::write(&target_file, b"AB\xE9\x98\xBFCD").unwrap();

        let log_dir = test_dir.join("logs");

        // User added 3-byte character at position 2
        button_make_character_action_changelog(&target_file, None, 2, EditType::Add, &log_dir)
            .unwrap();

        // Should create three "remove" logs
        assert!(log_dir.join("0.b").exists());
        assert!(log_dir.join("0.a").exists());
        assert!(log_dir.join("0").exists());

        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_button_make_character_action_changelog_remove_multibyte() {
        let test_dir = env::temp_dir().join("button_test_router_remove_multi");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();

        let target_file = test_dir.join("target.txt");
        fs::write(&target_file, b"ABCD").unwrap();

        let log_dir = test_dir.join("logs");

        // User removed '阿' at position 2
        button_make_character_action_changelog(
            &target_file,
            Some('阿'),
            2,
            EditType::Rmv,
            &log_dir,
        )
        .unwrap();

        // Should create three "add" logs with correct bytes
        assert!(log_dir.join("0.b").exists());
        assert!(log_dir.join("0.a").exists());
        assert!(log_dir.join("0").exists());

        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_button_make_hexedit_changelog() {
        let test_dir = env::temp_dir().join("button_test_router_hexedit");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();

        let target_file = test_dir.join("target.txt");
        fs::write(&target_file, b"ABCD").unwrap();

        let log_dir = test_dir.join("logs");

        // User hex-edited position 2: 0x43 ('C') to something else
        button_make_hexedit_changelog(&target_file, 2, 0x43, &log_dir).unwrap();

        // Should create one "edit" log
        assert!(log_dir.join("0").exists());

        let content = fs::read_to_string(log_dir.join("0")).unwrap();
        assert!(content.contains("edt"));
        assert!(content.contains("43"));

        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_button_undo_next_changelog_lifo_single_byte() {
        let test_dir = env::temp_dir().join("button_test_router_undo_single");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();

        let target_file = test_dir.join("target.txt");
        fs::write(&target_file, b"ABXCD").unwrap(); // User added 'X' at position 2

        let log_dir = test_dir.join("logs");

        // Create log for user add
        button_make_character_action_changelog(&target_file, None, 2, EditType::Add, &log_dir)
            .unwrap();

        // Undo should remove 'X'
        button_undo_redo_next_inverse_changelog_pop_lifo(&target_file, &log_dir).unwrap();

        let content = fs::read(&target_file).unwrap();
        assert_eq!(content, b"ABCD");

        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_button_undo_next_changelog_lifo_multibyte() {
        let test_dir = env::temp_dir().join("button_test_router_undo_multi");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();

        let target_file = test_dir.join("target.txt");
        fs::write(&target_file, b"AB\xE9\x98\xBFCD").unwrap(); // User added '阿'

        let log_dir = test_dir.join("logs");

        // Create logs for user add
        button_make_character_action_changelog(&target_file, None, 2, EditType::Add, &log_dir)
            .unwrap();

        // Undo should remove '阿'
        button_undo_redo_next_inverse_changelog_pop_lifo(&target_file, &log_dir).unwrap();

        let content = fs::read(&target_file).unwrap();
        assert_eq!(content, b"ABCD");

        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_get_changelog_directory_path() {
        let target_file = Path::new("/home/user/documents/myfile.txt");
        let log_dir = get_undo_changelog_directory_path(target_file).unwrap();

        assert!(log_dir.to_string_lossy().contains("changelog_myfile"));
    }

    #[test]
    fn test_get_redo_changelog_directory_path() {
        let target_file = Path::new("/home/user/documents/myfile.txt");
        let redo_dir = get_redo_changelog_directory_path(target_file).unwrap();

        assert!(redo_dir.to_string_lossy().contains("changelog_redo_myfile"));
    }

    #[test]
    fn test_button_clear_all_redo_logs() {
        let test_dir = env::temp_dir().join("button_test_clear_redo");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();

        let target_file = test_dir.join("target.txt");
        fs::write(&target_file, b"test").unwrap();

        // Manually create redo directory with some files
        let redo_dir = test_dir.join("changelog_redo_targettxt");
        fs::create_dir_all(&redo_dir).unwrap();
        fs::write(redo_dir.join("0"), "test").unwrap();
        fs::write(redo_dir.join("1"), "test").unwrap();
        fs::write(redo_dir.join("2"), "test").unwrap();

        // Clear redo logs
        button_clear_all_redo_logs(&target_file).unwrap();

        // Files should be removed
        assert!(!redo_dir.join("0").exists());
        assert!(!redo_dir.join("1").exists());
        assert!(!redo_dir.join("2").exists());

        // Directory should still exist (empty)
        assert!(redo_dir.exists());

        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_full_workflow_with_routers() {
        // Test complete workflow: add, remove, undo, undo
        let test_dir = env::temp_dir().join("button_test_full_workflow");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();

        let target_file = test_dir.join("target.txt");
        fs::write(&target_file, b"AB").unwrap(); // Start: "AB"

        let log_dir = test_dir.join("logs");

        // User adds 'X' at position 2: "AB" -> "ABX"
        fs::write(&target_file, b"ABX").unwrap();
        button_make_character_action_changelog(&target_file, None, 2, EditType::Add, &log_dir)
            .unwrap();

        // User adds 'Y' at position 3: "ABX" -> "ABXY"
        fs::write(&target_file, b"ABXY").unwrap();
        button_make_character_action_changelog(&target_file, None, 3, EditType::Add, &log_dir)
            .unwrap();

        // Undo last (remove 'Y'): "ABXY" -> "ABX"
        button_undo_redo_next_inverse_changelog_pop_lifo(&target_file, &log_dir).unwrap();
        let content = fs::read(&target_file).unwrap();
        assert_eq!(content, b"ABX");

        // Undo again (remove 'X'): "ABX" -> "AB"
        button_undo_redo_next_inverse_changelog_pop_lifo(&target_file, &log_dir).unwrap();
        let content = fs::read(&target_file).unwrap();
        assert_eq!(content, b"AB");

        let _ = fs::remove_dir_all(&test_dir);
    }
}

// ============================================================================
// UNIT TESTS FOR REDO-AWARE UNDO FUNCTIONS
// ============================================================================

#[cfg(test)]
mod redo_aware_undo_tests {
    use super::*;
    use std::env;

    // ========================================================================
    // Tests for button_undo_single_byte_with_redo_support (ACTUAL function used)
    // ========================================================================

    #[test]
    fn test_single_byte_undo_remove_creates_redo() {
        // Test: undo removes a byte AND creates redo log to restore it
        let test_dir = env::temp_dir().join("test_single_undo_remove_redo");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();

        let target_file = test_dir.join("target.txt");
        fs::write(&target_file, b"ABXCD").unwrap(); // File with 'X' at position 2
        let target_abs = target_file.canonicalize().unwrap();

        let log_dir = test_dir.join("logs");
        fs::create_dir_all(&log_dir).unwrap();
        let log_dir_abs = log_dir.canonicalize().unwrap();

        let redo_dir = test_dir.join("redo_logs");
        fs::create_dir_all(&redo_dir).unwrap();
        let redo_dir_abs = redo_dir.canonicalize().unwrap();

        // Create undo log: "rmv at position 2"
        let log_entry = LogEntry::new(EditType::Rmv, 2, None).unwrap();
        fs::write(log_dir.join("0"), log_entry.to_file_format()).unwrap();

        // Execute undo WITH redo support
        button_undo_single_byte_with_redo_support(
            &target_abs,
            &log_dir_abs,
            true, // is_undo_operation = true (will create redo)
            Some(&redo_dir_abs),
        )
        .unwrap();

        // Verify: byte removed
        let content = fs::read(&target_file).unwrap();
        assert_eq!(content, b"ABCD", "Should remove byte at position 2");

        // Verify: undo log removed
        assert!(!log_dir.join("0").exists(), "Undo log should be deleted");

        // Verify: redo log created (inverse: add X back)
        assert!(redo_dir.join("0").exists(), "Redo log should be created");

        let redo_content = fs::read_to_string(redo_dir.join("0")).unwrap();
        assert!(redo_content.contains("add"), "Redo should say 'add'");
        assert!(
            redo_content.contains("58"),
            "Redo should have byte 0x58 (X)"
        );

        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_single_byte_undo_add_creates_redo() {
        // Test: undo adds byte AND creates redo log to remove it again
        let test_dir = env::temp_dir().join("test_single_undo_add_redo");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();

        let target_file = test_dir.join("target.txt");
        fs::write(&target_file, b"ABCD").unwrap();
        let target_abs = target_file.canonicalize().unwrap();

        let log_dir = test_dir.join("logs");
        fs::create_dir_all(&log_dir).unwrap();
        let log_dir_abs = log_dir.canonicalize().unwrap();

        let redo_dir = test_dir.join("redo_logs");
        fs::create_dir_all(&redo_dir).unwrap();
        let redo_dir_abs = redo_dir.canonicalize().unwrap();

        // Create undo log: "add 0x58 at position 2"
        let log_entry = LogEntry::new(EditType::Add, 2, Some(0x58)).unwrap();
        fs::write(log_dir.join("0"), log_entry.to_file_format()).unwrap();

        // Execute undo
        button_undo_single_byte_with_redo_support(
            &target_abs,
            &log_dir_abs,
            true,
            Some(&redo_dir_abs),
        )
        .unwrap();

        // Verify: byte added
        let content = fs::read(&target_file).unwrap();
        assert_eq!(content, b"ABXCD", "Should add byte at position 2");

        // Verify: redo log created (inverse: remove)
        assert!(redo_dir.join("0").exists(), "Redo log should be created");
        let redo_content = fs::read_to_string(redo_dir.join("0")).unwrap();
        assert!(redo_content.contains("rmv"), "Redo should say 'rmv'");

        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_single_byte_undo_edit_creates_redo() {
        // Test: undo hex-edits byte AND creates redo log to edit back
        let test_dir = env::temp_dir().join("test_single_undo_edit_redo");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();

        let target_file = test_dir.join("target.txt");
        fs::write(&target_file, b"ABZD").unwrap(); // User changed 'C' to 'Z'
        let target_abs = target_file.canonicalize().unwrap();

        let log_dir = test_dir.join("logs");
        fs::create_dir_all(&log_dir).unwrap();
        let log_dir_abs = log_dir.canonicalize().unwrap();

        let redo_dir = test_dir.join("redo_logs");
        fs::create_dir_all(&redo_dir).unwrap();
        let redo_dir_abs = redo_dir.canonicalize().unwrap();

        // Create undo log: "edt 0x43 at position 2" (restore 'C')
        let log_entry = LogEntry::new(EditType::Edt, 2, Some(0x43)).unwrap();
        fs::write(log_dir.join("0"), log_entry.to_file_format()).unwrap();

        // Execute undo
        button_undo_single_byte_with_redo_support(
            &target_abs,
            &log_dir_abs,
            true,
            Some(&redo_dir_abs),
        )
        .unwrap();

        // Verify: byte restored to 'C'
        let content = fs::read(&target_file).unwrap();
        assert_eq!(content, b"ABCD", "Should restore original byte");

        // Verify: redo log created (inverse: edit back to Z)
        assert!(redo_dir.join("0").exists(), "Redo log should be created");
        let redo_content = fs::read_to_string(redo_dir.join("0")).unwrap();
        assert!(redo_content.contains("edt"), "Redo should say 'edt'");
        assert!(
            redo_content.contains("5A"),
            "Redo should have byte 0x5A (Z)"
        );

        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_single_byte_redo_no_redo_logs_created() {
        // Test: redo operations (is_undo_operation=false) don't create more redo logs
        let test_dir = env::temp_dir().join("test_single_redo_no_logs");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();

        let target_file = test_dir.join("target.txt");
        fs::write(&target_file, b"ABXCD").unwrap();
        let target_abs = target_file.canonicalize().unwrap();

        let redo_dir = test_dir.join("redo_logs");
        fs::create_dir_all(&redo_dir).unwrap();
        let redo_dir_abs = redo_dir.canonicalize().unwrap();

        // Create redo log: "rmv at position 2"
        let log_entry = LogEntry::new(EditType::Rmv, 2, None).unwrap();
        fs::write(redo_dir.join("0"), log_entry.to_file_format()).unwrap();

        // Execute REDO (is_undo_operation = false, no redo_dir provided)
        button_undo_single_byte_with_redo_support(
            &target_abs,
            &redo_dir_abs,
            false, // is_undo_operation = false (REDO mode)
            None,  // No redo directory for redo operations
        )
        .unwrap();

        // Verify: byte removed
        let content = fs::read(&target_file).unwrap();
        assert_eq!(content, b"ABCD", "Should remove byte");

        // Verify: original redo log removed
        assert!(!redo_dir.join("0").exists(), "Redo log should be consumed");

        // Verify: no new logs created in redo dir
        let entries: Vec<_> = fs::read_dir(&redo_dir_abs)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert_eq!(entries.len(), 0, "No new redo logs should be created");

        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_single_byte_undo_malformed_log_quarantined() {
        // Test: malformed log gets quarantined, redo not created
        let test_dir = env::temp_dir().join("test_single_undo_malformed");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();

        let target_file = test_dir.join("target.txt");
        fs::write(&target_file, b"ABCD").unwrap();
        let target_abs = target_file.canonicalize().unwrap();

        let log_dir = test_dir.join("logs");
        fs::create_dir_all(&log_dir).unwrap();
        let log_dir_abs = log_dir.canonicalize().unwrap();

        let redo_dir = test_dir.join("redo_logs");
        fs::create_dir_all(&redo_dir).unwrap();
        let redo_dir_abs = redo_dir.canonicalize().unwrap();

        // Create malformed log
        fs::write(log_dir.join("0"), "GARBAGE\n").unwrap();

        // Execute undo - should fail
        let result = button_undo_single_byte_with_redo_support(
            &target_abs,
            &log_dir_abs,
            true,
            Some(&redo_dir_abs),
        );

        assert!(result.is_err(), "Should fail with malformed log");

        // Verify: log quarantined (not in original location)
        assert!(!log_dir.join("0").exists(), "Log should be quarantined");

        // Verify: no redo log created
        assert!(
            !redo_dir.join("0").exists(),
            "No redo log for failed operation"
        );

        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_single_byte_undo_no_logs_error() {
        // Test: returns error when no logs exist
        let test_dir = env::temp_dir().join("test_single_undo_no_logs");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();

        let target_file = test_dir.join("target.txt");
        fs::write(&target_file, b"ABCD").unwrap();
        let target_abs = target_file.canonicalize().unwrap();

        let log_dir = test_dir.join("logs");
        fs::create_dir_all(&log_dir).unwrap();
        let log_dir_abs = log_dir.canonicalize().unwrap();

        // No redo dir needed for this test
        let result =
            button_undo_single_byte_with_redo_support(&target_abs, &log_dir_abs, true, None);

        assert!(result.is_err(), "Should fail with no logs");
        match result {
            Err(ButtonError::NoLogsFound { .. }) => {} // Expected
            _ => panic!("Should return NoLogsFound error"),
        }

        let _ = fs::remove_dir_all(&test_dir);
    }

    // ========================================================================
    // Tests for button_undo_multibyte_with_redo_support (ACTUAL function used)
    // ========================================================================

    #[test]
    fn test_multibyte_undo_remove_creates_redo() {
        // Test: undo removes 3-byte char AND creates redo logs
        let test_dir = env::temp_dir().join("test_multi_undo_remove_redo");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();

        let target_file = test_dir.join("target.txt");
        fs::write(&target_file, b"AB\xE9\x98\xBFCD").unwrap(); // Has '阿'
        let target_abs = target_file.canonicalize().unwrap();

        let log_dir = test_dir.join("logs");
        fs::create_dir_all(&log_dir).unwrap();
        let log_dir_abs = log_dir.canonicalize().unwrap();

        let redo_dir = test_dir.join("redo_logs");
        fs::create_dir_all(&redo_dir).unwrap();
        let redo_dir_abs = redo_dir.canonicalize().unwrap();

        // Create undo log set: 0.b, 0.a, 0 (all say "rmv at 2")
        fs::write(log_dir.join("0.b"), "rmv\n2\n").unwrap();
        fs::write(log_dir.join("0.a"), "rmv\n2\n").unwrap();
        fs::write(log_dir.join("0"), "rmv\n2\n").unwrap();

        // Execute undo
        button_undo_multibyte_with_redo_support(
            &target_abs,
            &log_dir_abs,
            true,
            Some(&redo_dir_abs),
        )
        .unwrap();

        // Verify: character removed
        let content = fs::read(&target_file).unwrap();
        assert_eq!(content, b"ABCD", "Should remove 3-byte character");

        // Verify: undo logs removed
        assert!(!log_dir.join("0.b").exists());
        assert!(!log_dir.join("0.a").exists());
        assert!(!log_dir.join("0").exists());

        // Verify: redo logs created (inverse: add bytes back)
        assert!(redo_dir.join("0.b").exists(), "Redo log 0.b created");
        assert!(redo_dir.join("0.a").exists(), "Redo log 0.a created");
        assert!(redo_dir.join("0").exists(), "Redo log 0 created");

        // Verify redo logs contain correct inverse (add E9, 98, BF)
        let redo_0 = fs::read_to_string(redo_dir.join("0")).unwrap();
        assert!(redo_0.contains("add"));
        assert!(redo_0.contains("E9")); // First byte

        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_multibyte_undo_add_creates_redo() {
        // Test: undo adds 3-byte char back AND creates redo logs to remove it
        let test_dir = env::temp_dir().join("test_multi_undo_add_redo");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();

        let target_file = test_dir.join("target.txt");
        fs::write(&target_file, b"ABCD").unwrap(); // Missing '阿'
        let target_abs = target_file.canonicalize().unwrap();

        let log_dir = test_dir.join("logs");
        fs::create_dir_all(&log_dir).unwrap();
        let log_dir_abs = log_dir.canonicalize().unwrap();

        let redo_dir = test_dir.join("redo_logs");
        fs::create_dir_all(&redo_dir).unwrap();
        let redo_dir_abs = redo_dir.canonicalize().unwrap();

        // Create undo log set: add BF, 98, E9 at position 2
        fs::write(log_dir.join("0.b"), "add\n2\nBF\n").unwrap();
        fs::write(log_dir.join("0.a"), "add\n2\n98\n").unwrap();
        fs::write(log_dir.join("0"), "add\n2\nE9\n").unwrap();

        // Execute undo
        button_undo_multibyte_with_redo_support(
            &target_abs,
            &log_dir_abs,
            true,
            Some(&redo_dir_abs),
        )
        .unwrap();

        // Verify: character added
        let content = fs::read(&target_file).unwrap();
        assert_eq!(content, b"AB\xE9\x98\xBFCD", "Should add 3-byte character");

        // Verify: redo logs created (inverse: remove)
        assert!(redo_dir.join("0.b").exists());
        assert!(redo_dir.join("0.a").exists());
        assert!(redo_dir.join("0").exists());

        let redo_0 = fs::read_to_string(redo_dir.join("0")).unwrap();
        assert!(redo_0.contains("rmv"), "Redo should say 'rmv'");

        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_multibyte_redo_no_redo_logs_created() {
        // Test: redo operations don't create more redo logs (prevents infinite chain)
        let test_dir = env::temp_dir().join("test_multi_redo_no_logs");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();

        let target_file = test_dir.join("target.txt");
        fs::write(&target_file, b"AB\xE9\x98\xBFCD").unwrap();
        let target_abs = target_file.canonicalize().unwrap();

        let redo_dir = test_dir.join("redo_logs");
        fs::create_dir_all(&redo_dir).unwrap();
        let redo_dir_abs = redo_dir.canonicalize().unwrap();

        // Create redo log set
        fs::write(redo_dir.join("0.b"), "rmv\n2\n").unwrap();
        fs::write(redo_dir.join("0.a"), "rmv\n2\n").unwrap();
        fs::write(redo_dir.join("0"), "rmv\n2\n").unwrap();

        // Execute REDO (is_undo_operation = false)
        button_undo_multibyte_with_redo_support(
            &target_abs,
            &redo_dir_abs,
            false, // REDO mode
            None,
        )
        .unwrap();

        // Verify: character removed
        let content = fs::read(&target_file).unwrap();
        assert_eq!(content, b"ABCD");

        // Verify: no new redo logs created
        let entries: Vec<_> = fs::read_dir(&redo_dir_abs)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert_eq!(
            entries.len(),
            0,
            "No new redo logs in redo mode (prevents infinite chain)"
        );

        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_multibyte_undo_incomplete_set_fails() {
        // Test: incomplete log set causes graceful failure, no redo created
        let test_dir = env::temp_dir().join("test_multi_undo_incomplete");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();

        let target_file = test_dir.join("target.txt");
        fs::write(&target_file, b"AB\xE9\x98\xBFCD").unwrap();
        let target_abs = target_file.canonicalize().unwrap();

        let log_dir = test_dir.join("logs");
        fs::create_dir_all(&log_dir).unwrap();
        let log_dir_abs = log_dir.canonicalize().unwrap();

        let redo_dir = test_dir.join("redo_logs");
        fs::create_dir_all(&redo_dir).unwrap();
        let redo_dir_abs = redo_dir.canonicalize().unwrap();

        // Create INCOMPLETE log set: missing 0.a
        fs::write(log_dir.join("0.b"), "rmv\n2\n").unwrap();
        // Missing 0.a!
        fs::write(log_dir.join("0"), "rmv\n2\n").unwrap();

        // Execute undo - should fail
        let result = button_undo_multibyte_with_redo_support(
            &target_abs,
            &log_dir_abs,
            true,
            Some(&redo_dir_abs),
        );

        assert!(result.is_err(), "Should fail with incomplete set");

        // Verify: no redo logs created for failed operation
        assert!(
            !redo_dir.join("0.b").exists(),
            "No redo for failed operation"
        );

        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_multibyte_undo_malformed_quarantines_all() {
        // Test: one malformed log causes entire set to be quarantined
        let test_dir = env::temp_dir().join("test_multi_undo_malformed");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();

        let target_file = test_dir.join("target.txt");
        fs::write(&target_file, b"AB\xE9\x98\xBFCD").unwrap();
        let target_abs = target_file.canonicalize().unwrap();

        let log_dir = test_dir.join("logs");
        fs::create_dir_all(&log_dir).unwrap();
        let log_dir_abs = log_dir.canonicalize().unwrap();

        let redo_dir = test_dir.join("redo_logs");
        fs::create_dir_all(&redo_dir).unwrap();
        let redo_dir_abs = redo_dir.canonicalize().unwrap();

        // Create log set with one malformed
        fs::write(log_dir.join("0.b"), "rmv\n2\n").unwrap();
        fs::write(log_dir.join("0.a"), "GARBAGE\n").unwrap(); // Malformed!
        fs::write(log_dir.join("0"), "rmv\n2\n").unwrap();

        // Execute undo - should fail
        let result = button_undo_multibyte_with_redo_support(
            &target_abs,
            &log_dir_abs,
            true,
            Some(&redo_dir_abs),
        );

        assert!(result.is_err(), "Should fail with malformed log");

        // Verify: entire set quarantined
        assert!(!log_dir.join("0.b").exists(), "Set should be quarantined");
        assert!(!log_dir.join("0.a").exists());
        assert!(!log_dir.join("0").exists());

        // Verify: no redo logs created
        assert!(!redo_dir.join("0.b").exists(), "No redo for failed op");

        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_multibyte_undo_2byte_character() {
        // Test: works correctly with 2-byte UTF-8 character
        let test_dir = env::temp_dir().join("test_multi_undo_2byte");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();

        let target_file = test_dir.join("target.txt");
        fs::write(&target_file, b"AB\xC2\xA9CD").unwrap(); // '©' at position 2
        let target_abs = target_file.canonicalize().unwrap();

        let log_dir = test_dir.join("logs");
        fs::create_dir_all(&log_dir).unwrap();
        let log_dir_abs = log_dir.canonicalize().unwrap();

        let redo_dir = test_dir.join("redo_logs");
        fs::create_dir_all(&redo_dir).unwrap();
        let redo_dir_abs = redo_dir.canonicalize().unwrap();

        // Create log set for 2-byte character: 0.a, 0
        fs::write(log_dir.join("0.a"), "rmv\n2\n").unwrap();
        fs::write(log_dir.join("0"), "rmv\n2\n").unwrap();

        // Execute undo
        button_undo_multibyte_with_redo_support(
            &target_abs,
            &log_dir_abs,
            true,
            Some(&redo_dir_abs),
        )
        .unwrap();

        // Verify: 2-byte character removed
        let content = fs::read(&target_file).unwrap();
        assert_eq!(content, b"ABCD", "Should remove 2-byte character");

        // Verify: redo logs created
        assert!(redo_dir.join("0.a").exists());
        assert!(redo_dir.join("0").exists());

        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_multibyte_undo_4byte_character() {
        // Test: works correctly with 4-byte UTF-8 character (emoji)
        let test_dir = env::temp_dir().join("test_multi_undo_4byte");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();

        let target_file = test_dir.join("target.txt");
        fs::write(&target_file, b"AB\xF0\x9F\x98\x80CD").unwrap(); // '😀'
        let target_abs = target_file.canonicalize().unwrap();

        let log_dir = test_dir.join("logs");
        fs::create_dir_all(&log_dir).unwrap();
        let log_dir_abs = log_dir.canonicalize().unwrap();

        let redo_dir = test_dir.join("redo_logs");
        fs::create_dir_all(&redo_dir).unwrap();
        let redo_dir_abs = redo_dir.canonicalize().unwrap();

        // Create log set for 4-byte character: 0.c, 0.b, 0.a, 0
        fs::write(log_dir.join("0.c"), "rmv\n2\n").unwrap();
        fs::write(log_dir.join("0.b"), "rmv\n2\n").unwrap();
        fs::write(log_dir.join("0.a"), "rmv\n2\n").unwrap();
        fs::write(log_dir.join("0"), "rmv\n2\n").unwrap();

        // Execute undo
        button_undo_multibyte_with_redo_support(
            &target_abs,
            &log_dir_abs,
            true,
            Some(&redo_dir_abs),
        )
        .unwrap();

        // Verify: 4-byte emoji removed
        let content = fs::read(&target_file).unwrap();
        assert_eq!(content, b"ABCD", "Should remove 4-byte emoji");

        // Verify: all 4 redo logs created
        assert!(redo_dir.join("0.c").exists());
        assert!(redo_dir.join("0.b").exists());
        assert!(redo_dir.join("0.a").exists());
        assert!(redo_dir.join("0").exists());

        let _ = fs::remove_dir_all(&test_dir);
    }

    // ========================================================================
    // Integration Tests: Complete Undo/Redo Workflow via Router Function
    // ========================================================================

    #[test]
    fn test_complete_undo_redo_workflow_single_byte() {
        // Test: Complete workflow through router function
        let test_dir = env::temp_dir().join("test_workflow_single");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();

        let target_file = test_dir.join("target.txt");
        fs::write(&target_file, b"ABXCD").unwrap();

        let undo_dir = test_dir.join("changelog_targettxt");
        let redo_dir = test_dir.join("changelog_redo_targettxt");

        // Create undo log
        fs::create_dir_all(&undo_dir).unwrap();
        fs::write(undo_dir.join("0"), "rmv\n2\n").unwrap();

        // UNDO via router (detects undo dir, creates redo)
        button_undo_redo_next_inverse_changelog_pop_lifo(&target_file, &undo_dir).unwrap();
        assert_eq!(fs::read(&target_file).unwrap(), b"ABCD", "Undo removes X");
        assert!(redo_dir.join("0").exists(), "Redo log created");

        // REDO via router (detects redo dir, no more redo logs)
        button_undo_redo_next_inverse_changelog_pop_lifo(&target_file, &redo_dir).unwrap();
        assert_eq!(fs::read(&target_file).unwrap(), b"ABXCD", "Redo restores X");

        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_complete_undo_redo_workflow_multibyte() {
        // Test: Complete workflow with multi-byte character
        let test_dir = env::temp_dir().join("test_workflow_multi");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();

        let target_file = test_dir.join("target.txt");
        fs::write(&target_file, b"AB\xE9\x98\xBFCD").unwrap(); // Has '阿'

        let undo_dir = test_dir.join("changelog_targettxt");
        let redo_dir = test_dir.join("changelog_redo_targettxt");

        // Create undo log set
        fs::create_dir_all(&undo_dir).unwrap();
        fs::write(undo_dir.join("0.b"), "rmv\n2\n").unwrap();
        fs::write(undo_dir.join("0.a"), "rmv\n2\n").unwrap();
        fs::write(undo_dir.join("0"), "rmv\n2\n").unwrap();

        // UNDO via router
        button_undo_redo_next_inverse_changelog_pop_lifo(&target_file, &undo_dir).unwrap();
        assert_eq!(fs::read(&target_file).unwrap(), b"ABCD", "Undo removes 阿");
        assert!(redo_dir.join("0.b").exists(), "Redo logs created");

        // REDO via router
        button_undo_redo_next_inverse_changelog_pop_lifo(&target_file, &redo_dir).unwrap();
        assert_eq!(
            fs::read(&target_file).unwrap(),
            b"AB\xE9\x98\xBFCD",
            "Redo restores 阿"
        );

        let _ = fs::remove_dir_all(&test_dir);
    }
}

// ============================================================================
// ADDITIONAL COMPREHENSIVE TESTS
// ============================================================================

#[cfg(test)]
mod additional_comprehensive_tests {
    use super::*;
    use std::env;

    // ========================================================================
    // TEST: Complete Editing Session Simulation
    // ========================================================================

    /// Tests a realistic editing session with mixed operations
    ///
    /// Simulates a user:
    /// 1. Types "Hello" (5 add operations)
    /// 2. Deletes one character (1 remove operation)
    /// 3. Adds a multi-byte emoji
    /// 4. Undoes everything step by step
    /// 5. Redoes some operations
    ///
    /// This tests LIFO ordering, mixed single/multi-byte, and undo/redo chains.
    #[test]
    fn test_realistic_editing_session() {
        let test_dir = env::temp_dir().join("test_editing_session");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();

        let target_file = test_dir.join("document.txt");
        fs::write(&target_file, b"").unwrap(); // Start with empty file

        let log_dir = test_dir.join("changelog_documenttxt");
        fs::create_dir_all(&log_dir).unwrap();

        println!("\n=== Realistic Editing Session Test ===");

        // Phase 1: User types "Hello" (5 characters)
        println!("\nPhase 1: User types 'Hello'");
        let chars = ['H', 'e', 'l', 'l', 'o'];
        for (i, ch) in chars.iter().enumerate() {
            // Simulate: user adds character
            let mut content = fs::read(&target_file).unwrap();
            content.push(*ch as u8);
            fs::write(&target_file, &content).unwrap();

            // Create log (log says "remove" to undo the add)
            button_make_character_action_changelog(
                &target_file,
                None,
                i as u128,
                EditType::Add,
                &log_dir,
            )
            .unwrap();

            println!("  Added '{}' at position {}", ch, i);
        }

        assert_eq!(fs::read_to_string(&target_file).unwrap(), "Hello");
        println!("  File now: 'Hello'");

        // Phase 2: User deletes last 'o'
        println!("\nPhase 2: User deletes last 'o'");
        fs::write(&target_file, b"Hell").unwrap();
        button_make_character_action_changelog(
            &target_file,
            Some('o'),
            4, // Position of deleted 'o'
            EditType::Rmv,
            &log_dir,
        )
        .unwrap();
        assert_eq!(fs::read_to_string(&target_file).unwrap(), "Hell");
        println!("  File now: 'Hell'");

        // Phase 3: User adds emoji '😀' (4-byte UTF-8)
        println!("\nPhase 3: User adds emoji '😀'");
        fs::write(&target_file, "Hell😀").unwrap();
        button_make_character_action_changelog(
            &target_file,
            None,
            4, // Position after "Hell"
            EditType::Add,
            &log_dir,
        )
        .unwrap();
        assert_eq!(fs::read_to_string(&target_file).unwrap(), "Hell😀");
        println!("  File now: 'Hell😀'");

        // Phase 4: Undo everything (LIFO order)
        println!("\nPhase 4: Undo operations (LIFO)");

        // Undo 1: Remove emoji
        button_undo_redo_next_inverse_changelog_pop_lifo(&target_file, &log_dir).unwrap();
        assert_eq!(fs::read_to_string(&target_file).unwrap(), "Hell");
        println!("  After undo 1: 'Hell' (emoji removed)");

        // Undo 2: Restore 'o'
        button_undo_redo_next_inverse_changelog_pop_lifo(&target_file, &log_dir).unwrap();
        assert_eq!(fs::read_to_string(&target_file).unwrap(), "Hello");
        println!("  After undo 2: 'Hello' ('o' restored)");

        // Undo 3-7: Remove "Hello" one by one
        for i in 0..5 {
            button_undo_redo_next_inverse_changelog_pop_lifo(&target_file, &log_dir).unwrap();
            let expected = ["Hell", "Hel", "He", "H", ""];
            assert_eq!(fs::read_to_string(&target_file).unwrap(), expected[i]);
            println!("  After undo {}: '{}'", i + 3, expected[i]);
        }

        // Phase 5: Redo some operations
        println!("\nPhase 5: Redo operations");
        let redo_dir = test_dir.join("changelog_redo_documenttxt");

        // Redo 1: Restore 'H'
        button_undo_redo_next_inverse_changelog_pop_lifo(&target_file, &redo_dir).unwrap();
        assert_eq!(fs::read_to_string(&target_file).unwrap(), "H");
        println!("  After redo 1: 'H'");

        // Redo 2: Restore 'e'
        button_undo_redo_next_inverse_changelog_pop_lifo(&target_file, &redo_dir).unwrap();
        assert_eq!(fs::read_to_string(&target_file).unwrap(), "He");
        println!("  After redo 2: 'He'");

        println!("\n✅ Realistic editing session test PASSED");
        let _ = fs::remove_dir_all(&test_dir);
    }

    // ========================================================================
    // TEST: Redo Cleared After Normal Edit
    // ========================================================================

    /// Tests that redo logs are cleared when user makes a new edit
    ///
    /// This is critical behavior: after undo, if user makes a new edit,
    /// the redo history becomes invalid and must be cleared.
    ///
    /// Sequence:
    /// 1. User adds 'A'
    /// 2. User undoes (now have redo log)
    /// 3. User adds 'B' (different edit)
    /// 4. Redo log should be cleared (can't redo 'A' anymore)
    #[test]
    fn test_redo_cleared_after_normal_edit() {
        let test_dir = env::temp_dir().join("test_redo_cleared");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();

        let target_file = test_dir.join("file.txt");
        fs::write(&target_file, b"").unwrap();

        let log_dir = test_dir.join("changelog_filetxt");
        let redo_dir = test_dir.join("changelog_redo_filetxt");

        println!("\n=== Redo Cleared After Normal Edit Test ===");

        // Step 1: User adds 'A'
        println!("\nStep 1: User adds 'A'");
        fs::write(&target_file, b"A").unwrap();
        button_make_character_action_changelog(&target_file, None, 0, EditType::Add, &log_dir)
            .unwrap();

        // Step 2: User undos (creates redo log)
        println!("Step 2: User undoes");
        button_undo_redo_next_inverse_changelog_pop_lifo(&target_file, &log_dir).unwrap();
        assert_eq!(fs::read_to_string(&target_file).unwrap(), "");

        // Verify redo log exists
        fs::create_dir_all(&redo_dir).unwrap();
        assert!(
            fs::read_dir(&redo_dir).unwrap().count() > 0,
            "Redo log should exist after undo"
        );
        println!("  Redo log created: can redo 'A'");

        // Step 3: User makes NEW edit (adds 'B')
        println!("Step 3: User makes new edit (adds 'B')");
        fs::write(&target_file, b"B").unwrap();
        button_make_character_action_changelog(&target_file, None, 0, EditType::Add, &log_dir)
            .unwrap();

        // Step 4: Clear redo logs (should happen automatically in real editor)
        println!("Step 4: Clearing redo logs (new edit invalidates redo history)");
        button_clear_all_redo_logs(&target_file).unwrap();

        // Verify redo logs are gone
        let redo_count = fs::read_dir(&redo_dir)
            .map(|entries| entries.count())
            .unwrap_or(0);
        assert_eq!(redo_count, 0, "Redo logs should be cleared after new edit");

        println!("  ✓ Redo logs cleared (can't redo 'A' anymore)");
        println!("\n✅ Redo cleared after normal edit test PASSED");

        let _ = fs::remove_dir_all(&test_dir);
    }

    // ========================================================================
    // TEST: "Cheap Trick" Button Stack with Complex Characters
    // ========================================================================

    /// Tests the "cheap trick" button stack behavior with mixed characters
    ///
    /// The cheap trick: when adding multi-byte chars, all log entries use
    /// the SAME position (first byte position). When undoing/redoing:
    /// - Each add at position N pushes previous bytes forward
    /// - Each remove at position N naturally shifts remaining bytes back
    ///
    /// This tests that the cheap trick works with:
    /// - ASCII followed by emoji
    /// - Multiple multi-byte characters in sequence
    /// - Proper reconstruction order
    #[test]
    fn test_cheap_trick_button_stack_complex() {
        let test_dir = env::temp_dir().join("test_cheap_trick");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();

        let target_file = test_dir.join("file.txt");
        let log_dir = test_dir.join("changelog_filetxt");

        println!("\n=== Cheap Trick Button Stack Test ===");

        // Setup: File contains "A😀B阿C" (ASCII + emoji + ASCII + CJK + ASCII)
        println!("\nSetup: File contains 'A😀B阿C'");
        let content = "A😀B阿C";
        fs::write(&target_file, content).unwrap();
        println!("  Byte structure:");
        println!("    'A'  : 1 byte  at position 0");
        println!("    '😀' : 4 bytes at positions 1-4");
        println!("    'B'  : 1 byte  at position 5");
        println!("    '阿' : 3 bytes at positions 6-8");
        println!("    'C'  : 1 byte  at position 9");

        // Create remove logs for entire file (user "added" all of it)
        println!("\nCreating remove logs (simulating user added all chars)");

        // Remove 'A' at 0
        button_remove_byte_make_log_file(&fs::canonicalize(&target_file).unwrap(), 0, &log_dir)
            .unwrap();

        // Remove '😀' at 1 (4 bytes, cheap trick: all use position 1)
        button_remove_multibyte_make_log_files(
            &fs::canonicalize(&target_file).unwrap(),
            1,
            4,
            &log_dir,
        )
        .unwrap();

        // Remove 'B' at 5
        button_remove_byte_make_log_file(&fs::canonicalize(&target_file).unwrap(), 5, &log_dir)
            .unwrap();

        // Remove '阿' at 6 (3 bytes, cheap trick: all use position 6)
        button_remove_multibyte_make_log_files(
            &fs::canonicalize(&target_file).unwrap(),
            6,
            3,
            &log_dir,
        )
        .unwrap();

        // Remove 'C' at 9
        button_remove_byte_make_log_file(&fs::canonicalize(&target_file).unwrap(), 9, &log_dir)
            .unwrap();

        // Test: Undo all (LIFO - removes from end to start)
        println!("\nUndoing all operations (LIFO - removes from end to start)");

        button_undo_redo_next_inverse_changelog_pop_lifo(&target_file, &log_dir).unwrap();
        assert_eq!(fs::read_to_string(&target_file).unwrap(), "A😀B阿");
        println!("  After undo 1: 'A😀B阿' (removed 'C')");

        button_undo_redo_next_inverse_changelog_pop_lifo(&target_file, &log_dir).unwrap();
        assert_eq!(fs::read_to_string(&target_file).unwrap(), "A😀B");
        println!("  After undo 2: 'A😀B' (removed '阿')");

        button_undo_redo_next_inverse_changelog_pop_lifo(&target_file, &log_dir).unwrap();
        assert_eq!(fs::read_to_string(&target_file).unwrap(), "A😀");
        println!("  After undo 3: 'A😀' (removed 'B')");

        button_undo_redo_next_inverse_changelog_pop_lifo(&target_file, &log_dir).unwrap();
        assert_eq!(fs::read_to_string(&target_file).unwrap(), "A");
        println!("  After undo 4: 'A' (removed '😀')");

        button_undo_redo_next_inverse_changelog_pop_lifo(&target_file, &log_dir).unwrap();
        assert_eq!(fs::read_to_string(&target_file).unwrap(), "");
        println!("  After undo 5: '' (removed 'A')");

        // Test: Redo all (restores in same order)
        println!("\nRedoing all operations (restores in same order)");
        let redo_dir = test_dir.join("changelog_redo_filetxt");

        button_undo_redo_next_inverse_changelog_pop_lifo(&target_file, &redo_dir).unwrap();
        assert_eq!(fs::read_to_string(&target_file).unwrap(), "A");
        println!("  After redo 1: 'A'");

        button_undo_redo_next_inverse_changelog_pop_lifo(&target_file, &redo_dir).unwrap();
        assert_eq!(fs::read_to_string(&target_file).unwrap(), "A😀");
        println!("  After redo 2: 'A😀'");

        button_undo_redo_next_inverse_changelog_pop_lifo(&target_file, &redo_dir).unwrap();
        assert_eq!(fs::read_to_string(&target_file).unwrap(), "A😀B");
        println!("  After redo 3: 'A😀B'");

        button_undo_redo_next_inverse_changelog_pop_lifo(&target_file, &redo_dir).unwrap();
        assert_eq!(fs::read_to_string(&target_file).unwrap(), "A😀B阿");
        println!("  After redo 4: 'A😀B阿'");

        button_undo_redo_next_inverse_changelog_pop_lifo(&target_file, &redo_dir).unwrap();
        assert_eq!(fs::read_to_string(&target_file).unwrap(), "A😀B阿C");
        println!("  After redo 5: 'A😀B阿C' (fully restored!)");

        println!("\n✅ Cheap trick button stack test PASSED");
        let _ = fs::remove_dir_all(&test_dir);
    }

    // ========================================================================
    // TEST: Log File Corruption Recovery
    // ========================================================================

    /// Tests that corrupted log files are quarantined and don't crash system
    ///
    /// Tests various corruption scenarios:
    /// 1. Missing required fields
    /// 2. Invalid hex bytes
    /// 3. Invalid position numbers
    /// 4. Truncated multi-byte log sets
    ///
    /// System should:
    /// - Detect corruption
    /// - Quarantine bad logs
    /// - Continue operating
    /// - Never crash
    #[test]
    fn test_log_corruption_recovery() {
        let test_dir = env::temp_dir().join("test_corruption");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();

        let target_file = test_dir.join("file.txt");
        fs::write(&target_file, b"ABC").unwrap();
        let target_abs = target_file.canonicalize().unwrap();

        let log_dir = test_dir.join("changelog_file");
        fs::create_dir_all(&log_dir).unwrap();
        let log_dir_abs = log_dir.canonicalize().unwrap();

        println!("\n=== Log Corruption Recovery Test ===");

        // Test 1: Missing position field
        println!("\nTest 1: Log missing position field");
        fs::write(log_dir.join("0"), "add\n").unwrap();

        let result = button_undo_redo_next_inverse_changelog_pop_lifo(&target_abs, &log_dir_abs);
        assert!(result.is_err(), "Should fail gracefully");
        assert!(
            !log_dir.join("0").exists(),
            "Corrupted log should be quarantined"
        );
        println!("  ✓ Corrupted log quarantined");

        // Test 2: Invalid hex byte
        println!("\nTest 2: Log with invalid hex byte");
        fs::write(log_dir.join("1"), "add\n5\nZZ\n").unwrap();

        let result = button_undo_redo_next_inverse_changelog_pop_lifo(&target_abs, &log_dir_abs);
        assert!(result.is_err(), "Should fail gracefully");
        assert!(
            !log_dir.join("1").exists(),
            "Corrupted log should be quarantined"
        );
        println!("  ✓ Invalid hex byte log quarantined");

        // Test 3: Invalid position (not a number)
        println!("\nTest 3: Log with invalid position");
        fs::write(log_dir.join("2"), "add\nNOTANUMBER\n41\n").unwrap();

        let result = button_undo_redo_next_inverse_changelog_pop_lifo(&target_abs, &log_dir_abs);
        assert!(result.is_err(), "Should fail gracefully");
        assert!(
            !log_dir.join("2").exists(),
            "Corrupted log should be quarantined"
        );
        println!("  ✓ Invalid position log quarantined");

        // Test 4: Incomplete multi-byte set (missing middle file)
        println!("\nTest 4: Incomplete multi-byte log set");
        fs::write(log_dir.join("3.b"), "rmv\n1\n").unwrap();
        // Missing 3.a!
        fs::write(log_dir.join("3"), "rmv\n1\n").unwrap();

        let result = button_undo_redo_next_inverse_changelog_pop_lifo(&target_abs, &log_dir_abs);
        assert!(result.is_err(), "Should fail gracefully");
        println!("  ✓ Incomplete set detected");

        // Test 5: Completely garbage data
        println!("\nTest 5: Log with garbage data");
        fs::write(log_dir.join("4"), "�����\x00\x01\x02GARBAGE!@#$%").unwrap();

        let result = button_undo_redo_next_inverse_changelog_pop_lifo(&target_abs, &log_dir_abs);
        assert!(result.is_err(), "Should fail gracefully");
        assert!(
            !log_dir.join("4").exists(),
            "Garbage log should be quarantined"
        );
        println!("  ✓ Garbage log quarantined");

        // Verify system still works with valid log
        println!("\nTest 6: System still works after handling corruptions");
        fs::write(log_dir.join("5"), "rmv\n1\n").unwrap();

        let result = button_undo_redo_next_inverse_changelog_pop_lifo(&target_abs, &log_dir_abs);
        assert!(result.is_ok(), "Should work with valid log");
        assert_eq!(fs::read_to_string(&target_file).unwrap(), "AC");
        println!("  ✓ System recovered, valid operation succeeded");

        println!("\n✅ Log corruption recovery test PASSED");
        let _ = fs::remove_dir_all(&test_dir);
    }

    // ========================================================================
    // TEST: Position Out of Bounds Handling
    // ========================================================================

    /// Tests that operations at invalid positions are handled safely
    ///
    /// Tests:
    /// 1. Position beyond file end (for remove/edit)
    /// 2. Position exactly at file end (valid for add, invalid for remove)
    /// 3. Position negative (u128 wrapping)
    /// 4. Very large position numbers
    ///
    /// System should:
    /// - Detect out of bounds
    /// - Return appropriate error
    /// - Not corrupt file
    /// - Not crash
    #[test]
    fn test_position_out_of_bounds() {
        let test_dir = env::temp_dir().join("test_out_of_bounds");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();

        let target_file = test_dir.join("file.txt");
        fs::write(&target_file, b"ABC").unwrap(); // 3 bytes (positions 0, 1, 2)
        let target_abs = target_file.canonicalize().unwrap();

        let log_dir = test_dir.join("changelog_file");
        fs::create_dir_all(&log_dir).unwrap();
        let log_dir_abs = log_dir.canonicalize().unwrap();

        println!("\n=== Position Out of Bounds Test ===");

        // Test 1: Remove at position beyond end
        println!("\nTest 1: Remove at position 10 (file size = 3)");
        fs::write(log_dir.join("0"), "rmv\n10\n").unwrap();

        let result = button_undo_redo_next_inverse_changelog_pop_lifo(&target_abs, &log_dir_abs);
        assert!(result.is_err(), "Should fail with out of bounds");
        assert_eq!(
            fs::read_to_string(&target_file).unwrap(),
            "ABC",
            "File unchanged"
        );
        println!("  ✓ Out of bounds detected, file unchanged");

        // Clean up
        let _ = fs::remove_file(log_dir.join("0"));

        // Test 2: Edit at position equal to file size
        println!("\nTest 2: Edit at position 3 (file size = 3)");
        fs::write(log_dir.join("1"), "edt\n3\n41\n").unwrap();

        let result = button_undo_redo_next_inverse_changelog_pop_lifo(&target_abs, &log_dir_abs);
        assert!(result.is_err(), "Should fail (position 3 is out of bounds)");
        assert_eq!(
            fs::read_to_string(&target_file).unwrap(),
            "ABC",
            "File unchanged"
        );
        println!("  ✓ Position at file size rejected for edit");

        let _ = fs::remove_file(log_dir.join("1"));

        // Test 3: Add at position equal to file size (should be valid)
        println!("\nTest 3: Add at position 3 (file size = 3, valid for append)");
        fs::write(log_dir.join("2"), "add\n3\n44\n").unwrap();

        let result = button_undo_redo_next_inverse_changelog_pop_lifo(&target_abs, &log_dir_abs);
        assert!(result.is_ok(), "Should succeed (valid append position)");
        assert_eq!(fs::read_to_string(&target_file).unwrap(), "ABCD");
        println!("  ✓ Add at file size succeeded (append)");

        // Test 4: Very large position
        println!("\nTest 4: Remove at position u128::MAX");
        fs::write(&target_file, b"ABC").unwrap(); // Reset
        fs::write(log_dir.join("3"), format!("rmv\n{}\n", u128::MAX)).unwrap();

        let result = button_undo_redo_next_inverse_changelog_pop_lifo(&target_abs, &log_dir_abs);
        assert!(result.is_err(), "Should fail with out of bounds");
        assert_eq!(
            fs::read_to_string(&target_file).unwrap(),
            "ABC",
            "File unchanged"
        );
        println!("  ✓ Very large position rejected");

        println!("\n✅ Position out of bounds test PASSED");
        let _ = fs::remove_dir_all(&test_dir);
    }

    // ========================================================================
    // TEST: Empty File Operations
    // ========================================================================

    /// Tests operations on empty files
    ///
    /// Edge cases:
    /// 1. Add to empty file (should work)
    /// 2. Remove from empty file (should fail gracefully)
    /// 3. Edit empty file (should fail gracefully)
    /// 4. Undo until empty, then redo
    #[test]
    fn test_empty_file_operations() {
        let test_dir = env::temp_dir().join("test_empty_file");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();

        let target_file = test_dir.join("file.txt");
        let log_dir = test_dir.join("changelog_filetxt");
        fs::create_dir_all(&log_dir).unwrap();

        println!("\n=== Empty File Operations Test ===");

        // Test 1: Add to empty file
        println!("\nTest 1: Add byte to empty file");
        fs::write(&target_file, b"").unwrap();
        fs::write(log_dir.join("0"), "add\n0\n41\n").unwrap();

        button_undo_redo_next_inverse_changelog_pop_lifo(&target_file, &log_dir).unwrap();
        assert_eq!(fs::read_to_string(&target_file).unwrap(), "A");
        println!("  ✓ Add to empty file succeeded");

        // Test 2: Remove from empty file
        println!("\nTest 2: Remove from empty file");
        fs::write(&target_file, b"").unwrap();
        fs::write(log_dir.join("1"), "rmv\n0\n").unwrap();

        let result = button_undo_redo_next_inverse_changelog_pop_lifo(&target_file, &log_dir);
        assert!(result.is_err(), "Should fail on empty file");
        println!("  ✓ Remove from empty file rejected");

        let _ = fs::remove_file(log_dir.join("1"));

        // Test 3: Edit empty file
        println!("\nTest 3: Edit empty file");
        fs::write(&target_file, b"").unwrap();
        fs::write(log_dir.join("2"), "edt\n0\n41\n").unwrap();

        let result = button_undo_redo_next_inverse_changelog_pop_lifo(&target_file, &log_dir);
        assert!(result.is_err(), "Should fail on empty file");
        println!("  ✓ Edit empty file rejected");

        let _ = fs::remove_file(log_dir.join("2"));

        // Test 4: Start with content, undo to empty, then redo
        println!("\nTest 4: Undo to empty, then redo back");
        fs::write(&target_file, b"A").unwrap();

        button_make_character_action_changelog(&target_file, None, 0, EditType::Add, &log_dir)
            .unwrap();

        // Undo to empty
        button_undo_redo_next_inverse_changelog_pop_lifo(&target_file, &log_dir).unwrap();
        assert_eq!(fs::read_to_string(&target_file).unwrap(), "");
        println!("  ✓ Undone to empty file");

        // Redo back
        let redo_dir = test_dir.join("changelog_redo_filetxt");
        button_undo_redo_next_inverse_changelog_pop_lifo(&target_file, &redo_dir).unwrap();
        assert_eq!(fs::read_to_string(&target_file).unwrap(), "A");
        println!("  ✓ Redone from empty file");

        println!("\n✅ Empty file operations test PASSED");
        let _ = fs::remove_dir_all(&test_dir);
    }

    // ========================================================================
    // TEST: Maximum Undo Chain Depth
    // ========================================================================

    /// Tests very long undo/redo chains
    ///
    /// Creates 100 operations and ensures:
    /// 1. All can be undone in correct LIFO order
    /// 2. All can be redone in correct order
    /// 3. Log numbering works correctly
    /// 4. No performance degradation
    #[test]
    fn test_maximum_undo_chain_depth() {
        let test_dir = env::temp_dir().join("test_max_chain");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();

        let target_file = test_dir.join("file.txt");
        fs::write(&target_file, b"").unwrap();

        let log_dir = test_dir.join("changelog_filetxt");

        println!("\n=== Maximum Undo Chain Depth Test ===");

        const OPERATION_COUNT: usize = 100;

        // Phase 1: Create 100 operations
        println!("\nPhase 1: Creating {} operations", OPERATION_COUNT);
        for i in 0..OPERATION_COUNT {
            let ch = ('A' as u8 + (i % 26) as u8) as char;

            // Add character
            let mut content = fs::read(&target_file).unwrap();
            content.push(ch as u8);
            fs::write(&target_file, &content).unwrap();

            // Create log
            button_make_character_action_changelog(
                &target_file,
                None,
                i as u128,
                EditType::Add,
                &log_dir,
            )
            .unwrap();

            if (i + 1) % 20 == 0 {
                println!("  Created {} operations...", i + 1);
            }
        }

        let final_content = fs::read_to_string(&target_file).unwrap();
        assert_eq!(final_content.len(), OPERATION_COUNT);
        println!("  ✓ All {} operations created", OPERATION_COUNT);

        // Phase 2: Undo all operations
        println!("\nPhase 2: Undoing all {} operations", OPERATION_COUNT);
        for i in 0..OPERATION_COUNT {
            button_undo_redo_next_inverse_changelog_pop_lifo(&target_file, &log_dir).unwrap();

            if (i + 1) % 20 == 0 {
                println!("  Undone {} operations...", i + 1);
            }
        }

        assert_eq!(fs::read_to_string(&target_file).unwrap(), "");
        println!("  ✓ All operations undone (file empty)");

        // Phase 3: Redo all operations
        println!("\nPhase 3: Redoing all {} operations", OPERATION_COUNT);
        let redo_dir = test_dir.join("changelog_redo_filetxt");

        for i in 0..OPERATION_COUNT {
            button_undo_redo_next_inverse_changelog_pop_lifo(&target_file, &redo_dir).unwrap();

            if (i + 1) % 20 == 0 {
                println!("  Redone {} operations...", i + 1);
            }
        }

        let restored_content = fs::read_to_string(&target_file).unwrap();
        assert_eq!(restored_content, final_content);
        println!("  ✓ All operations redone (file restored)");

        println!(
            "\n✅ Maximum undo chain depth test PASSED ({} ops)",
            OPERATION_COUNT
        );
        let _ = fs::remove_dir_all(&test_dir);
    }
}

// ===================================
// Sample main code, e.g. for testning
// ===================================

/*
// main.rs for buttons_reversible_edit_changelog_module

mod buttons_reversible_edit_changelog_module;
use buttons_reversible_edit_changelog_module::{
    EditType, button_add_byte_make_log_file, button_clear_all_redo_logs,
    button_hexeditinplace_byte_make_log_file, button_make_character_action_changelog,
    button_make_hexedit_changelog, button_remove_byte_make_log_file,
    button_remove_multibyte_make_log_files, button_undo_redo_next_inverse_changelog_pop_lifo,
    get_undo_changelog_directory_path,
};
use std::fs;

fn main() -> std::io::Result<()> {
    println!("=============================================================");
    println!("BUTTON UNDO/REDO SYSTEM - COMPREHENSIVE TEST");
    println!("=============================================================\n");

    // Get current directory
    let test_dir = std::env::current_dir()?;

    // =========================================================================
    // TEST 1: REMOVE OPERATION (User added 'a', log says remove it)
    // =========================================================================
    println!("─────────────────────────────────────────────────────────────");
    println!("TEST 1: REMOVE OPERATION");
    println!("─────────────────────────────────────────────────────────────");

    let remove_test_file = test_dir.join("remove_test.txt");

    // Setup: Create file with 'a' (simulating user added it)
    println!("1. Setup: Creating file with 'a' (user just added it)");
    fs::write(&remove_test_file, b"a")?;
    println!(
        "   File contents: {:?}",
        fs::read_to_string(&remove_test_file)?
    );

    // Create changelog: rmv at position 0
    println!("2. Creating changelog: RMV at position 0");
    let log_dir_remove = test_dir.join("changelog_remove_testtxt");
    button_remove_byte_make_log_file(&fs::canonicalize(&remove_test_file)?, 0, &log_dir_remove)
        .expect("Failed to create remove log");
    println!("   ✓ Changelog created in: {}", log_dir_remove.display());

    // Execute undo (should remove 'a', leaving empty file)
    println!("3. Executing UNDO (should remove 'a')");
    button_undo_redo_next_inverse_changelog_pop_lifo(&remove_test_file, &log_dir_remove)
        .expect("Failed to undo remove");
    let result = fs::read_to_string(&remove_test_file)?;
    println!(
        "   File after undo: {:?} (length: {})",
        result,
        result.len()
    );
    assert_eq!(result, "", "TEST 1 FAILED: File should be empty");
    println!("   ✅ TEST 1 PASSED: File is now empty\n");

    // Test REDO functionality
    println!("4. Testing REDO (should restore 'a')");
    let redo_dir_remove = test_dir.join("changelog_redo_remove_testtxt");
    button_undo_redo_next_inverse_changelog_pop_lifo(&remove_test_file, &redo_dir_remove)
        .expect("Failed to redo");
    let result = fs::read_to_string(&remove_test_file)?;
    println!("   File after redo: {:?}", result);
    assert_eq!(result, "a", "TEST 1 REDO FAILED: File should contain 'a'");
    println!("   ✅ TEST 1 REDO PASSED: 'a' restored\n");

    // Cleanup
    let _ = fs::remove_file(&remove_test_file);
    let _ = fs::remove_dir_all(&log_dir_remove);
    let _ = fs::remove_dir_all(&redo_dir_remove);

    // =========================================================================
    // TEST 2: HEX EDIT OPERATION (User changed 'a' to 'b', log says change back)
    // =========================================================================
    println!("─────────────────────────────────────────────────────────────");
    println!("TEST 2: HEX EDIT OPERATION");
    println!("─────────────────────────────────────────────────────────────");

    let hexedit_test_file = test_dir.join("hex_edit_test.txt");

    // Setup: Create file with 'b' (simulating user hex-edited 'a' to 'b')
    println!("1. Setup: Creating file with 'b' (user hex-edited 'a'→'b')");
    fs::write(&hexedit_test_file, b"b")?;
    println!(
        "   File contents: {:?}",
        fs::read_to_string(&hexedit_test_file)?
    );

    // Create changelog: edt 61 (hex for 'a') at position 0
    println!("2. Creating changelog: EDT 0x61 ('a') at position 0");
    let log_dir_hexedit = test_dir.join("changelog_hex_edit_testtxt");
    button_hexeditinplace_byte_make_log_file(
        &fs::canonicalize(&hexedit_test_file)?,
        0,
        0x61, // Original value 'a'
        &log_dir_hexedit,
    )
    .expect("Failed to create hex edit log");
    println!("   ✓ Changelog created in: {}", log_dir_hexedit.display());

    // Execute undo (should change 'b' back to 'a')
    println!("3. Executing UNDO (should change 'b' back to 'a')");
    button_undo_redo_next_inverse_changelog_pop_lifo(&hexedit_test_file, &log_dir_hexedit)
        .expect("Failed to undo hex edit");
    let result = fs::read_to_string(&hexedit_test_file)?;
    println!("   File after undo: {:?}", result);
    assert_eq!(result, "a", "TEST 2 FAILED: File should contain 'a'");
    println!("   ✅ TEST 2 PASSED: 'b' changed back to 'a'\n");

    // Test REDO functionality
    println!("4. Testing REDO (should change back to 'b')");
    let redo_dir_hexedit = test_dir.join("changelog_redo_hex_edit_testtxt");
    button_undo_redo_next_inverse_changelog_pop_lifo(&hexedit_test_file, &redo_dir_hexedit)
        .expect("Failed to redo hex edit");
    let result = fs::read_to_string(&hexedit_test_file)?;
    println!("   File after redo: {:?}", result);
    assert_eq!(result, "b", "TEST 2 REDO FAILED: File should contain 'b'");
    println!("   ✅ TEST 2 REDO PASSED: 'a' changed back to 'b'\n");

    // Cleanup
    let _ = fs::remove_file(&hexedit_test_file);
    let _ = fs::remove_dir_all(&log_dir_hexedit);
    let _ = fs::remove_dir_all(&redo_dir_hexedit);

    // =========================================================================
    // TEST 3: ADD OPERATION (User removed 'a', log says add it back)
    // =========================================================================
    println!("─────────────────────────────────────────────────────────────");
    println!("TEST 3: ADD OPERATION");
    println!("─────────────────────────────────────────────────────────────");

    let add_test_file = test_dir.join("add_test.txt");

    // Setup: Create empty file (simulating user removed 'a')
    println!("1. Setup: Creating empty file (user just removed 'a')");
    fs::write(&add_test_file, b"")?;
    let content = fs::read_to_string(&add_test_file)?;
    println!(
        "   File contents: {:?} (length: {})",
        content,
        content.len()
    );

    // Create changelog: add 61 ('a') at position 0
    println!("2. Creating changelog: ADD 0x61 ('a') at position 0");
    let log_dir_add = test_dir.join("changelog_add_testtxt");
    button_add_byte_make_log_file(
        &fs::canonicalize(&add_test_file)?,
        0,
        0x61, // 'a'
        &log_dir_add,
    )
    .expect("Failed to create add log");
    println!("   ✓ Changelog created in: {}", log_dir_add.display());

    // Execute undo (should add 'a' back)
    println!("3. Executing UNDO (should add 'a' back)");
    button_undo_redo_next_inverse_changelog_pop_lifo(&add_test_file, &log_dir_add)
        .expect("Failed to undo add");
    let result = fs::read_to_string(&add_test_file)?;
    println!("   File after undo: {:?}", result);
    assert_eq!(result, "a", "TEST 3 FAILED: File should contain 'a'");
    println!("   ✅ TEST 3 PASSED: 'a' added back\n");

    // Test REDO functionality
    println!("4. Testing REDO (should remove 'a' again)");
    let redo_dir_add = test_dir.join("changelog_redo_add_testtxt");
    button_undo_redo_next_inverse_changelog_pop_lifo(&add_test_file, &redo_dir_add)
        .expect("Failed to redo add");
    let result = fs::read_to_string(&add_test_file)?;
    println!(
        "   File after redo: {:?} (length: {})",
        result,
        result.len()
    );
    assert_eq!(result, "", "TEST 3 REDO FAILED: File should be empty");
    println!("   ✅ TEST 3 REDO PASSED: 'a' removed again\n");

    // Cleanup
    let _ = fs::remove_file(&add_test_file);
    let _ = fs::remove_dir_all(&log_dir_add);
    let _ = fs::remove_dir_all(&redo_dir_add);

    // =========================================================================
    // TEST 4: MULTI-BYTE CHARACTER (UTF-8)
    // =========================================================================
    println!("─────────────────────────────────────────────────────────────");
    println!("BONUS TEST: MULTI-BYTE CHARACTER (UTF-8 '阿')");
    println!("─────────────────────────────────────────────────────────────");

    let multibyte_test_file = test_dir.join("multibyte_test.txt");

    // Setup: Create file with '阿' (3-byte UTF-8 character)
    println!("1. Setup: Creating file with '阿' (user just added it)");
    fs::write(&multibyte_test_file, "阿")?;
    println!(
        "   File contents: {:?}",
        fs::read_to_string(&multibyte_test_file)?
    );

    // Create changelog: rmv at position 0 (3 log files)
    println!("2. Creating changelog: RMV (multi-byte) at position 0");
    let log_dir_multibyte = test_dir.join("changelog_multibyte_testtxt");
    button_remove_multibyte_make_log_files(
        &fs::canonicalize(&multibyte_test_file)?,
        0,
        3, // 3 bytes in '阿'
        &log_dir_multibyte,
    )
    .expect("Failed to create multibyte remove log");
    println!("   ✓ Changelog created in: {}", log_dir_multibyte.display());

    // Execute undo (should remove '阿')
    println!("3. Executing UNDO (should remove '阿')");
    button_undo_redo_next_inverse_changelog_pop_lifo(&multibyte_test_file, &log_dir_multibyte)
        .expect("Failed to undo multibyte remove");
    let result = fs::read_to_string(&multibyte_test_file)?;
    println!(
        "   File after undo: {:?} (length: {})",
        result,
        result.len()
    );
    assert_eq!(result, "", "MULTIBYTE TEST FAILED: File should be empty");
    println!("   ✅ MULTIBYTE TEST PASSED: '阿' removed\n");

    // Test REDO functionality
    println!("4. Testing REDO (should restore '阿')");
    let redo_dir_multibyte = test_dir.join("changelog_redo_multibyte_testtxt");
    button_undo_redo_next_inverse_changelog_pop_lifo(&multibyte_test_file, &redo_dir_multibyte)
        .expect("Failed to redo multibyte");
    let result = fs::read_to_string(&multibyte_test_file)?;
    println!("   File after redo: {:?}", result);
    assert_eq!(
        result, "阿",
        "MULTIBYTE REDO FAILED: File should contain '阿'"
    );
    println!("   ✅ MULTIBYTE REDO PASSED: '阿' restored\n");

    // =========================================================================
    // NEW TEST 5: HIGH-LEVEL API - button_make_character_action_changelog()
    // =========================================================================
    println!("─────────────────────────────────────────────────────────────");
    println!("TEST 5: HIGH-LEVEL API - Character Action Changelog");
    println!("─────────────────────────────────────────────────────────────");

    let test5_file = test_dir.join("test5_character.txt");

    // Test 5a: User ADDS single-byte character
    println!("5a. User adds 'X' at position 2");
    fs::write(&test5_file, b"AB")?;

    // Manually add 'X' to simulate user action
    fs::write(&test5_file, b"ABX")?;
    println!("   ✓ Character add log created");

    // Simulate: user adds 'X', log should say "remove"
    let log_dir_5a = test_dir.join("changelog_test5_charactertxt");
    button_make_character_action_changelog(
        &test5_file,
        None, // Don't need character for Add
        2,
        EditType::Add,
        &log_dir_5a,
    )
    .expect("Failed to create character add log");

    // Undo should remove 'X'
    button_undo_redo_next_inverse_changelog_pop_lifo(&test5_file, &log_dir_5a)
        .expect("Failed to undo character add");

    let result = fs::read_to_string(&test5_file)?;
    assert_eq!(result, "AB", "TEST 5a FAILED: X should be removed");
    println!("   ✅ TEST 5a PASSED: Character add undone\n");

    // ===========================================
    // Test 5b: User REMOVES single-byte character
    println!("5b. User removes 'B' at position 1");
    fs::write(&test5_file, b"AB")?;

    // Simulate: user removes 'B', log should say "add B"
    button_make_character_action_changelog(
        &test5_file,
        Some('B'), // Need character to restore
        1,
        EditType::Rmv,
        &log_dir_5a,
    )
    .expect("Failed to create character remove log");

    println!("   ✓ Character remove log created");

    // Manually remove 'B' to simulate user action
    fs::write(&test5_file, b"A")?;

    // Undo should restore 'B'
    button_undo_redo_next_inverse_changelog_pop_lifo(&test5_file, &log_dir_5a)
        .expect("Failed to undo character remove");

    let result = fs::read_to_string(&test5_file)?;
    assert_eq!(result, "AB", "TEST 5b FAILED: B should be restored");
    println!("   ✅ TEST 5b PASSED: Character remove undone\n");

    // =======================================
    // Test 5c: User ADDS multi-byte character
    println!("5c. User adds '阿' at position 2");
    fs::write(&test5_file, b"AB")?;

    // Manually add '阿' to simulate user action
    fs::write(&test5_file, "AB阿")?;

    // Simulate: user adds '阿', log should say "remove" (3 times)
    button_make_character_action_changelog(&test5_file, None, 2, EditType::Add, &log_dir_5a)
        .expect("Failed to create multi-byte add log");
    println!("   ✓ Multi-byte character add log created");

    // Undo should remove '阿'
    button_undo_redo_next_inverse_changelog_pop_lifo(&test5_file, &log_dir_5a)
        .expect("Failed to undo multi-byte add");

    let result = fs::read_to_string(&test5_file)?;
    assert_eq!(result, "AB", "TEST 5c FAILED: 阿 should be removed");
    println!("   ✅ TEST 5c PASSED: Multi-byte character add undone\n");

    // ==========================================
    // Test 5d: User REMOVES multi-byte character
    /*
     * Note: only remove-action (shoud, at least)
     * validate the target position, so there is
     * no sequence issue for add-action
     */
    println!("5d. User removes '阿' at position 2");
    fs::write(&test5_file, "AB阿")?;

    // Simulate: user removes '阿', log should say "add 阿"
    button_make_character_action_changelog(&test5_file, Some('阿'), 2, EditType::Rmv, &log_dir_5a)
        .expect("Failed to create multi-byte remove log");

    // HERE, AFTER LOG, HOW IS LOG TESTING THE POSITION?
    // Manually remove '阿' to simulate user action
    fs::write(&test5_file, b"AB")?;

    println!("   ✓ Multi-byte character remove log created");

    // Undo should restore '阿'
    button_undo_redo_next_inverse_changelog_pop_lifo(&test5_file, &log_dir_5a)
        .expect("Failed to undo multi-byte remove");

    let result = fs::read_to_string(&test5_file)?;
    assert_eq!(result, "AB阿", "TEST 5d FAILED: 阿 should be restored");
    println!("   ✅ TEST 5d PASSED: Multi-byte character remove undone\n");

    // Cleanup
    let _ = fs::remove_file(&test5_file);
    let _ = fs::remove_dir_all(&log_dir_5a);

    // =========================================================================
    // NEW TEST 6: HIGH-LEVEL API - button_make_hexedit_changelog()
    // =========================================================================
    println!("─────────────────────────────────────────────────────────────");
    println!("TEST 6: HIGH-LEVEL API - Hex Edit Changelog");
    println!("─────────────────────────────────────────────────────────────");

    let test6_file = test_dir.join("test6_hexedit.txt");

    println!("6. User hex-edits position 1: 'B' (0x42) → 'Z' (0x5A)");
    fs::write(&test6_file, b"ABC")?;

    // Log original value before user's hex-edit
    let log_dir_6 = test_dir.join("changelog_test6_hexedittxt");
    button_make_hexedit_changelog(
        &test6_file,
        1,
        0x42, // Original 'B'
        &log_dir_6,
    )
    .expect("Failed to create hex-edit log");

    println!("   ✓ Hex-edit log created");

    // Manually hex-edit to simulate user action
    fs::write(&test6_file, b"AZC")?;

    // Undo should restore 'B'
    button_undo_redo_next_inverse_changelog_pop_lifo(&test6_file, &log_dir_6)
        .expect("Failed to undo hex-edit");

    let result = fs::read_to_string(&test6_file)?;
    assert_eq!(result, "ABC", "TEST 6 FAILED: B should be restored");
    println!("   ✅ TEST 6 PASSED: Hex-edit undone\n");

    // Test redo
    let redo_dir_6 = test_dir.join("changelog_redo_test6_hexedittxt");
    button_undo_redo_next_inverse_changelog_pop_lifo(&test6_file, &redo_dir_6)
        .expect("Failed to redo hex-edit");

    let result = fs::read_to_string(&test6_file)?;
    assert_eq!(result, "AZC", "TEST 6 REDO FAILED: Z should be restored");
    println!("   ✅ TEST 6 REDO PASSED: Hex-edit redone\n");

    // Cleanup
    let _ = fs::remove_file(&test6_file);
    let _ = fs::remove_dir_all(&log_dir_6);
    let _ = fs::remove_dir_all(&redo_dir_6);

    // =========================================================================
    // NEW TEST 7: HIGH-LEVEL API - get_undo_changelog_directory_path()
    // =========================================================================
    println!("─────────────────────────────────────────────────────────────");
    println!("TEST 7: HIGH-LEVEL API - Get Changelog Directory Path");
    println!("─────────────────────────────────────────────────────────────");

    let test7_file = test_dir.join("myfile.txt");
    fs::write(&test7_file, b"test")?;

    let log_dir =
        get_undo_changelog_directory_path(&test7_file).expect("Failed to get changelog directory path");

    println!("7. Changelog directory path: {}", log_dir.display());

    // Verify naming convention
    let dir_name = log_dir.file_name().unwrap().to_string_lossy();
    assert!(
        dir_name.starts_with("changelog_"),
        "TEST 7 FAILED: Directory should start with 'changelog_'"
    );
    assert!(
        dir_name.contains("myfile"),
        "TEST 7 FAILED: Directory should contain filename"
    );

    println!("   ✅ TEST 7 PASSED: Directory path correct\n");

    // Cleanup
    let _ = fs::remove_file(&test7_file);

    // =========================================================================
    // NEW TEST 8: HIGH-LEVEL API - button_clear_all_redo_logs()
    // =========================================================================
    println!("─────────────────────────────────────────────────────────────");
    println!("TEST 8: HIGH-LEVEL API - Clear All Redo Logs");
    println!("─────────────────────────────────────────────────────────────");

    let test8_file = test_dir.join("test8_clear.txt");
    fs::write(&test8_file, b"A")?;

    // Create some redo logs manually
    let redo_dir_8 = test_dir.join("changelog_redo_test8_cleartxt");
    fs::create_dir_all(&redo_dir_8)?;
    fs::write(redo_dir_8.join("0"), "rmv\n0\n")?;
    fs::write(redo_dir_8.join("1"), "rmv\n1\n")?;
    fs::write(redo_dir_8.join("2"), "rmv\n2\n")?;

    println!("8. Created 3 redo log files");

    // Verify they exist
    assert!(redo_dir_8.join("0").exists());
    assert!(redo_dir_8.join("1").exists());
    assert!(redo_dir_8.join("2").exists());

    // Clear redo logs
    button_clear_all_redo_logs(&test8_file).expect("Failed to clear redo logs");

    println!("   Called button_clear_all_redo_logs()");

    // Verify they're gone
    assert!(
        !redo_dir_8.join("0").exists(),
        "TEST 8 FAILED: Redo log 0 should be removed"
    );
    assert!(
        !redo_dir_8.join("1").exists(),
        "TEST 8 FAILED: Redo log 1 should be removed"
    );
    assert!(
        !redo_dir_8.join("2").exists(),
        "TEST 8 FAILED: Redo log 2 should be removed"
    );

    println!("   ✅ TEST 8 PASSED: All redo logs cleared\n");

    // Cleanup
    let _ = fs::remove_file(&test8_file);
    let _ = fs::remove_dir_all(&redo_dir_8);

    // =========================================================================
    // FINAL SUMMARY
    // =========================================================================
    println!("=============================================================");
    println!("✅ ALL TESTS PASSED!");
    println!("=============================================================");
    println!("✓ Test 1: Remove operation (undo + redo)");
    println!("✓ Test 2: Hex edit operation (undo + redo)");
    println!("✓ Test 3: Add operation (undo + redo)");
    println!("✓ Test 4: Multi-byte UTF-8 character (undo + redo)");
    println!("✓ Test 5: HIGH-LEVEL API - Character action changelog");
    println!("✓ Test 6: HIGH-LEVEL API - Hex edit changelog");
    println!("✓ Test 7: HIGH-LEVEL API - Get changelog directory path");
    println!("✓ Test 8: HIGH-LEVEL API - Clear all redo logs");
    println!("=============================================================\n");

    // // =========================================================================
    // // Manual Tests
    // // =========================================================================
    // println!("─────────────────────────────────────────────────────────────");
    // println!("Manual Tests");
    // println!("─────────────────────────────────────────────────────────────");

    // let manual_add_testfile = test_dir.join("manual_a_test.txt");

    // // Setup: Create empty file (simulating user removed 'a')
    // println!("1. Assuming you have an empty manual_a_test.txt, will add: a");

    // let content = fs::read_to_string(&manual_add_testfile)?;
    // println!(
    //     "   File contents: {:?} (length: {})",
    //     content,
    //     content.len()
    // );

    // // Create changelog: add 61 ('a') at position 0
    // println!("Creating changelog: ADD 0x61 ('a') at position 0");
    // let log_dir_manual_test_add = test_dir.join("manual_a_testtxt");
    // button_add_byte_make_log_file(
    //     &fs::canonicalize(&manual_add_testfile)?,
    //     0,
    //     0x61, // 'a'
    //     &log_dir_manual_test_add,
    // )
    // .expect("Failed to create add log");
    // println!(
    //     "   ✓ Changelog created in: {}",
    //     log_dir_manual_test_add.display()
    // );

    // // Execute undo (should add 'a' back)
    // println!("Executing add-operation (should add 'a')");
    // button_undo_redo_next_inverse_changelog_pop_lifo(&manual_add_testfile, &log_dir_manual_test_add)
    //     .expect("Failed to undo add");
    // let result = fs::read_to_string(&manual_add_testfile)?;
    // println!("   File after undo: {:?}", result);
    // assert_eq!(result, "a", "TEST FAILED: File should contain 'a'");
    // println!("   ✅ TEST PASSED: 'a' added\n");

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

    Ok(())
}

*/
