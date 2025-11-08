//! Utility functions for actor operations

/// Get key code and Windows virtual key code for a key
pub fn get_key_info(key: &str) -> (String, Option<u32>) {
    // Key mapping: (code, windowsVirtualKeyCode)
    let key_map: std::collections::HashMap<&str, (&str, u32)> = [
        // Navigation keys
        ("Backspace", ("Backspace", 8)),
        ("Tab", ("Tab", 9)),
        ("Enter", ("Enter", 13)),
        ("Escape", ("Escape", 27)),
        ("Space", ("Space", 32)),
        (" ", ("Space", 32)),
        ("PageUp", ("PageUp", 33)),
        ("PageDown", ("PageDown", 34)),
        ("End", ("End", 35)),
        ("Home", ("Home", 36)),
        ("ArrowLeft", ("ArrowLeft", 37)),
        ("ArrowUp", ("ArrowUp", 38)),
        ("ArrowRight", ("ArrowRight", 39)),
        ("ArrowDown", ("ArrowDown", 40)),
        ("Insert", ("Insert", 45)),
        ("Delete", ("Delete", 46)),
        // Modifier keys
        ("Shift", ("ShiftLeft", 16)),
        ("ShiftLeft", ("ShiftLeft", 16)),
        ("ShiftRight", ("ShiftRight", 16)),
        ("Control", ("ControlLeft", 17)),
        ("ControlLeft", ("ControlLeft", 17)),
        ("ControlRight", ("ControlRight", 17)),
        ("Alt", ("AltLeft", 18)),
        ("AltLeft", ("AltLeft", 18)),
        ("AltRight", ("AltRight", 18)),
        ("Meta", ("MetaLeft", 91)),
        ("MetaLeft", ("MetaLeft", 91)),
        ("MetaRight", ("MetaRight", 92)),
    ]
    .iter()
    .cloned()
    .collect();

    if let Some((code, vk_code)) = key_map.get(key) {
        return (code.to_string(), Some(*vk_code));
    }

    // Handle alphanumeric keys dynamically
    if key.len() == 1 {
        let ch = key.chars().next().unwrap();
        if ch.is_alphabetic() {
            let upper = ch.to_uppercase().to_string();
            return (format!("Key{}", upper), Some(ch.to_uppercase().next().unwrap() as u32));
        } else if ch.is_ascii_digit() {
            return (format!("Digit{}", ch), Some(ch as u32));
        }
    }

    // Fallback: use the key name as code, no virtual key code
    (key.to_string(), None)
}

