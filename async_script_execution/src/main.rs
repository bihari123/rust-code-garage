use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::os::unix::process::ExitStatusExt;
use std::process::{exit, Child, Command};
use std::thread::sleep;
use std::time::Duration;

fn main() {
    let mut child_processes: HashMap<u32, Child> = HashMap::new();
    let script_names = ["./src/script1.sh", "./src/script2.sh", "./src/script3.sh"];

    for (index, script_name) in script_names.iter().enumerate() {
        println!("The script to execute is {}", script_name);
        let script_command = Command::new("bash")
            .arg("-c")
            .arg(script_name)
            .stderr(
                File::create(format!("{}.error", script_name))
                    .expect("Failed to create error file."),
            )
            .spawn()
            .expect("Failed to execute script.");

        let script_pid = script_command.id();
        println!(
            "Script '{}' started execution. PID: {}",
            script_name, script_pid
        );

        child_processes.insert(index as u32, script_command);

        // Do other tasks or spawn additional child processes if needed
    }

    // Periodically check the status of each child process
    loop {
        let mut completed_processes: Vec<u32> = Vec::new();

        for (pid, process) in &mut child_processes {
            match process.try_wait() {
                Ok(Some(exit_status)) => {
                    let exit_code = exit_status.code().unwrap_or_else(|| {
                        // If the script was terminated by a signal, print an error message
                        let signal = exit_status
                            .signal()
                            .expect("Failed to retrieve script termination signal");
                        println!("Script with PID {} terminated by signal: {}", pid, signal);
                        1 // Default exit code for signal termination
                    });

                    println!(
                        "Script with PID {} completed with exit code: {} and output ",
                        pid, exit_code
                    );

                    // Read the error output from the file
                    let mut error_content = String::new();
                    File::open(format!("{}.error", script_names[*pid as usize]))
                        .and_then(|mut file| file.read_to_string(&mut error_content))
                        .expect("Failed to read error file.");

                    println!("Error output for PID {}: {}", pid, error_content);

                    // Remove the completed process and its error file from the list
                    completed_processes.push(*pid);
                }
                Ok(None) => {
                    // The process is still running
                    // Perform any necessary operations
                }
                Err(err) => {
                    println!("Failed to check status for PID {}: {}", pid, err);
                }
            }
        }

        // Remove the completed processes and their error files from the HashMap
        for pid in completed_processes {
            child_processes.remove(&pid);
            std::fs::remove_file(format!("{}.error", script_names[pid as usize]))
                .expect("Failed to delete error file.");
        }

        if child_processes.is_empty() {
            // All child processes have completed
            break;
        }

        // Sleep for a short duration before checking again
        sleep(Duration::from_millis(100));
    }

    // Perform any necessary cleanup or finalize the program
    exit(0);
}
