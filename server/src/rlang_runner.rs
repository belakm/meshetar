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
    let mut child_process = Command::new("Rscript")
        .arg(path_to_script)
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| format!("R err: {:?}", e))?;

    let mut output = String::new();
    let mut reader = if let Some(stdout) = child_process.stdout.take() {
        Some(tokio::io::BufReader::new(stdout))
    } else {
        None
    };

    let mut receiver = task_control.lock().await.receiver.clone();
    loop {
        tokio::select! {
            result = reader.as_mut().unwrap().read_to_string(&mut output).fuse() => {
                result.map_err(|e| format!("Error parsing output: {:?}", e))?;
            },
            _ = receiver.changed() => {
                if *receiver.borrow() == false {
                    child_process.kill().await.map_err(|e| format!("Failed to stop command: {:?}", e))?;
                    return Err("R Command: execution was stopped".to_owned())
                }
            }
        }

        let exit_status = child_process
            .try_wait()
            .map_err(|e| format!("Failed to check child status: {:?}", e))?;

        if let Some(exit_status) = exit_status {
            if exit_status.success() {
                return Ok(output);
            } else {
                return Err(format!(
                    "R script returned with error status: {:?}",
                    exit_status
                ));
            }
        }
    }
}
