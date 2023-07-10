use futures::FutureExt;
use std::{process::Stdio, sync::Arc};
use tokio::io::AsyncReadExt;
use tokio::{process::Command, sync::Mutex};

use crate::TaskControl;

pub async fn run_script(
    path_to_script: &str,
    task_control: Arc<Mutex<TaskControl>>,
) -> Result<String, String> {
    // Set the Rscript command and the path to the R script
    let child_process = Command::new("Rscript")
        .arg(path_to_script)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    match child_process {
        Ok(mut child_process) => {
            let mut receiver = task_control.lock().await.receiver.clone();
            loop {
                tokio::select! {
                    _ = receiver.changed() => {
                        if *receiver.borrow() == false {
                            &child_process.kill().await;
                            break Err("R Command: execution was stopped".to_owned())
                        }
                    }
                    output = read_command_output(&mut child_process).fuse() => {
                        break match output {
                            Ok(output) => {
                                Ok(output)
                            },
                            Err(e) => Err(format!("R err: {:?}", e))
                        }
                    }
                }
            }
        }
        Err(e) => Err(format!("R err: {:?}", e)),
    }
}

pub async fn read_command_output(
    child_process: &mut tokio::process::Child,
) -> Result<String, String> {
    let stdout = child_process.stdout.take();
    match stdout {
        Some(mut stdout) => {
            let mut output = Vec::new();
            match stdout.read_to_end(&mut output).await {
                Ok(_) => Ok(String::from_utf8_lossy(&output).to_string()),
                Err(e) => Err(format!("Error parsing output: {:?}", e)),
            }
        }
        None => Err("R command: No output".to_string()),
    }
}
