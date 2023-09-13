extern crate gtk;
use gtk::prelude::*;
use std::fs;
use std::fs::DirEntry;
use std::path::Path;
use std::collections::HashMap;
use rand::Rng;

fn main() {
    // Initialize GTK
    gtk::init().expect("Failed to initialize GTK.");

    // Create a new GTK window
    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    window.set_title("File Sorter");
    window.set_default_size(400, 200);

    // Create a vertical box container
    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 5);
    window.add(&vbox);

    // Create a label and entry for directory input
    let label = gtk::Label::new(Some("Enter directory path:"));
    vbox.pack_start(&label, false, false, 0);

    let directory_entry = gtk::Entry::new();
    vbox.pack_start(&directory_entry, false, false, 0);

    // Create radio buttons for sorting criteria
    let radio_button1 = gtk::RadioButton::with_label("Sort by Extension");
    vbox.pack_start(&radio_button1, false, false, 0);

    let radio_button2 = gtk::RadioButton::with_label_from_widget(&radio_button1, "Sort by Name");
    vbox.pack_start(&radio_button2, false, false, 0);

    let radio_button3 = gtk::RadioButton::with_label_from_widget(&radio_button1, "Sort by Modification Date");
    vbox.pack_start(&radio_button3, false, false, 0);

    // Create a button to trigger sorting
    let sort_button = gtk::Button::with_label("Sort Files");
    vbox.pack_start(&sort_button, false, false, 0);

    // Create a text view to display the sorting result
    let text_view = gtk::TextView::new();
    let text_buffer = text_view.get_buffer().unwrap();
    vbox.pack_start(&text_view, true, true, 0);

    // Handle button click event
    sort_button.connect_clicked(move |_| {
        let directory_path = directory_entry.get_text().to_string();
        let result = if radio_button2.get_active() {
            sort_files_by_name(&directory_path)
        } else if radio_button3.get_active() {
            sort_files_by_modification_date(&directory_path)
        } else {
            sort_files_by_extension(&directory_path)
        };
        text_buffer.set_text(&result);
    });

    // Handle window close event
    window.connect_delete_event(|_, _| {
        // Terminate the GTK main loop
        gtk::main_quit();
        Inhibit(false)
    });

    // Show all widgets
    window.show_all();

    // Start the GTK main loop
    gtk::main();
}

fn sort_files_by_extension(directory_path: &str) -> String {
   let mut result = String::new();

    let dir = match fs::read_dir(directory_path) {
        Ok(d) => d,
        Err(e) => {
            return format!("Error: {}", e);
        }
    };

    let mut file_map: HashMap<String, Vec<DirEntry>> = HashMap::new();

    for entry in dir {
        if let Ok(entry) = entry {
            if entry.path().is_file() {
                let file_ext = match entry.path().extension() {
                    Some(ext) => ext.to_string_lossy().to_string(),
                    None => "Other".to_string(),
                };

                if !file_map.contains_key(&file_ext) {
                    file_map.insert(file_ext.clone(), Vec::new());
                }

                file_map.get_mut(&file_ext).unwrap().push(entry);
            }
        }
    }

    for (ext, entries) in file_map.iter_mut() {
        let target_dir = Path::new(directory_path).join(ext);
        if !target_dir.exists() {
            fs::create_dir(&target_dir).expect("Failed to create directory");
        }

        for entry in entries.iter_mut() {
            let file_name = entry.file_name();
            let target_path = target_dir.join(file_name.clone());

            // Handle collisions by appending random numbers
            if target_path.exists() {
                let mut new_target_path = target_dir.clone();
                new_target_path.push(format!(
                    "{}_{:016x}",
                    ext,
                    rand::thread_rng().gen::<u64>()
                ));
                fs::rename(entry.path(), &new_target_path).expect("Failed to rename file");
            } else {
                fs::rename(entry.path(), &target_path).expect("Failed to rename file");
            }

            result.push_str(&format!(
                "Moved {} to {}\n",
                file_name.to_string_lossy(),
                target_path.to_string_lossy()
            ));
        }
    }

    result
}

fn sort_files_by_name(directory_path: &str) -> String {
    let mut result = String::new();

    let dir = match fs::read_dir(directory_path) {
        Ok(d) => d,
        Err(e) => {
            return format!("Error: {}", e);
        }
    };

    let mut entries: Vec<DirEntry> = dir.filter_map(Result::ok).collect();
    entries.sort_by_key(|dir| dir.file_name());

    for entry in entries {
        if entry.path().is_file() {
            let file_name = entry.file_name();
            let first_letter = file_name.to_string_lossy().chars().next().unwrap_or('_').to_string();

            let target_dir = Path::new(directory_path).join(first_letter.clone());
            if !target_dir.exists() {
                fs::create_dir(&target_dir).expect("Failed to create directory");
            }

            let target_path = target_dir.join(file_name.clone());

            // Handle collisions by appending random numbers
            if target_path.exists() {
                let mut new_target_path = target_dir.clone();
                new_target_path.push(format!(
                    "{}_{:016x}",
                    first_letter,
                    rand::thread_rng().gen::<u64>()
                ));
                fs::rename(entry.path(), &new_target_path).expect("Failed to rename file");
            } else {
                fs::rename(entry.path(), &target_path).expect("Failed to rename file");
            }

            result.push_str(&format!(
                "Moved {} to {}\n",
                file_name.to_string_lossy(),
                target_path.to_string_lossy()
            ));
        }
    }

    result
}

fn sort_files_by_modification_date(directory_path: &str) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let mut result = String::new();

    let dir = match fs::read_dir(directory_path) {
        Ok(d) => d,
        Err(e) => {
            return format!("Error: {}", e);
        }
    };

    let mut entries: Vec<DirEntry> = dir.filter_map(Result::ok).collect();
    entries.sort_by_key(|dir| dir.metadata().unwrap().modified().unwrap_or(SystemTime::now()));

    for entry in entries {
        if entry.path().is_file() {
            let file_name = entry.file_name();
            let modified_date = entry.metadata().unwrap().modified().unwrap_or(SystemTime::now());
            let since_the_epoch = modified_date.duration_since(UNIX_EPOCH).unwrap();
            let in_seconds = since_the_epoch.as_secs();
            let days_since_epoch = in_seconds / (60 * 60 * 24);

            let target_dir = Path::new(directory_path).join(days_since_epoch.to_string());
            if !target_dir.exists() {
                fs::create_dir(&target_dir).expect("Failed to create directory");
            }

            let target_path = target_dir.join(file_name.clone());

            // Handle collisions by appending random numbers
            if target_path.exists() {
                let mut new_target_path = target_dir.clone();
                new_target_path.push(format!(
                    "{}_{:016x}",
                    days_since_epoch,
                    rand::thread_rng().gen::<u64>()
                ));
                fs::rename(entry.path(), &new_target_path).expect("Failed to rename file");
            } else {
                fs::rename(entry.path(), &target_path).expect("Failed to rename file");
            }

            result.push_str(&format!(
                "Moved {} to {}\n",
                file_name.to_string_lossy(),
                target_path.to_string_lossy()
            ));
        }
    }

    result
}
