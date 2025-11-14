// main.rs for buttons_reversible_edit_changelog_module

mod buttons_reversible_edit_changelog_module;
use buttons_reversible_edit_changelog_module::{
    EditType, button_add_byte_make_log_file, button_base_clear_all_redo_logs,
    button_hexeditinplace_byte_make_log_file,
    button_make_changelog_from_user_character_action_level, button_make_hexedit_in_place_changelog,
    button_remove_byte_make_log_file, button_remove_multibyte_make_log_files,
    button_safe_clear_all_redo_logs, button_undo_redo_next_inverse_changelog_pop_lifo,
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
    // NEW TEST 5: HIGH-LEVEL API - button_make_changeloge_from_user_character_action_level()
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
    button_make_changelog_from_user_character_action_level(
        &test5_file,
        None, // Don't need character for Add
        None,
        2,
        EditType::AddCharacter,
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

    /*
    pub fn button_make_changelog_from_user_character_action_level(
        target_file: &Path,
        character: Option<char>,
        byte_value: Option<u8>, // TODO
        position: u128,
        edit_type: EditType,
        log_directory_path: &Path,
    ) -> ButtonResult<()> {
    */

    // Simulate: user removes 'B', log should say "add B"
    button_make_changelog_from_user_character_action_level(
        &test5_file,
        Some('B'), // Need character to restore
        None,
        1,
        EditType::RmvCharacter,
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
    button_make_changelog_from_user_character_action_level(
        &test5_file,
        None,
        None,
        2,
        EditType::AddCharacter,
        &log_dir_5a,
    )
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
    button_make_changelog_from_user_character_action_level(
        &test5_file,
        Some('阿'),
        None,
        2,
        EditType::RmvCharacter,
        &log_dir_5a,
    )
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
    button_make_hexedit_in_place_changelog(
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

    let log_dir = get_undo_changelog_directory_path(&test7_file)
        .expect("Failed to get changelog directory path");

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
    button_safe_clear_all_redo_logs(&test8_file).expect("Failed to clear redo logs");

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
    _ = button_base_clear_all_redo_logs(&manual_test_file);
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
