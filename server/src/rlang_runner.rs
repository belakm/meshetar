use std::process::Command;

pub fn r_script(path_to_script: &str, arguments: Option<&str>) -> Result<String, String> {
    // Set the Rscript command and the path to the R script
    let rscript_cmd = "Rscript";
    let output: std::process::Output;
    match arguments {
        None => {
            // Execute the Rscript command with the input parameters
            output = Command::new(rscript_cmd)
                .arg(path_to_script)
                .stderr(std::process::Stdio::piped())
                .output()
                .expect("Failed to execute Rscript command");
        }
        Some(argument) => {
            // Execute the Rscript command with the input parameters
            output = Command::new(rscript_cmd)
                .arg(path_to_script)
                .arg(argument)
                .stderr(std::process::Stdio::piped())
                .output()
                .expect("Failed to execute Rscript command");
        }
    }
    // Check if the R script returned a success code
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        let status = output.status.to_string();
        let error_string = "Status: ".to_owned() + &status + " - error: " + &error;
        Err(error_string)
    }
}
